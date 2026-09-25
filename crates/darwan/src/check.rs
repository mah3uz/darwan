use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use darwan_core::catalog::{Catalog, Theme};

use crate::paths::Paths;
use crate::qs;

pub struct Options {
    pub ids: Vec<String>,
    pub all: bool,
    pub no_fonts: bool,
    pub size: (u32, u32),
    pub shots: Option<PathBuf>,
    pub jobs: usize,
    pub timeout: Duration,
}

struct Outcome {
    id: String,
    verdict: String,
    problems: Vec<String>,
}

pub fn run(paths: &Paths, opts: Options) -> Result<ExitCode, String> {
    let (catalog, load_problems) = Catalog::load(&paths.themes()).map_err(|e| e.to_string())?;
    let mut failed = load_problems.len();
    for p in &load_problems {
        println!("FAIL  {}  cannot load: {}", p.id, p.message);
    }

    let themes: Vec<&Theme> = if opts.all {
        catalog.themes().iter().collect()
    } else if opts.ids.is_empty() {
        return Err("name one or more themes, or pass --all".into());
    } else {
        opts.ids
            .iter()
            .map(|id| {
                catalog
                    .get(id)
                    .ok_or_else(|| format!("unknown theme {id:?}"))
            })
            .collect::<Result<_, _>>()?
    };

    let work = tempfile::tempdir().map_err(|e| e.to_string())?;
    let screen = work.path().join("screen.json");
    let (w, h) = opts.size;
    std::fs::write(
        &screen,
        format!(r#"{{"screens":[{{"name":"darwan","x":0,"y":0,"width":{w},"height":{h},"logicalDpi":96,"logicalBaseDpi":96,"dpr":1}}]}}"#),
    )
    .map_err(|e| e.to_string())?;
    if let Some(dir) = &opts.shots {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }

    let queue = Mutex::new(themes.into_iter());
    let results = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..opts.jobs.max(1) {
            scope.spawn(|| {
                loop {
                    let Some(theme) = queue.lock().unwrap().next() else {
                        break;
                    };
                    let outcome = check_one(paths, theme, &opts, work.path(), &screen);
                    results.lock().unwrap().push(outcome);
                }
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.id.cmp(&b.id));
    for r in &results {
        if r.verdict.is_empty() && r.problems.is_empty() {
            println!("ok    {}", r.id);
            continue;
        }
        failed += 1;
        println!("FAIL  {}  {}", r.id, r.verdict);
        for p in &r.problems {
            println!("        {p}");
        }
    }
    println!("{} checked, {failed} failed", results.len());
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn check_one(paths: &Paths, theme: &Theme, opts: &Options, work: &Path, screen: &Path) -> Outcome {
    let slug = theme.id.replace('/', "_");
    let fail = |verdict: String| Outcome {
        id: theme.id.clone(),
        verdict,
        problems: Vec::new(),
    };

    let theme_dir = if opts.no_fonts {
        match mirror_without_fonts(&theme.dir, &work.join("mirror").join(&slug)) {
            Ok(dir) => dir,
            Err(e) => return fail(format!("cannot hide font/: {e}")),
        }
    } else {
        theme.dir.clone()
    };

    let log_path = work.join(format!("{slug}.log"));
    let log = match std::fs::File::create(&log_path) {
        Ok(f) => f,
        Err(e) => return fail(e.to_string()),
    };
    let mut cmd = qs::command(paths, "check_shell.qml", &[]);
    qs::theme_env(&mut cmd, theme, &theme_dir, None);
    cmd.env_remove("WAYLAND_DISPLAY")
        .env(
            "QT_QPA_PLATFORM",
            format!("offscreen:configfile={}", screen.display()),
        )
        .env("DARWAN_CHECK_FONTS", if opts.no_fonts { "1" } else { "0" })
        .env("DARWAN_USER", "traveler")
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log);
    if let Some(dir) = &opts.shots {
        cmd.env("DARWAN_SHOT", dir.join(format!("{slug}.png")));
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return fail(format!("cannot start quickshell: {e}")),
    };
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() > opts.timeout => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return fail(e.to_string()),
        }
    };

    let text = std::fs::read_to_string(&log_path).unwrap_or_default();
    // With font/ hidden on purpose, a font that fails to load is the premise, not a defect.
    let problems = qs::problem_lines(&text)
        .into_iter()
        .filter(|l| !(opts.no_fonts && l.contains("Cannot load font:")))
        .map(|l| l.replace(&format!("file://{}/", theme_dir.display()), ""))
        .collect();
    let verdict = match status.map(|s| s.code()) {
        None => format!("timed out after {}s", opts.timeout.as_secs()),
        Some(Some(0)) => String::new(),
        Some(Some(2)) => "theme failed to load; the fallback prompt was shown".into(),
        Some(Some(3)) => "text with no font family (missing font and no fallback)".into(),
        Some(Some(code)) => format!("quickshell exited with {code}"),
        Some(None) => "quickshell was killed by a signal".into(),
    };
    Outcome {
        id: theme.id.clone(),
        verdict,
        problems,
    }
}

// A fresh clone has no licensed fonts, so --no-fonts mirrors the theme without font/.
fn mirror_without_fonts(theme_dir: &Path, mirror: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(mirror)?;
    for entry in std::fs::read_dir(theme_dir)? {
        let entry = entry?;
        if entry.file_name() != "font" {
            std::os::unix::fs::symlink(entry.path(), mirror.join(entry.file_name()))?;
        }
    }
    Ok(mirror.to_path_buf())
}

pub fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s
        .split_once('x')
        .ok_or("expected WIDTHxHEIGHT, e.g. 1920x1080")?;
    let parse = |v: &str| v.parse::<u32>().ok().filter(|n| (320..=16384).contains(n));
    match (parse(w), parse(h)) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => Err(format!("invalid size {s:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_parses_and_rejects_nonsense() {
        assert_eq!(parse_size("3840x2160"), Ok((3840, 2160)));
        assert!(parse_size("1920").is_err());
        assert!(parse_size("0x0").is_err());
    }

    #[test]
    fn mirror_keeps_everything_but_font() {
        let src = tempfile::tempdir().unwrap();
        std::fs::create_dir(src.path().join("font")).unwrap();
        std::fs::write(src.path().join("Main.qml"), "").unwrap();
        let dst = tempfile::tempdir().unwrap();
        let m = mirror_without_fonts(src.path(), &dst.path().join("t")).unwrap();
        assert!(m.join("Main.qml").exists());
        assert!(!m.join("font").exists());
    }
}
