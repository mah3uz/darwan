use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::custom::Wallpaper;

pub mod filter;
pub mod library;
pub mod online;
pub mod set;

// Where to look; `run` executes a query command. Injected so every provider can be tested on fixtures.
pub struct Env {
    pub proc_dir: PathBuf,
    pub home: PathBuf,
    pub state_home: PathBuf,
    pub config_home: PathBuf,
    pub uid: u32,
    pub wayland_display: Option<String>,
    // XDG_CURRENT_DESKTOP, lower-cased: which desktop environment's own background applies.
    pub current_desktop: Option<String>,
    pub run: fn(&str, &[&str]) -> Option<String>,
    pub which: fn(&str) -> bool,
}

fn xdg(var: &str, home: &Path, fallback: &str) -> PathBuf {
    match std::env::var_os(var) {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => home.join(fallback),
    }
}

fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

impl Env {
    pub fn system() -> Self {
        let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
        Self {
            proc_dir: PathBuf::from("/proc"),
            state_home: xdg("XDG_STATE_HOME", &home, ".local/state"),
            config_home: xdg("XDG_CONFIG_HOME", &home, ".config"),
            home,
            uid: own_uid(),
            wayland_display: std::env::var("WAYLAND_DISPLAY")
                .ok()
                .filter(|d| !d.is_empty()),
            current_desktop: std::env::var("XDG_CURRENT_DESKTOP")
                .ok()
                .filter(|d| !d.is_empty())
                .map(|d| d.to_lowercase()),
            run: run_command,
            which: on_path,
        }
    }
}

pub(crate) fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|dir| {
            std::fs::metadata(dir.join(program)).is_ok_and(|m| {
                m.is_file()
                    && std::os::unix::fs::PermissionsExt::mode(&m.permissions()) & 0o111 != 0
            })
        })
    })
}

fn own_uid() -> u32 {
    std::fs::metadata("/proc/self").map_or(0, |m| std::os::unix::fs::MetadataExt::uid(&m))
}

#[derive(Debug, Clone)]
struct Proc {
    pid: u32,
    comm: String,
    argv: Vec<String>,
    dir: PathBuf,
}

impl Proc {
    fn exe_name(&self) -> &str {
        self.argv
            .first()
            .and_then(|a| Path::new(a).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&self.comm)
    }

    fn mentions(&self, needle: &str) -> bool {
        self.argv.iter().any(|a| a.contains(needle))
    }

    // `-c name` or `--config name`, as Quickshell and its launchers take a named config.
    fn config_named(&self, name: &str) -> bool {
        self.argv
            .windows(2)
            .any(|w| (w[0] == "-c" || w[0] == "--config") && w[1] == name)
            || self.argv.iter().any(|a| a == &format!("--config={name}"))
    }

    fn is_quickshell(&self) -> bool {
        matches!(self.exe_name(), "qs" | "quickshell")
    }

