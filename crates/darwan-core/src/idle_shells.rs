use serde_json::Value;

use crate::wallpaper::{
    Env, is_caelestia, is_dms, is_noctalia, is_noctalia_legacy, is_omarchy_shell, processes,
};

// Idle timers a running shell or idle daemon keeps beside hypridle's. Each acts on the same idle as Darwan's
// screensaver, lock and screen-off, so any that is on is a collision to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Screensaver,
    Lock,
    ScreenOff,
    // Counted from the lock, as saver.screen_off_locked is.
    ScreenOffLocked,
    Suspend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    pub action: Action,
    pub secs: u32,
    // "on AC" or "on battery" when the shell keeps the two apart and they differ.
    pub power: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    pub name: &'static str,
    // Where the user turns it off.
    pub place: &'static str,
    pub timers: Vec<Timer>,
}

impl Shell {
    // "DMS also turns screens off at 20 min on AC. Turn off in Settings → Power & Sleep."
    pub fn describe(&self) -> String {
        let parts: Vec<String> = self
            .timers
            .iter()
            .map(|t| {
                let what = match t.action {
                    Action::Screensaver => format!("starts a screensaver at {}", span(t.secs)),
                    Action::Lock => format!("locks at {}", span(t.secs)),
                    Action::ScreenOff => format!("turns screens off at {}", span(t.secs)),
                    Action::ScreenOffLocked => {
                        format!("turns screens off {} after locking", span(t.secs))
                    }
                    Action::Suspend => format!("suspends at {}", span(t.secs)),
                };
                match t.power {
                    Some(p) => format!("{what} {p}"),
                    None => what,
                }
            })
            .collect();
        format!(
            "{} also {}. Turn off in {}.",
            self.name,
            parts.join(", "),
            self.place
        )
    }
}

// Short, for timelines and summaries: "30 s", "5 min", "1 hour".
pub fn span(secs: u32) -> String {
    match secs {
        s if s % 3600 == 0 => format!("{} hour{}", s / 3600, if s == 3600 { "" } else { "s" }),
        s if s % 60 == 0 => format!("{} min", s / 60),
        s => format!("{s} s"),
    }
}

pub fn find(env: &Env) -> Vec<Shell> {
    let procs = processes(env);
    let running = |is: fn(&crate::wallpaper::Proc) -> bool| procs.iter().any(is);
    let read = |path: std::path::PathBuf| std::fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    let mut add = |name, place, timers: Vec<Timer>| {
        if !timers.is_empty() {
            out.push(Shell {
                name,
                place,
                timers,
            });
        }
    };
    if running(is_dms) {
        let text = read(env.config_home.join("DankMaterialShell/settings.json"));
        add("DMS", "Settings → Power & Sleep", dms(&text));
    }
    if running(is_noctalia_legacy) {
        let text = read(env.config_home.join("noctalia/settings.json"));
        add("Noctalia", "Settings → Idle", noctalia_legacy(&text));
    }
    if running(is_noctalia) {
        let text = (env.run)("noctalia", &["config", "export", "full"]).unwrap_or_default();
        add("Noctalia", "Settings → Idle", noctalia(&text));
    }
    if running(is_caelestia) {
        let text = read(env.config_home.join("caelestia/shell.json"));
        add(
            "Caelestia",
            "~/.config/caelestia/shell.json, general.idle",
            caelestia(&text),
        );
    }
    if running(is_omarchy_shell)
        && !env
            .state_home
            .join("omarchy/indicators/stay-awake")
            .exists()
    {
        let text = read(env.config_home.join("omarchy/shell.json"));
        add(
            "Omarchy",
            "~/.config/omarchy/shell.json, idle",
            omarchy(&text),
        );
    }
    for p in procs.iter().filter(|p| p.exe_name() == "swayidle") {
        add("swayidle", "its command line", swayidle(&p.argv));
    }
    out
}

fn secs(v: Option<&Value>) -> Option<u32> {
    v.and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .filter(|&n| n > 0)
}

// settings.json holds only what differs from DMS's defaults, and every idle timeout defaults to 0, off.
fn dms(text: &str) -> Vec<Timer> {
    let v: Value = serde_json::from_str(text).unwrap_or_default();
    let mut out = Vec::new();
    for (key, action) in [
        ("LockTimeout", Action::Lock),
        ("MonitorTimeout", Action::ScreenOff),
        ("PostLockMonitorTimeout", Action::ScreenOffLocked),
        ("SuspendTimeout", Action::Suspend),
    ] {
        let ac = secs(v.get(format!("ac{key}")));
        let battery = secs(v.get(format!("battery{key}")));
        if ac == battery {
            out.extend(ac.map(|secs| Timer {
                action,
                secs,
                power: None,
            }));
        } else {
            out.extend(ac.map(|secs| Timer {
                action,
                secs,
                power: Some("on AC"),
            }));
            out.extend(battery.map(|secs| Timer {
                action,
                secs,
                power: Some("on battery"),
            }));
        }
    }
    out
}

