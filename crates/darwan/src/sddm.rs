use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::paths::{self, HELPER, Paths};
use darwan_core::{ini, resolve};

use crate::session::WaylandSession;
use crate::style;

pub const INSTALLED_THEMES: &str = "/usr/share/darwan/themes";
pub const SDDM_THEMES: &str = "/usr/share/sddm/themes";

struct Planned {
    theme: Theme,
    overlay: BTreeMap<String, String>,
    config: UserConfig,
}

// Strict, unlike the lock: a bad setting would otherwise reach a root-written file.
fn plan(paths: &Paths, id: Option<&str>) -> Result<Planned, String> {
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let id = match id {
        Some(id) => id.to_string(),
        None => config
            .theme(Target::Sddm)
            .map_err(|e| format!("{}: {e}", config_path.display()))?
            .ok_or("no SDDM theme set: pass a theme id, or set [sddm] theme in the config")?
            .to_string(),
    };
    let (catalog, _) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let theme = catalog
        .into_themes()
        .into_iter()
        .find(|t| t.id == id)
        .ok_or_else(|| darwan_core::catalog::unknown_theme(&id))?;
    let resolved = resolve::resolve(&theme, &config, &darwan_core::system::SystemHost::new());
    if !resolved.issues.is_empty() {
        let list: Vec<String> = resolved
            .issues
            .iter()
            .map(|i| format!("  {}: {}", i.key, i.message))
            .collect();
        return Err(format!("fix these settings first:\n{}", list.join("\n")));
    }
    Ok(Planned {
        theme,
        overlay: resolved.overlay,
        config,
    })
}

pub fn apply(paths: &Paths, id: Option<&str>) -> Result<ExitCode, String> {
    let Planned {
        theme,
        overlay,
        mut config,
    } = plan(paths, id)?;
    if !Path::new(INSTALLED_THEMES)
        .join(&theme.id)
        .join("Main.qml")
        .is_file()
    {
        return Err(format!(
            "{} is not installed under {INSTALLED_THEMES}; SDDM can only use installed themes",
            theme.id
        ));
    }
    println!(
        "{} {} {}",
        style::bold("SDDM theme:"),
        style::id(&theme.id),
        style::dim(format!("({})", theme.manifest.name))
    );
    for (k, v) in &overlay {
        println!("  {} = {}", style::id(k), style::value(v));
    }
    let (header, files) = request(&overlay)?;
    run_helper_streaming(&["sddm-apply", &theme.id], &header, &files)?;

    config
        .set_theme(Target::Sddm, &theme.id)
        .map_err(|e| format!("{}: {e}", paths::config_file().display()))?;
    let config_path = paths::config_file();
    config
        .save(&config_path)
        .map_err(|e| format!("{}: {e}", config_path.display()))?;
    Ok(ExitCode::SUCCESS)
}

pub fn reset() -> Result<ExitCode, String> {
    run_helper(&["sddm-reset"], b"")?;
    Ok(ExitCode::SUCCESS)
}

// Overlay keys that name one of the user's files; SDDM can't read home folders, so the helper stores a copy.
const FILE_KEYS: [&str; 3] = ["backgroundPath", "fontTextFile", "fontClockFile"];