    // The fields after the command name in /proc/<pid>/stat: [0] is the state, [1] the parent, [19] the start time.
    fn stat(&self) -> Vec<String> {
        let text = std::fs::read_to_string(self.dir.join("stat")).unwrap_or_default();
        text.rsplit_once(')')
            .map(|(_, rest)| rest.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn parent(&self) -> Option<u32> {
        self.stat().get(1)?.parse().ok()
    }

    fn start_time(&self) -> Option<u64> {
        self.stat().get(19)?.parse().ok()
    }

    // The systemd unit a process runs in, e.g. `swaybg.service`; scopes (apps started by a launcher) don't count.
    fn service(&self) -> Option<String> {
        let text = std::fs::read_to_string(self.dir.join("cgroup")).ok()?;
        let last = text
            .lines()
            .find_map(|l| l.strip_prefix("0::"))?
            .rsplit('/')
            .next()?;
        last.ends_with(".service").then(|| last.to_string())
    }
}

// Each shell by the shape of its own process, never by a name anywhere in argv: an editor open on
// `~/Projects/omarchy` mentions Omarchy too.
fn is_dms(p: &Proc) -> bool {
    p.exe_name() == "dms" && p.argv.iter().skip(1).any(|a| a == "run")
}

fn is_omarchy_shell(p: &Proc) -> bool {
    p.is_quickshell() && p.mentions("omarchy/shell")
}

fn is_omarchy_swaybg(p: &Proc) -> bool {
    p.exe_name() == "swaybg" && p.mentions("omarchy/current/background")
}

fn is_caelestia(p: &Proc) -> bool {
    p.is_quickshell() && p.config_named("caelestia")
}

fn is_noctalia(p: &Proc) -> bool {
    p.exe_name() == "noctalia"
}

fn is_noctalia_legacy(p: &Proc) -> bool {
    p.is_quickshell() && p.config_named("noctalia-shell")
}

// awww and swww daemons started with a namespace (`-n backdrop`) draw somewhere else, e.g. niri's overview.
fn is_default_namespace(p: &Proc) -> bool {
    !p.argv
        .iter()
        .any(|a| a == "-n" || a == "--namespace" || a.starts_with("--namespace="))
}

// This user's processes in this Wayland session: a daemon from another session draws somewhere else.
fn processes(env: &Env) -> Vec<Proc> {
    let Ok(entries) = std::fs::read_dir(&env.proc_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
        })
        .filter(|e| {
            std::fs::metadata(e.path())
                .is_ok_and(|m| std::os::unix::fs::MetadataExt::uid(&m) == env.uid)
        })
        .filter_map(|e| {
            let dir = e.path();
            let argv: Vec<String> = std::fs::read(dir.join("cmdline"))
                .ok()?
                .split(|b| *b == 0)
                .filter(|a| !a.is_empty())
                .map(|a| String::from_utf8_lossy(a).into_owned())
                .collect();
            if argv.is_empty() {
                return None;
            }
            if let (Some(want), Ok(environ)) =
                (&env.wayland_display, std::fs::read(dir.join("environ")))
            {
                let display = environ
                    .split(|b| *b == 0)
                    .find_map(|kv| kv.strip_prefix(b"WAYLAND_DISPLAY="));
                if display.is_some_and(|d| d != want.as_bytes()) {
                    return None;
                }
            }
            let comm = std::fs::read_to_string(dir.join("comm"))
                .unwrap_or_default()
                .trim()
                .to_string();
            let pid = e.file_name().to_str()?.parse().ok()?;
            Some(Proc {
                pid,
                comm,
                argv,
                dir,
            })
        })
        .collect()
}

fn existing(path: &str) -> Option<PathBuf> {
    let p = PathBuf::from(path.trim());
    p.is_file().then_some(p)
}

fn dms(env: &Env, dark: Option<bool>) -> Option<PathBuf> {
    let text =
        std::fs::read_to_string(env.state_home.join("DankMaterialShell/session.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let get = |k: &str| {
        v.get(k)
            .and_then(serde_json::Value::as_str)
            .and_then(existing)
    };
    let variant = match dark {
        Some(true) => get("wallpaperPathDark"),
        Some(false) => get("wallpaperPathLight"),
        None => None,
    };
    variant.or_else(|| get("wallpaperPath"))
}

fn omarchy(env: &Env) -> Option<PathBuf> {
    for link in [
        env.state_home.join("omarchy/current/background"),
        env.config_home.join("omarchy/current/background"),
    ] {
        if let Ok(target) = std::fs::canonicalize(&link)
            && target.is_file()
        {
            return Some(target);
        }
    }
    None
}

fn caelestia(env: &Env) -> Option<PathBuf> {
    existing(&std::fs::read_to_string(env.state_home.join("caelestia/wallpaper/path.txt")).ok()?)
}

// settings.toml overrides Noctalia's declarative *.toml; a colour is stored as `color:#RRGGBB`.
fn noctalia(env: &Env) -> Option<PathBuf> {
    let text = std::fs::read_to_string(env.config_home.join("noctalia/settings.toml")).ok()?;
    let v: toml::Table = text.parse().ok()?;
    let path = v.get("wallpaper")?.get("default")?.get("path")?.as_str()?;
    match path.strip_prefix("color:") {
        Some(c) => Some(PathBuf::from(c)),
        None => existing(path),
    }
}

fn hyprpaper(env: &Env) -> Option<PathBuf> {
    let out = (env.run)("hyprctl", &["hyprpaper", "listactive"])?;
    out.lines()
        .find_map(|l| l.split_once(": ").and_then(|(_, p)| existing(p)))
}

fn swww_like(env: &Env, program: &str) -> Option<PathBuf> {
    let out = (env.run)(program, &["query"])?;
    out.lines().find_map(|l| {
        let shown = l.split_once("currently displaying: ")?.1;
        if let Some(p) = shown.strip_prefix("image: ") {
            existing(p)
        } else {
            shown
                .strip_prefix("color: ")
                .map(|c| PathBuf::from(format!("#{}", c.trim())))
        }
    })
}

// mpvpaper and gSlapper take the file as a positional argument, usually the last.
fn last_file_argument(p: &Proc) -> Option<PathBuf> {
    p.argv.iter().skip(1).rev().find_map(|a| existing(a))
}

fn swaybg(p: &Proc) -> Option<PathBuf> {
    p.argv
        .windows(2)
        .find(|w| w[0] == "-i" || w[0] == "--image")
        .and_then(|w| existing(&w[1]))
        .or_else(|| {
            p.argv
                .iter()
                .find_map(|a| a.strip_prefix("--image=").and_then(existing))
        })
}

fn waypaper(env: &Env) -> Option<PathBuf> {
    let text = std::fs::read_to_string(env.config_home.join("waypaper/config.ini")).ok()?;
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("wallpaper"))?;
    let (_, value) = line.split_once('=')?;
    let value = value.trim();
    existing(&match value.strip_prefix("~/") {
        Some(rest) => env.home.join(rest).display().to_string(),
        None => value.to_string(),
    })
}

fn found(path: PathBuf, source: &str) -> Wallpaper {
    Wallpaper {
        path,
        source: source.to_string(),
    }
}

// The drawer that is running wins; with none running, the most recently changed state file is used.
pub fn detect(env: &Env, dark: Option<bool>) -> Option<Wallpaper> {
    let procs = processes(env);

    // In order of precedence; each reads the file only when its drawer runs in this session.
    type Provider<'a> = (
        &'a str,
        Box<dyn Fn(&Proc) -> bool + 'a>,
        Box<dyn Fn(&Proc) -> Option<PathBuf> + 'a>,
    );
    let providers: Vec<Provider> = vec![
        ("DMS", Box::new(is_dms), Box::new(|_| dms(env, dark))),
        (
            "Omarchy",
            Box::new(|p| is_omarchy_shell(p) || is_omarchy_swaybg(p)),
            Box::new(|_| omarchy(env)),
        ),
        (
            "Caelestia",
            Box::new(is_caelestia),
            Box::new(|_| caelestia(env)),
        ),
        (
            "Noctalia",
            Box::new(|p| is_noctalia(p) || is_noctalia_legacy(p)),
            Box::new(|_| noctalia(env)),
        ),
        (
            "hyprpaper",
            Box::new(|p| p.exe_name() == "hyprpaper"),
            Box::new(|_| hyprpaper(env)),
        ),
        (
            "awww",
            Box::new(|p| p.exe_name() == "awww-daemon" && is_default_namespace(p)),
            Box::new(|_| swww_like(env, "awww")),
        ),
        (
            "swww",
            Box::new(|p| p.exe_name() == "swww-daemon" && is_default_namespace(p)),
            Box::new(|_| swww_like(env, "swww")),
        ),
        (
            "gSlapper",
            Box::new(|p| p.exe_name() == "gslapper"),
            Box::new(last_file_argument),
        ),
        (
            "mpvpaper",
            Box::new(|p| p.exe_name() == "mpvpaper"),
            Box::new(last_file_argument),
        ),
        (
            "swaybg",
            Box::new(|p| p.exe_name() == "swaybg"),
            Box::new(swaybg),
        ),
    ];
    for (name, is_it, read) in &providers {
        if let Some(w) = procs.iter().find(|p| is_it(p)).and_then(read) {
            return Some(found(w, name));
        }
    }
    if let Some(w) = waypaper(env) {
        return Some(found(w, "waypaper"));
    }

