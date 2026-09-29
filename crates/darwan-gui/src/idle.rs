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

const SAVER_TIMES: &[u32] = &[60, 120, 180, 300, 600, 900, 1800];
const SCREEN_TIMES: &[u32] = &[0, 300, 600, 900, 1800, 3600];
const SUSPEND_TIMES: &[u32] = &[0, 900, 1800, 3600, 7200];
const LOCK_TIMES: &[(&str, &str)] = &[
    ("0", "At once"),
    ("5", "After 5 seconds"),
    ("10", "After 10 seconds"),
    ("30", "After 30 seconds"),
    ("60", "After a minute"),
    ("300", "After 5 minutes"),
    ("never", "Never"),
];
// (value, label, one line under the buttons, the whole story on hover)
const QUALITIES: &[(&str, &str, &str, &str)] = &[
    (
        "auto",
        "Auto",
        "Picked for this machine: lighter on battery or integrated graphics.",
        "Chosen from this machine: smaller copies on integrated graphics, on battery, without a video decoder driver or with under 8 GB of memory; still frames in power-saver mode. Only the focused monitor plays video.",
    ),
    (
        "full",
        "Full",
        "As shipped, on every monitor · a 4K video theme: about 4% CPU and 1 GB.",
        "Videos as shipped, on every monitor. The smoothest, and the most GPU work, power and memory (a 4K video theme: about 4% CPU and 1 GB).",
    ),
    (
        "eco",
        "Eco",
        "Up to 1080p and 30 fps, focused monitor only · about a quarter of the work.",
        "Copies at up to 1080p and 30 fps, made once when you pick a theme or background: about a quarter of the decoding work, softer on large screens. Only the focused monitor plays video.",
    ),
    (
        "still",
        "Still",
        "The first frame of each video · almost no power.",
        "The first frame of each video: almost no GPU work or power, and no motion in video backgrounds. Other animations still play.",
    ),
];

// Short, for the timeline and the summary: "30 s", "5 min", "1 hour".
fn span(secs: u32) -> String {
    match secs {
        s if s % 3600 == 0 => format!("{} hour{}", s / 3600, if s == 3600 { "" } else { "s" }),
        s if s % 60 == 0 => format!("{} min", s / 60),
        s => format!("{s} s"),
    }
}

// Long, for menus: "1 minute", "10 minutes", "Never".
fn spelled(secs: u32) -> String {
    match secs {
        0 => "Never".into(),
        s if s % 3600 == 0 => format!("{} hour{}", s / 3600, if s == 3600 { "" } else { "s" }),
        60 => "1 minute".into(),
        s if s % 60 == 0 => format!("{} minutes", s / 60),
        s => format!("{s} seconds"),
    }
}

// A timeout set by hand that the list doesn't have still shows, as itself.
fn choices(list: &[u32], current: u32) -> Vec<Value> {
    let mut values = list.to_vec();
    if !values.contains(&current) {
        values.push(current);
        values.sort_unstable_by_key(|&v| if v == 0 { u32::MAX } else { v });
    }
    values
        .into_iter()
        .map(|v| json!({ "value": v.to_string(), "label": spelled(v), "on": v == current }))
        .collect()
}

fn problem(setup: &Setup, conf: Option<&Value>) -> &'static str {
    let needs_fix = |c: &Value| {
        c["resumed"] != true || (c["lockWithDarwan"] == true && c["waitsForLock"] != true)
    };
    match conf {
        _ if !setup.installed => "missing",
        _ if !setup.running => "stopped",
        None => "noconf",
        Some(c) if needs_fix(c) => "fix",
        Some(_) => "",
    }
}

