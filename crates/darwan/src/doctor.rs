use std::path::Path;
use std::process::{Command, ExitCode};

use darwan_core::catalog::Catalog;
use darwan_core::manifest::Background;
use darwan_core::paths::Paths;

use crate::sddm;
use crate::session::WaylandSession;
use crate::style;

#[derive(Default)]
struct Report {
    failed: usize,
}

impl Report {
    fn ok(&mut self, msg: impl AsRef<str>) {
        println!("{} {}", style::ok("ok   "), msg.as_ref());
    }
    fn warn(&mut self, msg: impl AsRef<str>) {
        println!("{} {}", style::warn("warn "), msg.as_ref());
    }
    fn fail(&mut self, msg: impl AsRef<str>) {
        self.failed += 1;
        println!("{} {}", style::fail("FAIL "), msg.as_ref());
    }
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(program).is_file()))
}

pub fn run(paths: &Paths) -> Result<ExitCode, String> {
    let mut r = Report::default();

    println!("{}", style::heading("Lockscreen"));
    let wayland = match WaylandSession::discover() {
        Ok(w) => {
            r.ok(format!("Wayland session {}", w.display));
            Some(w)
        }
        Err(e) => {
            r.fail(e);
            None
        }
    };
    if on_path("quickshell") {
        r.ok("quickshell is installed")
    } else {
        r.fail("quickshell is not installed")
    }
    if let Some(sig) = wayland.as_ref().and_then(|w| w.hyprland.as_ref()) {
        let out = Command::new("hyprctl")
            .args([
                "--instance",
                sig,
                "-j",
                "getoption",
                "misc:allow_session_lock_restore",
            ])
            .output()
            .ok()
            .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok());
        match out.and_then(|j| j.get("bool").and_then(serde_json::Value::as_bool)) {
            Some(true) => r.ok("Hyprland misc:allow_session_lock_restore is on"),
            Some(false) => r.fail("Hyprland misc:allow_session_lock_restore is off: a crashed lock can't be recovered; enable it in your Hyprland config"),
            None => r.warn("could not read Hyprland's misc:allow_session_lock_restore"),
        }
    }
    let hypridle = darwan_core::paths::config_file()
        .parent()
        .and_then(Path::parent)
        .map(|c| c.join("hypr/hypridle.conf"));
    if let Some(text) = hypridle.and_then(|p| std::fs::read_to_string(p).ok()) {
        match hypridle_advice(&text) {
            Some(Ok(msg)) => r.ok(msg),
            Some(Err(msg)) => r.warn(msg),
            None => {}
        }
    }

    {
        use darwan_core::custom::Host;
        let host = darwan_core::system::SystemHost::new();
        match host.desktop_wallpaper(None) {
            Some(w) => r.ok(format!("desktop wallpaper: {} ({})", w.path.display(), w.source)),
            None if darwan_core::wallpaper::unsupported_engine(&darwan_core::wallpaper::Env::system()) => r.warn(
                "the desktop wallpaper is a Wallpaper Engine scene; themes can't use it as a background",
            ),
            None => r.warn("no desktop wallpaper found; \"use my desktop wallpaper\" won't work"),
        }
        match host.desktop_prefers_dark() {
            Some(dark) => r.ok(format!(
                "the desktop prefers {} (auto light/dark follows it)",
                if dark { "dark" } else { "light" }
            )),
            None => {
                r.warn("the desktop states no light/dark preference; auto uses each theme's own")
            }
        }
    }

    println!("\n{}", style::heading("Themes"));
    let (catalog, problems) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    r.ok(format!(
        "{} themes in {}",
        catalog.themes().len(),
        paths.themes().display()
    ));
    for p in problems {
        r.fail(format!("{}: {}", p.id, p.message));
    }
    let ffmpeg = Path::new("/usr/lib/qt6/plugins/multimedia/libffmpegmediaplugin.so").is_file();
    let videos = catalog
        .themes()
        .iter()
        .filter(|t| t.manifest.background == Background::Video)
        .count();
    if ffmpeg {
        r.ok("Qt multimedia backend is installed (video themes)")
    } else {
        r.warn(format!(
            "qt6-multimedia-ffmpeg is not installed: {videos} video themes will show no background"
        ))
    }
    for theme in catalog.themes() {
        for font in &theme.manifest.fonts {
            if !theme.dir.join("font").join(&font.file).is_file() {
                let source = if font.url.is_empty() {
                    "no public source".to_string()
                } else {
                    font.url.clone()
                };
                r.warn(format!(
                    "{}: font {} missing (licence: {}; {}); it falls back to a generic font. Get it, then `darwan font import {} <file>`",
                    theme.id, font.family, font.license, source, theme.id
                ));
            }
        }
    }
    let nerd = Command::new("fc-list")
        .arg("JetBrainsMono Nerd Font")
        .output()
        .is_ok_and(|o| !o.stdout.is_empty());
    if !nerd {
        r.warn("JetBrainsMono Nerd Font is not installed: clockwork/neo-orbital icons will show as boxes (ttf-jetbrains-mono-nerd)");
    }

    println!("\n{}", style::heading("SDDM"));
    if !on_path("sddm-greeter-qt6") {
        r.warn("SDDM (Qt 6) is not installed; SDDM actions are unavailable");
    } else {
        r.ok("sddm-greeter-qt6 is installed");
        if Path::new(darwan_core::paths::HELPER).is_file()
            && Path::new("/usr/share/polkit-1/actions/org.darwan.policy").is_file()
        {
            r.ok("darwan-helper and its polkit policy are installed")
        } else {
            r.warn(format!(
                "{} or its polkit policy is not installed: `darwan sddm apply` won't work",
                darwan_core::paths::HELPER
            ))
        }
        let link = Path::new("/usr/share/sddm/themes/darwan");
        let broken = std::fs::read_link(link).is_ok_and(|t| !t.join("Main.qml").is_file());
        if let Ok(target) = std::fs::read_link(link)
            && broken
        {
            r.fail(format!(
                "{} points to {}, which no longer exists; run `darwan sddm reset` (or, without the helper: sudo rm {} /etc/sddm.conf.d/zz-darwan.conf)",
                link.display(),
                target.display(),
                link.display()
            ));
        }
        let eff = sddm::effective_theme();
        match &eff.current {
            Some((name, _)) if name == "darwan" => {
                if !broken {
                    r.ok("SDDM uses darwan")
                }
            }
            Some((name, file)) => {
                let ours = eff.files.iter().any(|f| f.ends_with("zz-darwan.conf"));
                if ours {
                    r.fail(format!("{} sets Current={name} after darwan's zz-darwan.conf and wins; remove that line", file.display()))
                } else {
                    r.ok(format!(
                        "SDDM uses {name} (from {}); darwan not applied",
                        file.display()
                    ))
                }
            }
            None => r.ok("SDDM uses its default theme; darwan not applied"),
        }
        if videos > 0 && !ffmpeg {
            r.warn("the SDDM greeter also needs qt6-multimedia-ffmpeg for video themes");
        }
    }

    println!();
    if r.failed == 0 {
        println!(
            "{}",
            style::ok("No problems that stop darwan from working.")
        );
        Ok(ExitCode::SUCCESS)
    } else {
        println!(
            "{}",
            style::fail(format!("{} problem(s) need fixing.", r.failed))
        );
        Ok(ExitCode::FAILURE)
    }
}