// Noctalia 4, on Quickshell: nothing runs until idle.enabled.
fn noctalia_legacy(text: &str) -> Vec<Timer> {
    let v: Value = serde_json::from_str(text).unwrap_or_default();
    let idle = &v["idle"];
    if idle["enabled"] != true {
        return Vec::new();
    }
    let at = |key: &str, default: u32| match idle.get(key) {
        Some(v) => secs(Some(v)),
        None => Some(default),
    };
    [
        (Action::ScreenOff, at("screenOffTimeout", 600)),
        (Action::Lock, at("lockTimeout", 660)),
        (Action::Suspend, at("suspendTimeout", 1800)),
    ]
    .into_iter()
    .filter_map(|(action, secs)| {
        secs.map(|secs| Timer {
            action,
            secs,
            power: None,
        })
    })
    .collect()
}

// Noctalia 5: `noctalia config export full` prints the merged config; each [idle.behavior.<name>] is off unless enabled.
fn noctalia(text: &str) -> Vec<Timer> {
    let Ok(doc) = text.parse::<toml_edit::DocumentMut>() else {
        return Vec::new();
    };
    let Some(behaviors) = doc
        .get("idle")
        .and_then(|i| i.get("behavior"))
        .and_then(toml_edit::Item::as_table_like)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (_, b) in behaviors.iter() {
        if b.get("enabled").and_then(toml_edit::Item::as_bool) != Some(true) {
            continue;
        }
        let secs = |key: &str| {
            b.get(key)
                .and_then(toml_edit::Item::as_integer)
                .and_then(|n| u32::try_from(n).ok())
                .filter(|&n| n > 0)
        };
        let actions: &[Action] = match b.get("action").and_then(toml_edit::Item::as_str) {
            Some("lock") => &[Action::Lock],
            Some("screen_off") => &[Action::ScreenOff],
            Some("suspend") => &[Action::Suspend],
            Some("lock_and_suspend") => &[Action::Lock, Action::Suspend],
            _ => &[],
        };
        for &action in actions {
            out.extend(secs("timeout").map(|secs| Timer {
                action,
                secs,
                power: None,
            }));
        }
        if actions == [Action::ScreenOff] {
            out.extend(secs("locked_timeout").map(|secs| Timer {
                action: Action::ScreenOffLocked,
                secs,
                power: None,
            }));
        }
    }
    out
}

