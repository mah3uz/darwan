use std::process::ExitCode;

use crate::paths::Paths;
use crate::session::{self, WaylandSession};
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
        .env("DARWAN_USER", session::user_name())
        .env("DARWAN_HOSTNAME", session::host_name())
        .env("DARWAN_SESSIONS", session::sessions_json());
    if !opts.pam {
        eprintln!("Mock login: the password is \"test\". Close the window to exit.");
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
    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
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
