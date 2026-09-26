use std::process::ExitCode;

use darwan_core::host;
use darwan_core::paths::Paths;

use crate::session::WaylandSession;
use crate::{overlay, qs};

pub struct Options {
    pub id: Option<String>,
    pub sddm: bool,
    pub pam: bool,
    pub at: Option<String>,
}

pub fn run(paths: &Paths, opts: Options) -> Result<ExitCode, String> {
    let wayland = WaylandSession::discover()?;
    let prepared = overlay::prepare(paths, opts.id.as_deref(), "preview.conf")?;

    let wrapper = match &opts.at {
        Some(at) => vec!["faketime".to_string(), format!("today {}", check_time(at)?)],
        None => Vec::new(),
    };
    let mut cmd = qs::command(paths, "preview_shell.qml", &wrapper);
    qs::theme_env(
        &mut cmd,
        &prepared.theme,
        &prepared.theme.dir,
        Some(&prepared.overlay),
    );
    wayland.apply(&mut cmd);
    cmd.env("DARWAN_MODE", if opts.sddm { "sddm" } else { "lock" })
        .env("DARWAN_AUTH", if opts.pam { "pam" } else { "mock" })
        .env("DARWAN_USER", host::user_name())
        .env("DARWAN_HOSTNAME", host::host_name())
        .env("DARWAN_SESSIONS", host::sessions_json());
    let log_path = darwan_core::paths::state_dir().join("preview.log");
    let log =
        std::fs::File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    cmd.stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    if opts.pam {
        println!("Unlock with your password, or press Ctrl+Q to close.");
    } else {
        println!("Unlock with the password \"test\", or press Ctrl+Q to close.");
    }
    let status = cmd.status().map_err(|e| {
        format!(
            "cannot start {}: {e}",
            if wrapper.is_empty() {
                "quickshell"
            } else {
                "faketime"
            }
        )
    })?;
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