fn setup_card(setup: &Setup, conf: Option<&Value>, problem: &str) -> Value {
    let action = |id: &str, label: &str, primary: bool| json!({ "id": id, "label": label, "primary": primary });
    match problem {
        "missing" => json!({
            "title": "hypridle isn’t installed",
            "text": "Hyprland’s idle daemon starts the screensaver when you step away. Install it, then check again.",
            "code": "sudo pacman -S hypridle",
            "actions": [action("check", "Check again", true)],
        }),
        "stopped" => {
            let (where_, code) = if setup.uwsm {
                (
                    "Your session runs under uwsm, so let systemd start it with every login:",
                    "systemctl --user enable --now hypridle.service",
                )
            } else if setup.lua {
                (
                    "To start it with Hyprland, add this to ~/.config/hypr/hyprland.lua:",
                    "hl.on(\"hyprland.start\", function() hl.exec_cmd(\"hypridle\") end)",
                )
            } else {
                (
                    "To start it with Hyprland, add this to ~/.config/hypr/hyprland.conf:",
                    "exec-once = hypridle",
                )
            };
            json!({
                "title": "hypridle isn’t running",
                "text": format!("Nothing starts the screensaver until it runs. {where_}"),
                "code": code,
                "actions": [action("start", "Start now", true), action("check", "Check again", false)],
            })
        }
        "noconf" => json!({
            "title": "No hypridle.conf yet",
            "text": "Enable writes one: the screensaver after 5 minutes, the screen off after 10, and Darwan locking before sleep.",
            "code": "",
            "actions": [action("enable", "Enable the screensaver", true)],
        }),
        "fix" => {
            let c = conf.expect("a config needs fixing only when there is one");
            let mut text = Vec::new();
            if c["resumed"] != true {
                text.push("Opening the lid without touching anything can bring the screensaver straight back.");
            }
            if c["lockWithDarwan"] == true && c["waitsForLock"] != true {
                text.push("The machine can go to sleep before Darwan’s lock is up.");
            }
            text.push("Fix keeps your other settings.");
            json!({
                "title": "hypridle.conf needs a fix",
                "text": text.join(" "),
                "code": "",
                "actions": [action("enable", "Fix", true)],
            })
        }
        _ => Value::Null,
    }
}

struct Marker {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    value: String,
    // Seconds after idle; None sorts to the end as "Never".
    at: Option<f64>,
    choices: Vec<Value>,
}

// Idle, then each thing that happens in the order it happens. The points are evenly spaced, not to scale:
// ten seconds next to ten minutes would be invisible.
fn timeline(conf: &Value, lock_after: &str) -> Value {
    let secs = |key: &str| conf[key].as_u64().map(|v| v as u32);
    let (saver, screen_off, suspend) = (secs("saver"), secs("screenOff"), secs("suspend"));
    let mut marks = Vec::new();
    if let Some(saver) = saver {
        marks.push(Marker {
            id: "saver",
            name: "Screensaver",
            icon: "moon",
            value: span(saver),
            at: Some(saver as f64),
            choices: choices(SAVER_TIMES, saver),
        });
        let lock: Option<u32> = lock_after.parse().ok();
        marks.push(Marker {
            id: "lock",
            name: "Locks",
            icon: "lock",
            value: match lock {
                Some(0) => "At once".into(),
                Some(s) => format!("+{}", span(s)),
                None => "Never".into(),
            },
            at: lock.map(|s| saver as f64 + s as f64 + 0.1),
            choices: LOCK_TIMES
                .iter()
                .map(|(v, l)| json!({ "value": v, "label": l, "on": *v == lock_after }))
                .collect(),
        });
    }
    // A screen that goes off with the screensaver sorts before it, so the clash shows.
    marks.push(Marker {
        id: "screenOff",
        name: "Screen off",
        icon: "display",
        value: screen_off.map_or("Never".into(), span),
        at: screen_off.map(|s| s as f64 - 0.2),
        choices: choices(SCREEN_TIMES, screen_off.unwrap_or(0)),
    });
    marks.push(Marker {
        id: "suspend",
        name: "Suspend",
        icon: "power",
        value: suspend.map_or("Never".into(), span),
        at: suspend.map(f64::from),
        choices: choices(SUSPEND_TIMES, suspend.unwrap_or(0)),
    });
    marks.sort_by(|a, b| {
        a.at.unwrap_or(f64::MAX)
            .total_cmp(&b.at.unwrap_or(f64::MAX))
    });
    let clash = matches!((saver, screen_off), (Some(s), Some(o)) if o <= s);
    let (first, last) = (0.2, 0.86);
    let step = if marks.len() > 1 {
        (last - first) / (marks.len() - 1) as f64
    } else {
        0.0
    };
    let place = |i: usize| first + step * i as f64;
    let position = |id: &str| marks.iter().position(|m| m.id == id).map(place);
    let open = match (position("saver"), position("lock")) {
        (Some(s), Some(l)) if l > s => json!({ "from": s, "to": l }),
        _ => Value::Null,
    };
    let (note, warn) = match saver {
        _ if clash => ("The screen turns off before the screensaver would start, so you’ll never see it.".to_string(), true),
        None => (format!("No screensaver: the screen just turns off{}.", screen_off.map_or(" never".into(), |s| format!(" after {}", span(s)))), false),
        Some(_) => match lock_after.parse::<u32>() {
            Err(_) => ("It never locks by itself: until you lock it, any key goes back to the desktop without a password.".into(), true),
            Ok(0) => ("It locks as soon as it starts.".into(), false),
            Ok(s) => (format!("For the first {} (the amber stretch), any key goes back to the desktop without a password.", span(s)), false),
        },
    };
    json!({
        "saverOn": saver.is_some(),
        "open": open,
        "note": note,
        "warn": warn,
        "markers": marks.iter().enumerate().map(|(i, m)| json!({
            "id": m.id,
            "name": m.name,
            "icon": m.icon,
            "value": m.value,
            "never": m.at.is_none(),
            "warn": clash && m.id == "saver",
            "x": place(i),
            "choices": m.choices,
        })).collect::<Vec<_>>(),
    })
}

