use std::process::{Command, Stdio};

use darwan_core::config::UserConfig;
use darwan_core::hypridle::{self, Conf, Hypridle, Listener, Setup, runs_command};
use darwan_core::saver::{LockAfter, Quality};
use serde_json::{Value, json};

// Everything the Screensaver window shows: hypridle's state, its settings, and darwan's saver settings.
pub fn status(setup: &Setup, conf: Option<&Hypridle>, config: &UserConfig) -> Value {
    let lock_after = config
        .saver_lock_after()
        .ok()
        .flatten()
        .unwrap_or(LockAfter::DEFAULT);
    let quality = config
        .saver_quality()
        .ok()
        .flatten()
        .unwrap_or(Quality::DEFAULT);
    let h = conf.map(|h| {
        let lock = h.lock_cmd.as_deref();
        json!({
            "saver": h.saver_timeout,
            "screenOff": h.screen_off_timeout,
            "suspend": h.suspend_timeout,
            "lockWithDarwan": lock.is_some_and(|c| runs_command(c, "lock")),
            // Another locker in lock_cmd is the user's to change, not a toggle's to overwrite silently.
            "otherLocker": lock.filter(|c| !runs_command(c, "lock")),
            "lockBeforeSleep": h.before_sleep_cmd.as_deref().is_some_and(|c| c.contains("lock-session") || runs_command(c, "lock")),
            "resumed": h.after_sleep_cmd.as_deref().is_some_and(|c| runs_command(c, "resumed")),
            "waitsForLock": h.inhibit_sleep == Some(3),
        })
    });
    json!({
        "installed": setup.installed,
        "running": setup.running,
        "serviceEnabled": setup.service_enabled,
        "uwsm": setup.uwsm,
        "lua": setup.lua,
        "path": hypridle::config_path().display().to_string(),
        "config": h,
        "lockAfter": lock_after.to_string(),
        "quality": quality.as_str(),
    })
}

// A change from the window applied to hypridle.conf's text; unknown actions are refused.
pub fn apply(text: Option<&str>, action: &str, value: &str, lua: bool) -> Result<String, String> {
    let mut conf = Conf::parse(text.unwrap_or(""));
    let secs = || -> Result<Option<u32>, String> {
        match value {
            "" | "never" | "0" => Ok(None),
            v => v
                .parse()
                .map(Some)
                .map_err(|_| format!("{v:?} is not a number of seconds")),
        }
    };
    let on = || value == "true";
    match action {
        "enable" => conf.enable(lua),
        "disable" => conf.set_listener(Listener::Saver, None, lua),
        "saver" => conf.set_listener(
            Listener::Saver,
            secs()?.or(Some(hypridle::DEFAULT_SAVER)),
            lua,
        ),
        "screenOff" => conf.set_listener(Listener::ScreenOff, secs()?, lua),
        "suspend" => conf.set_listener(Listener::Suspend, secs()?, lua),
        "lockWithDarwan" => conf.set_lock_with_darwan(on()),
        "lockBeforeSleep" => conf.set_lock_before_sleep(on()),
        other => return Err(format!("unknown hypridle setting {other:?}")),
    }
    Ok(conf.to_string())
}

// hypridle reads its config once, so a change needs a restart: through systemd when it runs there, else a new process.
pub fn restart() -> Result<(), String> {
    let quiet = |program: &str, args: &[&str]| {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    if quiet(
        "systemctl",
        &["--user", "is-active", "--quiet", "hypridle.service"],
    ) {
        return quiet("systemctl", &["--user", "restart", "hypridle.service"])
            .then_some(())
            .ok_or_else(|| "systemctl --user restart hypridle.service failed".into());
    }
    quiet("pkill", &["-x", "hypridle"]);
    start()
}

// Detached from the GUI, so hypridle keeps running after the window closes.
pub fn start() -> Result<(), String> {
    Command::new("setsid")
        .args(["-f", "hypridle"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("cannot start hypridle: {e}"))
        .and_then(|s| {
            s.success()
                .then_some(())
                .ok_or_else(|| "hypridle did not start".into())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Setup {
        Setup {
            installed: true,
            running: true,
            service_enabled: false,
            uwsm: false,
            lua: true,
        }
    }

    #[test]
    fn no_config_shows_null_and_the_saver_defaults() {
        let s = status(&setup(), None, &UserConfig::default());
        assert!(s["config"].is_null(), "the window then offers to enable");
        assert_eq!(s["lockAfter"], "10");
        assert_eq!(s["quality"], "full");
    }

    #[test]
    fn enabling_then_changing_timeouts_round_trips_through_the_status() {
        let text = apply(None, "enable", "", true).unwrap();
        let text = apply(Some(&text), "saver", "120", true).unwrap();
        let text = apply(Some(&text), "suspend", "1800", true).unwrap();
        let text = apply(Some(&text), "screenOff", "never", true).unwrap();
        let s = status(
            &setup(),
            Some(&hypridle::parse(&text)),
            &UserConfig::default(),
        );
        let c = &s["config"];
        assert_eq!(c["saver"], 120);
        assert_eq!(c["suspend"], 1800);
        assert!(c["screenOff"].is_null());
        assert_eq!(c["lockWithDarwan"], true);
        assert_eq!(c["resumed"], true);
        assert_eq!(c["waitsForLock"], true);
    }

    #[test]
    fn another_locker_is_reported_not_overwritten() {
        let text = apply(
            Some("general {\n    lock_cmd = hyprlock\n}\n"),
            "enable",
            "",
            false,
        )
        .unwrap();
        let s = status(
            &setup(),
            Some(&hypridle::parse(&text)),
            &UserConfig::default(),
        );
        assert_eq!(s["config"]["otherLocker"], "hyprlock");
        assert_eq!(s["config"]["lockWithDarwan"], false);
        assert!(apply(None, "bogus", "", true).is_err());
        assert!(apply(None, "saver", "soon", true).is_err());
    }
}
