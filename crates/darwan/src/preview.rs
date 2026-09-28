use std::path::PathBuf;
use std::process::{Command, ExitCode};

use darwan_core::host;
use darwan_core::paths::Paths;

use crate::session::WaylandSession;
use crate::style;
use crate::{overlay, qs};

pub struct Options {
    pub id: Option<String>,
    pub sddm: bool,
    pub pam: bool,
    pub at: Option<String>,
    pub shot: Option<PathBuf>,
    pub saver: bool,
}

pub fn run(paths: &Paths, opts: Options) -> Result<ExitCode, String> {
    if opts.pam && opts.at.is_some() {
        return Err("--at works only with the mock login (\"test\"), not --pam".into());
    }
    if opts.saver && (opts.sddm || opts.at.is_some()) {
        return Err(
            "--saver shows the lock's screensaver; it can't be combined with --sddm or --at".into(),
        );
    }
    if opts.shot.is_some() && opts.at.is_some() {
        return Err("--shot can't be combined with --at".into());
    }
    // Relative to where the user ran darwan, not to runtime/ where the host runs.
    let shot = match &opts.shot {
        Some(p) if !p.extension().is_some_and(|x| x == "png") => {
            return Err(format!(
                "--shot saves a PNG; {} should end in .png",
                p.display()
            ));
        }
        Some(p) => Some(std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()))?),
        None => None,
    };
    let wayland = WaylandSession::discover()?;
    let prepared = overlay::prepare(paths, opts.id.as_deref(), "preview.conf")?;
    let mode = if opts.sddm { "sddm" } else { "lock" };

    let (mut cmd, program) = match &opts.at {
        None => {
            let mut cmd = qs::command(paths, "preview_shell.qml", &[]);
            qs::theme_env(
                &mut cmd,
                &prepared.theme,
                &prepared.theme.dir,
                Some(&prepared.overlay),
            );
            cmd.env("DARWAN_PREVIEW_SAVER", if opts.saver { "1" } else { "0" })
                .env("DARWAN_MODE", mode)
                .env("DARWAN_AUTH", if opts.pam { "pam" } else { "mock" })
                .env("DARWAN_USER", host::user_name())
                .env("DARWAN_HOSTNAME", host::host_name())
                .env("DARWAN_SESSIONS", host::sessions_json());
            if let Some(shot) = &shot {
                cmd.env("DARWAN_SHOT", shot);
            }
            (cmd, "quickshell")
        }
        // TODO: back to Quickshell once it starts under libfaketime (its jemalloc deadlocks there).
        Some(at) => {
            let settings = serde_json::json!({
                "themeId": prepared.theme.id,
                "themePath": prepared.theme.dir,
                "overlay": prepared.overlay,
                "mode": mode,
                "user": host::user_name(),
                "hostName": host::host_name(),
                "sessions": serde_json::from_str::<serde_json::Value>(&host::sessions_json())
                    .unwrap_or_default(),
            });
            let mut cmd = Command::new("faketime");
            cmd.arg(format!("today {}", check_time(at)?))
                .arg("qml6")
                .arg(paths.runtime().join("qml_preview.qml"))
                .arg("--")
                .arg(settings.to_string());
            qs::runtime_env(&mut cmd, paths);
            // Without this, a qml runner whose stderr is not a terminal logs to the journal.
            cmd.env("QT_FORCE_STDERR_LOGGING", "1");
            (cmd, "faketime")
        }
    };
    wayland.apply(&mut cmd);
    let log_path = darwan_core::paths::state_dir().join("preview.log");
    let log =
        std::fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    cmd.stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    if let Some(shot) = &shot {
        println!("Saving {} once the theme has settled.", shot.display());
    } else if opts.pam {
        println!(
            "Unlock with your password, or press {} to close.",
            style::bold("Ctrl+Q")
        );
    } else {
        println!(
            "Unlock with the password {}, or press {} to close.",
            style::value("test"),
            style::bold("Ctrl+Q")
        );
    }
    let status = cmd
        .status()
        .map_err(|e| format!("cannot start {program}: {e}"))?;
    if status.success() {
        return Ok(ExitCode::SUCCESS);
    }
    let log = std::fs::read_to_string(&log_path).unwrap_or_default();
    let tail: Vec<&str> = log.lines().rev().take(12).collect();
    eprintln!(
        "The preview failed ({status}); end of {}:\n{}",
        log_path.display(),
        tail.into_iter().rev().collect::<Vec<_>>().join("\n")
    );
    Ok(ExitCode::FAILURE)
}

fn check_time(at: &str) -> Result<&str, String> {
    let ok = at.split_once(':').is_some_and(|(h, m)| {
        h.len() == 2
            && m.len() == 2
            && h.parse::<u8>().is_ok_and(|h| h < 24)
            && m.parse::<u8>().is_ok_and(|m| m < 60)
    });
    if ok {
        Ok(at)
    } else {
        Err(format!("--at expects HH:MM, got {at:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_accepts_only_valid_24h_times() {
        assert_eq!(check_time("00:00"), Ok("00:00"));
        assert_eq!(check_time("23:59"), Ok("23:59"));
        assert!(check_time("24:00").is_err());
        assert!(check_time("7:30").is_err());
        assert!(check_time("12:00; rm -rf").is_err());
    }
}