fn summary(conf: Option<&Value>, lock_after: &str, problem: &str) -> String {
    let c = match (problem, conf) {
        ("missing", _) => return "The screensaver can’t start: hypridle isn’t installed".into(),
        ("stopped", _) => return "The screensaver can’t start: hypridle isn’t running".into(),
        (_, None) => return "The screensaver isn’t set up yet".into(),
        (_, Some(c)) => c,
    };
    let secs = |key: &str| c[key].as_u64().map(|v| v as u32);
    let mut parts = vec![match secs("saver") {
        Some(s) => format!(
            "screensaver after {}, {}",
            span(s),
            match lock_after.parse::<u32>() {
                Err(_) => "never locks by itself".into(),
                Ok(0) => "locks at once".into(),
                Ok(l) => format!("locks {} later", span(l)),
            }
        ),
        None => "no screensaver".into(),
    }];
    parts.push(secs("screenOff").map_or("screen stays on".into(), |s| {
        format!("screen off after {}", span(s))
    }));
    parts.push(secs("suspend").map_or("never suspends".into(), |s| {
        format!("suspends after {}", span(s))
    }));
    let text = parts.join(" · ");
    if problem == "fix" {
        format!("{text} · needs a fix")
    } else {
        text
    }
}

// The Screensaver panel, whole: its state, what to do about it, the timeline and darwan's own saver settings.
pub fn panel(setup: &Setup, conf: Option<&Hypridle>, config: &UserConfig) -> Value {
    let s = status(setup, conf, config);
    let c = Some(&s["config"]).filter(|c| !c.is_null());
    let lock_after = s["lockAfter"].as_str().unwrap_or("10").to_string();
    let problem = problem(setup, c);
    let home = std::env::var("HOME").unwrap_or_default();
    let path = s["path"].as_str().unwrap_or("");
    let quality = s["quality"].as_str().unwrap_or("full");
    let q = QUALITIES
        .iter()
        .find(|q| q.0 == quality)
        .unwrap_or(&QUALITIES[1]);
    json!({
        "ready": problem.is_empty(),
        "problem": problem,
        "running": setup.running,
        "summary": summary(c, &lock_after, problem),
        "warn": !problem.is_empty(),
        "setup": setup_card(setup, c, problem),
        "timeline": c.map_or(Value::Null, |c| timeline(c, &lock_after)),
        "sleep": c.map_or(Value::Null, |c| json!({
            "lockBeforeSleep": c["lockBeforeSleep"],
            "lockWithDarwan": c["lockWithDarwan"],
            "lockWithDarwanNote": match c["otherLocker"].as_str() {
                Some(other) => format!("Now: {other}. Turning this on replaces it."),
                None => "loginctl lock-session and power menus show your Darwan theme".into(),
            },
        })),
        "quality": quality,
        "qualities": QUALITIES.iter().map(|(v, l, _, long)| json!({ "value": v, "label": l, "long": long })).collect::<Vec<_>>(),
        "qualityLine": q.2,
        "qualityLong": q.3,
        "path": if !home.is_empty() && path.starts_with(&home) { format!("~{}", &path[home.len()..]) } else { path.to_string() },
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
    fn panel_for(setup: &Setup, text: Option<&str>, config: &str) -> Value {
        panel(
            setup,
            text.map(hypridle::parse).as_ref(),
            &UserConfig::parse(config).unwrap(),
        )
    }

    fn ids(p: &Value) -> Vec<&str> {
        p["timeline"]["markers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn the_panel_asks_for_one_setup_step_at_a_time() {
        let enabled = apply(None, "enable", "", true).unwrap();
        let missing = Setup {
            installed: false,
            running: false,
            ..setup()
        };
        assert_eq!(
            panel_for(&missing, Some(&enabled), "")["problem"],
            "missing",
            "installing comes before anything else"
        );
        let stopped = Setup {
            running: false,
            uwsm: true,
            ..setup()
        };
        let p = panel_for(&stopped, Some(&enabled), "");
        assert_eq!(p["problem"], "stopped");
        assert_eq!(
            p["setup"]["code"], "systemctl --user enable --now hypridle.service",
            "a uwsm session starts it through systemd"
        );
        assert!(
            !p["timeline"].is_null(),
            "a stopped hypridle still has settings to show"
        );
        let none = panel_for(&setup(), None, "");
        assert_eq!(none["problem"], "noconf");
        assert!(none["timeline"].is_null());
        let broken = panel_for(
            &setup(),
            Some("listener {\n    timeout = 300\n    on-timeout = darwan saver\n}\n"),
            "",
        );
        assert_eq!(broken["problem"], "fix");
        assert!(broken["summary"].as_str().unwrap().ends_with("needs a fix"));
        let ready = panel_for(&setup(), Some(&enabled), "");
        assert_eq!(
            (ready["ready"].as_bool(), ready["setup"].is_null()),
            (Some(true), true)
        );
    }

    #[test]
    fn the_timeline_orders_what_happens_and_shows_a_screen_that_goes_off_first() {
        let text = apply(None, "enable", "", true).unwrap();
        let p = panel_for(&setup(), Some(&text), "");
        assert_eq!(ids(&p), ["saver", "lock", "screenOff", "suspend"]);
        assert_eq!(
            p["summary"],
            "screensaver after 5 min, locks 10 s later · screen off after 10 min · never suspends"
        );
        assert_eq!(
            p["timeline"]["markers"][3]["never"], true,
            "an unset step sits at the end, as Never"
        );
        assert!(
            !p["timeline"]["open"].is_null(),
            "the no-password stretch runs from the saver to the lock"
        );

        let text = apply(Some(&text), "saver", "900", true).unwrap();
        let p = panel_for(&setup(), Some(&text), "");
        assert_eq!(ids(&p)[0], "screenOff");
        assert_eq!(p["timeline"]["markers"][1]["warn"], true);
        assert_eq!(p["timeline"]["warn"], true);

        let text = apply(Some(&text), "saver", "600", true).unwrap();
        assert_eq!(
            ids(&panel_for(&setup(), Some(&text), ""))[0],
            "screenOff",
            "a tie is a clash too: the screen is already off"
        );
    }

    #[test]
    fn a_saver_that_never_locks_says_what_that_costs() {
        let text = apply(None, "enable", "", true).unwrap();
        let p = panel_for(&setup(), Some(&text), "[saver]\nlock_after = \"never\"\n");
        assert_eq!(ids(&p), ["saver", "screenOff", "lock", "suspend"]);
        assert_eq!(p["timeline"]["warn"], true);
        assert!(
            p["summary"]
                .as_str()
                .unwrap()
                .contains("never locks by itself")
        );
        let p = panel_for(&setup(), Some(&text), "[saver]\nlock_after = \"0\"\n");
        assert_eq!(p["timeline"]["markers"][1]["value"], "At once");
    }

    #[test]
    fn a_timeout_set_by_hand_stays_a_choice() {
        let text = apply(None, "enable", "", true).unwrap();
        let text = apply(Some(&text), "saver", "420", true).unwrap();
        let p = panel_for(&setup(), Some(&text), "");
        let saver = &p["timeline"]["markers"][0];
        assert_eq!(saver["value"], "7 min");
        let on: Vec<&Value> = saver["choices"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["on"] == true)
            .collect();
        assert_eq!(on.len(), 1);
        assert_eq!(on[0]["label"], "7 minutes");
    }
}