// None when hypridle doesn't lock with darwan: another locker's setup is not ours to judge.
fn hypridle_advice(conf: &str) -> Option<Result<String, String>> {
    let mut lock_cmd = None;
    let mut inhibit_sleep = None;
    for line in conf.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "lock_cmd" => lock_cmd = Some(value.trim().to_string()),
            "inhibit_sleep" => inhibit_sleep = value.trim().parse::<u8>().ok(),
            _ => {}
        }
    }
    if !lock_cmd?.contains("darwan lock") {
        return None;
    }
    // Mode 2, hypridle's default, waits for the lock only when the command names hyprlock.
    Some(match inhibit_sleep {
        Some(3) => Ok("hypridle locks with darwan and waits for the lock before sleep".into()),
        _ => Err("hypridle locks with darwan but may sleep before the lock is up: set inhibit_sleep = 3 in hypridle.conf".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hypridle_must_wait_for_darwans_lock_before_sleep() {
        let good = "general {\n    lock_cmd = darwan lock\n    inhibit_sleep = 3 # wait\n}\n";
        assert!(matches!(hypridle_advice(good), Some(Ok(_))));
        let default_mode = "general {\n    lock_cmd = darwan lock\n}\n";
        assert!(
            matches!(hypridle_advice(default_mode), Some(Err(_))),
            "mode 2 releases sleep at once for any locker but hyprlock"
        );
        let commented = "general {\n    lock_cmd = darwan lock\n    # inhibit_sleep = 3\n}\n";
        assert!(matches!(hypridle_advice(commented), Some(Err(_))));
        assert_eq!(
            hypridle_advice("general {\n    lock_cmd = hyprlock\n}\n"),
            None
        );
        assert_eq!(hypridle_advice("listener {\n    timeout = 300\n}\n"), None);
    }
}