// Caelestia's timeouts are on out of the box: lock at 3 min, screens off at 5, suspend at 10.
fn caelestia(text: &str) -> Vec<Timer> {
    let v: Value = serde_json::from_str(text).unwrap_or_default();
    let defaults = serde_json::json!([
        { "timeout": 180, "idleAction": "lock" },
        { "timeout": 300, "idleAction": "dpms off" },
        { "timeout": 600, "idleAction": ["suspendThenHibernate"] },
    ]);
    let list = v["general"]["idle"]["timeouts"]
        .as_array()
        .unwrap_or_else(|| defaults.as_array().expect("a literal array"));
    list.iter()
        .filter(|t| t["enabled"] != false)
        .filter_map(|t| {
            let action = match &t["idleAction"] {
                Value::String(s) if s == "lock" => Action::Lock,
                Value::String(s) if s.to_lowercase().contains("dpms off") => Action::ScreenOff,
                Value::Array(a) => {
                    let words: String = a
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase();
                    if words.contains("suspend") || words.contains("hibernate") {
                        Action::Suspend
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };
            secs(t.get("timeout")).map(|secs| Timer {
                action,
                secs,
                power: None,
            })
        })
        .collect()
}

// Omarchy's own shell: its screensaver at 2½ minutes and its lock, which also turns the display off, at 5.
fn omarchy(text: &str) -> Vec<Timer> {
    let v: Value = serde_json::from_str(text).unwrap_or_default();
    let at = |key: &str, default: u32| match v["idle"].get(key) {
        Some(x) => secs(Some(x)),
        None => Some(default),
    };
    [
        (Action::Screensaver, at("screensaver", 150)),
        (Action::Lock, at("lock", 300)),
    ]
    .into_iter()
    .filter_map(|(action, secs)| {
        secs.map(|secs| Timer {
            action,
            secs,
            power: None,
        })
    })
    .collect()
}

// `swayidle timeout 300 'cmd' resume 'cmd' …`: each timeout by what its command does.
fn swayidle(argv: &[String]) -> Vec<Timer> {
    argv.windows(3)
        .filter(|w| w[0] == "timeout")
        .filter_map(|w| {
            let secs: u32 = w[1].parse().ok().filter(|&n| n > 0)?;
            let cmd = w[2].to_lowercase();
            let action = if cmd.contains("suspend") || cmd.contains("hibernate") {
                Action::Suspend
            } else if cmd.contains("lock") {
                Action::Lock
            } else if cmd.contains("dpms")
                || cmd.contains("power off")
                || cmd.contains("power-off-monitors")
            {
                Action::ScreenOff
            } else {
                return None;
            };
            Some(Timer {
                action,
                secs,
                power: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(action: Action, secs: u32, power: Option<&'static str>) -> Timer {
        Timer {
            action,
            secs,
            power,
        }
    }

    // Needs DMS running; reads a scratch settings file, never the real one. `cargo test -- --ignored running_dms`
    #[test]
    #[ignore]
    fn running_dms_is_found_with_its_timers() {
        let dir = std::env::temp_dir().join(format!("darwan-idle-shells-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("DankMaterialShell")).unwrap();
        std::fs::write(
            dir.join("DankMaterialShell/settings.json"),
            r#"{"acMonitorTimeout": 1200, "batteryMonitorTimeout": 1200}"#,
        )
        .unwrap();
        let env = Env {
            config_home: dir.clone(),
            ..Env::system()
        };
        let found = find(&env);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            found.iter().map(Shell::describe).collect::<Vec<_>>(),
            ["DMS also turns screens off at 20 min. Turn off in Settings → Power & Sleep."]
        );
    }

    #[test]
    fn dms_is_quiet_by_default_and_tells_ac_from_battery() {
        assert!(
            dms("{}").is_empty(),
            "every DMS idle timeout defaults to off"
        );
        assert_eq!(
            dms(r#"{"acMonitorTimeout": 1200, "batteryMonitorTimeout": 1200}"#),
            [t(Action::ScreenOff, 1200, None)],
            "the screen went off at 20 min though hypridle's was Never"
        );
        assert_eq!(
            dms(
                r#"{"acLockTimeout": 600, "batteryLockTimeout": 300, "acPostLockMonitorTimeout": 60, "acSuspendTimeout": 0}"#
            ),
            [
                t(Action::Lock, 600, Some("on AC")),
                t(Action::Lock, 300, Some("on battery")),
                t(Action::ScreenOffLocked, 60, Some("on AC")),
            ]
        );
    }

    #[test]
    fn noctalia_acts_only_once_idle_management_is_on() {
        assert!(noctalia_legacy(r#"{"idle": {"screenOffTimeout": 60}}"#).is_empty());
        assert_eq!(
            noctalia_legacy(r#"{"idle": {"enabled": true, "lockTimeout": 0}}"#),
            [
                t(Action::ScreenOff, 600, None),
                t(Action::Suspend, 1800, None)
            ],
            "defaults apply to what isn't written; 0 turns one off"
        );
        let v5 = "[idle.behavior.lock]\nenabled = true\ntimeout = 600\naction = \"lock\"\n\n\
                  [idle.behavior.screen-off]\nenabled = true\ntimeout = 0\nlocked_timeout = 30\naction = \"screen_off\"\n\n\
                  [idle.behavior.sleep]\nenabled = false\ntimeout = 900\naction = \"suspend\"\n";
        assert_eq!(
            noctalia(v5),
            [
                t(Action::Lock, 600, None),
                t(Action::ScreenOffLocked, 30, None)
            ]
        );
    }

    #[test]
    fn caelestia_acts_out_of_the_box() {
        assert_eq!(
            caelestia("{}"),
            [
                t(Action::Lock, 180, None),
                t(Action::ScreenOff, 300, None),
                t(Action::Suspend, 600, None),
            ]
        );
        let own = r#"{"general": {"idle": {"timeouts": [
            {"timeout": 300, "idleAction": "lock", "enabled": false},
            {"timeout": 900, "idleAction": ["systemctl", "suspend"]}
        ]}}}"#;
        assert_eq!(caelestia(own), [t(Action::Suspend, 900, None)]);
    }

    #[test]
    fn omarchy_and_swayidle_timers_are_read_by_what_they_do() {
        assert_eq!(
            omarchy("{}"),
            [
                t(Action::Screensaver, 150, None),
                t(Action::Lock, 300, None)
            ]
        );
        let argv: Vec<String> = [
            "swayidle",
            "-w",
            "timeout",
            "300",
            "swaylock -f",
            "timeout",
            "600",
            "swaymsg 'output * power off'",
            "resume",
            "swaymsg 'output * power on'",
            "timeout",
            "900",
            "notify-send hi",
        ]
        .map(String::from)
        .into();
        assert_eq!(
            swayidle(&argv),
            [t(Action::Lock, 300, None), t(Action::ScreenOff, 600, None)]
        );
    }
}