// The helper's request line, and the files whose bytes follow it in the same order.
fn request(overlay: &BTreeMap<String, String>) -> Result<(Vec<u8>, Vec<PathBuf>), String> {
    let mut sent = overlay.clone();
    let mut attachments = Vec::new();
    let mut files = Vec::new();
    for key in FILE_KEYS {
        let Some(path) = overlay.get(key).map(PathBuf::from) else {
            continue;
        };
        let size = std::fs::metadata(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .len();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        attachments.push(serde_json::json!({ "key": key, "ext": ext, "size": size }));
        sent.insert(key.to_string(), "@attachment".to_string());
        files.push(path);
    }
    let mut header =
        serde_json::to_vec(&serde_json::json!({ "overlay": sent, "attachments": attachments }))
            .map_err(|e| e.to_string())?;
    header.push(b'\n');
    Ok((header, files))
}

fn run_helper_streaming(args: &[&str], header: &[u8], files: &[PathBuf]) -> Result<(), String> {
    run_helper_with(args, |stdin| {
        stdin.write_all(header)?;
        for f in files {
            std::io::copy(&mut std::fs::File::open(f)?, stdin)?;
        }
        Ok(())
    })
}

pub fn run_helper(args: &[&str], stdin: &[u8]) -> Result<(), String> {
    run_helper_with(args, |w| w.write_all(stdin))
}

fn run_helper_with(
    args: &[&str],
    feed: impl FnOnce(&mut std::process::ChildStdin) -> std::io::Result<()>,
) -> Result<(), String> {
    if !Path::new(HELPER).is_file() {
        return Err(format!("{HELPER} is not installed"));
    }
    let mut child = Command::new("pkexec")
        .arg(HELPER)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run pkexec: {e}"))?;
    {
        let mut stdin = child.stdin.take().expect("stdin is piped");
        feed(&mut stdin).map_err(|e| format!("writing to the helper: {e}"))?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    match status.code() {
        Some(0) => Ok(()),
        Some(126 | 127) => Err("authorisation was refused or cancelled".into()),
        _ => Err("the helper failed (see its message above)".into()),
    }
}

pub fn preview(paths: &Paths, id: Option<&str>) -> Result<ExitCode, String> {
    let wayland = WaylandSession::discover()?;
    let Planned { theme, overlay, .. } = plan(paths, id)?;
    let work = tempfile::tempdir().map_err(|e| e.to_string())?;
    let dir = work.path().join(theme.id.replace('/', "_"));
    std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(&theme.dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
    {
        if entry.file_name() != "theme.conf.user" {
            std::os::unix::fs::symlink(entry.path(), dir.join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
    }
    let text = ini::write_general(&overlay).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("theme.conf.user"), text).map_err(|e| e.to_string())?;

    println!(
        "Previewing {} as SDDM shows it. {}",
        style::id(&theme.id),
        style::dim("Close the window to exit.")
    );
    let mut cmd = Command::new("sddm-greeter-qt6");
    cmd.arg("--test-mode").arg("--theme").arg(&dir);
    wayland.apply(&mut cmd);
    let status = cmd
        .status()
        .map_err(|e| format!("cannot start sddm-greeter-qt6: {e}"))?;
    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

pub struct Effective {
    pub current: Option<(String, PathBuf)>,
    pub files: Vec<PathBuf>,
}

// SDDM reads /usr/lib/sddm/sddm.conf.d, then /etc/sddm.conf.d, then /etc/sddm.conf; later files win.
pub fn effective_theme() -> Effective {
    let mut files = Vec::new();
    for dir in ["/usr/lib/sddm/sddm.conf.d", "/etc/sddm.conf.d"] {
        let mut in_dir: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|rd| rd.filter_map(Result::ok).map(|e| e.path()).collect())
            .unwrap_or_default();
        in_dir.sort();
        files.extend(in_dir.into_iter().filter(|p| p.is_file()));
    }
    if Path::new("/etc/sddm.conf").is_file() {
        files.push("/etc/sddm.conf".into());
    }
    let mut current = None;
    for file in &files {
        if let Some(value) = std::fs::read_to_string(file)
            .ok()
            .and_then(|t| theme_current(&t))
        {
            current = Some((value, file.clone()));
        }
    }
    Effective { current, files }
}

// The theme link darwan's own config file selects, even when another file overrides it.
pub fn darwan_current() -> Option<String> {
    std::fs::read_to_string("/etc/sddm.conf.d/zz-darwan.conf")
        .ok()
        .and_then(|t| theme_current(&t))
}

fn theme_current(text: &str) -> Option<String> {
    let mut in_theme = false;
    let mut found = None;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_theme = line == "[Theme]";
        } else if in_theme
            && let Some(v) = line
                .strip_prefix("Current")
                .and_then(|r| r.trim_start().strip_prefix('='))
        {
            found = Some(v.trim().to_string());
        }
    }
    found
}

pub fn status() -> Result<ExitCode, String> {
    let eff = effective_theme();
    match &eff.current {
        None => println!(
            "{} {}",
            style::bold("SDDM theme:"),
            style::dim("not set (SDDM's built-in default)")
        ),
        Some((name, file)) => println!(
            "{} {} {}",
            style::bold("SDDM theme:"),
            style::id(name),
            style::dim(format!("(set in {})", file.display()))
        ),
    }
    let ours = darwan_current();
    match ours
        .as_ref()
        .map(|n| std::fs::read_link(Path::new(SDDM_THEMES).join(n)))
    {
        Some(Ok(target)) if !target.join("Main.qml").is_file() => {
            println!(
                "{} {}",
                style::bold("darwan theme:"),
                style::fail(format!(
                    "BROKEN, the link points to {}, which no longer exists",
                    target.display()
                ))
            );
            println!(
                "  SDDM falls back to its built-in theme; run `darwan sddm reset` or apply another theme"
            );
        }
        Some(Ok(target)) => {
            let id = target
                .strip_prefix(INSTALLED_THEMES)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| target.display().to_string());
            println!("{} {}", style::bold("darwan theme:"), style::id(&id));
            match std::fs::read_to_string(target.join("theme.conf.user")) {
                Ok(text) => {
                    for (k, v) in ini::parse_general(&text) {
                        println!("  {} = {}", style::id(k), style::value(v));
                    }
                }
                Err(_) => println!("  {}", style::dim("(theme defaults)")),
            }
            if eff
                .current
                .as_ref()
                .is_some_and(|(name, _)| Some(name) != ours.as_ref())
            {
                println!(
                    "{} darwan is set up, but another config file overrides it (see `darwan doctor`)",
                    style::warn("warning:")
                );
            }
        }
        _ => println!(
            "{} {}",
            style::bold("darwan theme:"),
            style::dim("not applied")
        ),
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_is_read_only_from_the_theme_section_and_the_last_one_wins() {
        assert_eq!(
            theme_current("[General]\nCurrent=x\n[Theme]\nCurrent = a\nCurrent=b\n"),
            Some("b".into())
        );
        assert_eq!(theme_current("[Theme]\nCurrentTheme=x\n"), None);
        assert_eq!(theme_current("[Autologin]\nCurrent=x\n"), None);
    }

    #[test]
    fn user_files_become_attachments_and_other_values_stay_in_the_overlay() {
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("Rain.MP4");
        std::fs::write(&clip, b"12345").unwrap();
        let overlay: BTreeMap<String, String> = [
            ("backgroundType", "video"),
            ("backgroundPath", clip.to_str().unwrap()),
            ("colorAccent", "#e6bb5c"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        let (header, files) = request(&overlay).unwrap();
        assert_eq!(header.last(), Some(&b'\n'));
        let v: serde_json::Value = serde_json::from_slice(&header).unwrap();
        assert_eq!(
            v["overlay"]["backgroundPath"], "@attachment",
            "the helper never gets a path to open"
        );
        assert_eq!(v["overlay"]["colorAccent"], "#e6bb5c");
        assert_eq!(
            v["attachments"][0],
            serde_json::json!({ "key": "backgroundPath", "ext": "mp4", "size": 5 })
        );
        assert_eq!(files, [clip]);
        let (plain, none) = request(&BTreeMap::new()).unwrap();
        assert!(none.is_empty());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&plain).unwrap()["attachments"],
            serde_json::json!([])
        );
    }
}