    let mut last_used: Vec<(std::time::SystemTime, Wallpaper)> = [
        (
            env.state_home.join("DankMaterialShell/session.json"),
            dms(env, dark),
            "DMS",
        ),
        (
            env.state_home.join("omarchy/current/background"),
            omarchy(env),
            "Omarchy",
        ),
        (
            env.state_home.join("caelestia/wallpaper/path.txt"),
            caelestia(env),
            "Caelestia",
        ),
    ]
    .into_iter()
    .filter_map(|(state, w, name)| {
        let changed = std::fs::symlink_metadata(state)
            .and_then(|m| m.modified())
            .ok()?;
        Some((changed, found(w?, &format!("{name}, last used"))))
    })
    .collect();
    last_used.sort_by_key(|(t, _)| *t);
    last_used.pop().map(|(_, w)| w)
}

// Running a wallpaper engine scene is not a file darwan can show.
pub fn unsupported_engine(env: &Env) -> bool {
    processes(env)
        .iter()
        .any(|p| p.exe_name().starts_with("linux-wallpaperengine"))
}

#[cfg(test)]
pub(crate) mod fixture {
    use super::*;

    pub(crate) struct Fixture {
        root: tempfile::TempDir,
    }

    impl Fixture {
        pub(crate) fn new() -> Self {
            let f = Self {
                root: tempfile::tempdir().unwrap(),
            };
            for d in ["proc", "home", "state", "config", "walls"] {
                std::fs::create_dir_all(f.root.path().join(d)).unwrap();
            }
            f
        }
        pub(crate) fn path(&self, p: &str) -> PathBuf {
            self.root.path().join(p)
        }
        pub(crate) fn wall(&self, name: &str) -> PathBuf {
            let p = self.path("walls").join(name);
            std::fs::write(&p, b"x").unwrap();
            p
        }
        pub(crate) fn process(&self, pid: u32, argv: &[&str], display: Option<&str>) {
            let dir = self.path("proc").join(pid.to_string());
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("cmdline"), argv.join("\0") + "\0").unwrap();
            std::fs::write(
                dir.join("comm"),
                Path::new(argv[0]).file_name().unwrap().to_str().unwrap(),
            )
            .unwrap();
            if let Some(d) = display {
                std::fs::write(
                    dir.join("environ"),
                    format!("HOME=/h\0WAYLAND_DISPLAY={d}\0"),
                )
                .unwrap();
            }
        }
        // `stat` as the kernel writes it: the command name in parentheses, then the fields the code reads.
        pub(crate) fn stat(&self, pid: u32, parent: u32, start: u64) {
            let mut fields = vec!["0".to_string(); 20];
            fields[0] = "S".into();
            fields[1] = parent.to_string();
            fields[19] = start.to_string();
            let dir = self.path("proc").join(pid.to_string());
            std::fs::write(
                dir.join("stat"),
                format!("{pid} (x y) {}\n", fields.join(" ")),
            )
            .unwrap();
        }
        pub(crate) fn cgroup(&self, pid: u32, unit: &str) {
            let dir = self.path("proc").join(pid.to_string());
            std::fs::write(
                dir.join("cgroup"),
                format!("0::/user.slice/user-1000.slice/user@1000.service/app.slice/{unit}\n"),
            )
            .unwrap();
        }
        pub(crate) fn write(&self, rel: &str, text: &str) {
            let p = self.path(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        pub(crate) fn env(&self, run: fn(&str, &[&str]) -> Option<String>) -> Env {
            Env {
                proc_dir: self.path("proc"),
                home: self.path("home"),
                state_home: self.path("state"),
                config_home: self.path("config"),
                uid: std::fs::metadata(self.root.path())
                    .map(|m| std::os::unix::fs::MetadataExt::uid(&m))
                    .unwrap(),
                wayland_display: Some("wayland-1".into()),
                current_desktop: None,
                run,
                which: |_| false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixture::Fixture;
    use super::*;

    fn no_commands(_: &str, _: &[&str]) -> Option<String> {
        None
    }

    #[test]
    fn dms_is_read_from_its_session_state_and_follows_the_variant() {
        let f = Fixture::new();
        let (main, light, dark) = (f.wall("geo.png"), f.wall("day.jpg"), f.wall("night.jpg"));
        f.process(10, &["/usr/bin/dms", "run", "--session"], Some("wayland-1"));
        f.write(
            "state/DankMaterialShell/session.json",
            &format!(
                r#"{{"wallpaperPath":"{}","wallpaperPathLight":"{}","wallpaperPathDark":"{}"}}"#,
                main.display(),
                light.display(),
                dark.display()
            ),
        );
        let env = f.env(no_commands);
        assert_eq!(detect(&env, None), Some(found(main, "DMS")));
        assert_eq!(detect(&env, Some(true)).unwrap().path, dark);
        assert_eq!(detect(&env, Some(false)).unwrap().path, light);
    }

    #[test]
    fn omarchy_follows_its_current_background_symlink_including_videos() {
        let f = Fixture::new();
        let clip = f.wall("loop.mp4");
        std::fs::create_dir_all(f.path("state/omarchy/current")).unwrap();
        std::os::unix::fs::symlink(&clip, f.path("state/omarchy/current/background")).unwrap();
        // As omarchy-launch-shell starts it.
        f.process(
            11,
            &[
                "quickshell",
                "-n",
                "-p",
                "/home/u/.local/share/omarchy/shell",
            ],
            None,
        );
        assert_eq!(
            detect(&f.env(no_commands), None),
            Some(found(clip, "Omarchy"))
        );
    }

    #[test]
    fn a_shell_is_known_by_its_process_not_by_its_name_in_someones_arguments() {
        let f = Fixture::new();
        let omarchy = f.wall("omarchy.png");
        std::fs::create_dir_all(f.path("state/omarchy/current")).unwrap();
        std::os::unix::fs::symlink(&omarchy, f.path("state/omarchy/current/background")).unwrap();
        let shown = f.wall("shown.png");
        // Editors open on checkouts of Omarchy and Caelestia aren't those shells drawing the desktop; swaybg is.
        f.process(
            21,
            &["nvim", "/home/u/Projects/omarchy/bin/omarchy-shell"],
            Some("wayland-1"),
        );
        f.process(
            22,
            &["zed", "/home/u/src/caelestia/shell.qml"],
            Some("wayland-1"),
        );
        f.process(
            23,
            &["swaybg", "-i", shown.to_str().unwrap()],
            Some("wayland-1"),
        );
        assert_eq!(
            detect(&f.env(no_commands), None),
            Some(found(shown, "swaybg"))
        );
    }

    #[test]
    fn caelestia_and_noctalia_state_files() {
        let f = Fixture::new();
        let w = f.wall("c.png");
        f.write(
            "state/caelestia/wallpaper/path.txt",
            &format!("{}\n", w.display()),
        );
        f.process(12, &["qs", "-c", "caelestia"], Some("wayland-1"));
        assert_eq!(
            detect(&f.env(no_commands), None).unwrap().source,
            "Caelestia"
        );

        let g = Fixture::new();
        g.write(
            "config/noctalia/settings.toml",
            "[wallpaper.default]\npath = \"color:#ff00ff\"\n",
        );
        g.process(13, &["/usr/bin/noctalia"], None);
        assert_eq!(
            detect(&g.env(no_commands), None).unwrap().path,
            PathBuf::from("#ff00ff")
        );
    }

    #[test]
    fn hyprpaper_and_awww_are_asked_through_their_query_commands() {
        let f = Fixture::new();
        f.process(14, &["hyprpaper"], Some("wayland-1"));
        fn hyprctl(program: &str, args: &[&str]) -> Option<String> {
            (program == "hyprctl" && args == ["hyprpaper", "listactive"])
                .then(|| "DP-1: /etc/hostname\n".to_string())
        }
        let w = detect(&f.env(hyprctl), None).unwrap();
        assert_eq!(
            (w.path, w.source.as_str()),
            (PathBuf::from("/etc/hostname"), "hyprpaper")
        );

        let g = Fixture::new();
        g.process(15, &["/usr/bin/awww-daemon"], None);
        fn awww(program: &str, _: &[&str]) -> Option<String> {
            (program == "awww").then(|| {
                "DP-1: 3840x2160, scale: 1, currently displaying: color: 1A2B3C\n".to_string()
            })
        }
        assert_eq!(
            detect(&g.env(awww), None).unwrap().path,
            PathBuf::from("#1A2B3C")
        );
    }

    #[test]
    fn mpvpaper_swaybg_and_waypaper() {
        let f = Fixture::new();
        let clip = f.wall("v.webm");
        f.process(
            16,
            &[
                "mpvpaper",
                "-o",
                "no-audio loop",
                "DP-1",
                clip.to_str().unwrap(),
            ],
            None,
        );
        assert_eq!(
            detect(&f.env(no_commands), None),
            Some(found(clip, "mpvpaper"))
        );

        let g = Fixture::new();
        let img = g.wall("s.jpg");
        g.process(
            17,
            &[
                "swaybg",
                "-o",
                "*",
                "-i",
                img.to_str().unwrap(),
                "-m",
                "fill",
            ],
            None,
        );
        assert_eq!(detect(&g.env(no_commands), None).unwrap().source, "swaybg");

        let h = Fixture::new();
        let pic = h.path("home/Pictures/w.png");
        std::fs::create_dir_all(pic.parent().unwrap()).unwrap();
        std::fs::write(&pic, b"x").unwrap();
        h.write(
            "config/waypaper/config.ini",
            "[Settings]\nwallpaper = ~/Pictures/w.png\n",
        );
        assert_eq!(
            detect(&h.env(no_commands), None),
            Some(found(pic, "waypaper"))
        );
    }

    #[test]
    fn a_daemon_in_another_wayland_session_is_not_this_sessions_wallpaper() {
        let f = Fixture::new();
        let w = f.wall("other.png");
        f.process(
            18,
            &["swaybg", "-i", w.to_str().unwrap()],
            Some("wayland-7"),
        );
        assert_eq!(detect(&f.env(no_commands), None), None);
    }

    #[test]
    fn with_nothing_running_the_most_recent_state_file_is_last_used() {
        let f = Fixture::new();
        let w = f.wall("geo.png");
        f.write(
            "state/DankMaterialShell/session.json",
            &format!(r#"{{"wallpaperPath":"{}"}}"#, w.display()),
        );
        let got = detect(&f.env(no_commands), None).unwrap();
        assert_eq!((got.path, got.source.as_str()), (w, "DMS, last used"));
    }

    #[test]
    fn a_missing_file_or_nothing_at_all_finds_nothing() {
        let f = Fixture::new();
        f.process(19, &["dms", "run"], None);
        f.write(
            "state/DankMaterialShell/session.json",
            r#"{"wallpaperPath":"/nope/gone.png"}"#,
        );
        assert_eq!(detect(&f.env(no_commands), None), None);
        assert_eq!(detect(&Fixture::new().env(no_commands), None), None);
    }
}
