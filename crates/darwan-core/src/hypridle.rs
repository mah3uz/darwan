use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// The parts of hypridle.conf darwan relies on.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Hypridle {
    pub lock_cmd: Option<String>,
    pub before_sleep_cmd: Option<String>,
    pub after_sleep_cmd: Option<String>,
    pub inhibit_sleep: Option<u32>,
    // The timeout of the listener that starts `darwan saver`.
    pub saver_timeout: Option<u32>,
    pub screen_off_timeout: Option<u32>,
    pub suspend_timeout: Option<u32>,
}

pub fn parse(text: &str) -> Hypridle {
    let conf = Conf::parse(text);
    let general = |key: &str| conf.value("general", 0, key).map(str::to_string);
    let timeout = |kind: Listener| {
        conf.listener(kind)
            .and_then(|b| conf.value_in(&b, "timeout"))
            .and_then(|v| v.parse().ok())
    };
    Hypridle {
        lock_cmd: general("lock_cmd"),
        before_sleep_cmd: general("before_sleep_cmd"),
        after_sleep_cmd: general("after_sleep_cmd"),
        inhibit_sleep: general("inhibit_sleep").and_then(|v| v.parse().ok()),
        saver_timeout: timeout(Listener::Saver),
        screen_off_timeout: timeout(Listener::ScreenOff),
        suspend_timeout: timeout(Listener::Suspend),
    }
}

// Whether a hypridle command runs `darwan <sub>`, alone or in a shell line.
pub fn runs_command(cmd: &str, sub: &str) -> bool {
    let words: Vec<&str> = cmd
        .split(|c: char| c.is_whitespace() || matches!(c, ';' | '&' | '|'))
        .filter(|w| !w.is_empty())
        .collect();
    words
        .windows(2)
        .any(|w| w[0].rsplit('/').next() == Some("darwan") && w[1] == sub)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Listener {
    Saver,
    ScreenOff,
    Suspend,
}

impl Listener {
    fn matches(self, on_timeout: &str) -> bool {
        let c = on_timeout.to_ascii_lowercase();
        match self {
            Listener::Saver => runs_command(on_timeout, "saver"),
            Listener::ScreenOff => {
                c.contains("dpms") && (c.contains("off") || c.contains("standby"))
            }
            Listener::Suspend => c.contains("suspend") || c.contains("hibernate"),
        }
    }
}

// Hyprland's dispatch syntax: a Lua config takes Lua dispatchers, a classic one the old words.
pub fn dpms(lua: bool, on: bool) -> String {
    match (lua, on) {
        (true, true) => r#"hyprctl dispatch 'hl.dsp.dpms({ action = "on" })'"#.into(),
        (true, false) => r#"hyprctl dispatch 'hl.dsp.dpms({ action = "off" })'"#.into(),
        (false, true) => "hyprctl dispatch dpms on".into(),
        (false, false) => "hyprctl dispatch dpms off".into(),
    }
}

// `darwan resumed` keeps a lid opened without input from bringing the saver back; turning the outputs on saves
// pressing a key twice after wake.
pub fn after_sleep(lua: bool) -> String {
    format!("darwan resumed; {}", dpms(lua, true))
}

pub const LOCK_CMD: &str = "darwan lock";
pub const BEFORE_SLEEP_CMD: &str = "loginctl lock-session";
pub const DEFAULT_SAVER: u32 = 300;
pub const DEFAULT_SCREEN_OFF: u32 = 600;

// What "enable" writes when there is no hypridle.conf yet.
pub fn minimal(lua: bool) -> String {
    let mut c = Conf::parse("");
    c.enable(lua);
    c.to_string()
}

#[derive(Debug, Clone, Copy)]
struct Block {
    start: usize,
    end: usize,
}

// hypridle.conf as lines, so edits keep the user's comments and everything darwan doesn't manage.
#[derive(Debug, Clone, Default)]
pub struct Conf {
    lines: Vec<String>,
}

impl Conf {
    pub fn parse(text: &str) -> Self {
        Self {
            lines: text.lines().map(str::to_string).collect(),
        }
    }

    fn code(line: &str) -> &str {
        line.split_once('#').map_or(line, |(l, _)| l).trim()
    }

    fn blocks(&self, name: &str) -> Vec<Block> {
        let mut out = Vec::new();
        let mut open: Option<usize> = None;
        for (i, line) in self.lines.iter().enumerate() {
            let code = Self::code(line);
            if let Some(n) = code.strip_suffix('{') {
                open = (n.trim() == name).then_some(i);
            } else if code == "}"
                && let Some(start) = open.take()
            {
                out.push(Block { start, end: i });
            }
        }
        out
    }

    fn key_line(&self, b: &Block, key: &str) -> Option<usize> {
        (b.start + 1..b.end).find(|&i| {
            Self::code(&self.lines[i])
                .split_once('=')
                .is_some_and(|(k, _)| k.trim() == key)
        })
    }

    fn value_in(&self, b: &Block, key: &str) -> Option<&str> {
        let i = self.key_line(b, key)?;
        Self::code(&self.lines[i])
            .split_once('=')
            .map(|(_, v)| v.trim())
    }

    fn value(&self, block: &str, nth: usize, key: &str) -> Option<&str> {
        let b = *self.blocks(block).get(nth)?;
        self.value_in(&b, key)
    }

    fn listener(&self, kind: Listener) -> Option<Block> {
        self.blocks("listener").into_iter().find(|b| {
            self.value_in(b, "on-timeout")
                .is_some_and(|c| kind.matches(c))
        })
    }

    fn general(&mut self) -> Block {
        if let Some(b) = self.blocks("general").first() {
            return *b;
        }
        let tail = if self.lines.is_empty() {
            vec![]
        } else {
            vec![String::new()]
        };
        let mut head = vec!["general {".to_string(), "}".to_string()];
        head.extend(tail);
        self.lines.splice(0..0, head);
        Block { start: 0, end: 1 }
    }

    pub fn set_general(&mut self, key: &str, value: Option<&str>) {
        let b = self.general();
        match (self.key_line(&b, key), value) {
            (Some(i), Some(v)) => self.lines[i] = format!("    {key} = {v}"),
            (Some(i), None) => {
                self.lines.remove(i);
            }
            (None, Some(v)) => self.lines.insert(b.end, format!("    {key} = {v}")),
            (None, None) => {}
        }
    }

    // None removes the listener; a timeout updates it, or adds it at the end.
    pub fn set_listener(&mut self, kind: Listener, timeout: Option<u32>, lua: bool) {
        match (self.listener(kind), timeout) {
            (Some(b), None) => {
                let mut end = b.end + 1;
                if self.lines.get(end).is_some_and(|l| l.trim().is_empty()) {
                    end += 1;
                }
                self.lines.drain(b.start..end);
            }
            (Some(b), Some(t)) => match self.key_line(&b, "timeout") {
                Some(i) => self.lines[i] = format!("    timeout = {t}"),
                None => self.lines.insert(b.start + 1, format!("    timeout = {t}")),
            },
            (None, Some(t)) => {
                if self.lines.last().is_some_and(|l| !l.trim().is_empty()) {
                    self.lines.push(String::new());
                }
                self.lines.push("listener {".into());
                self.lines.push(format!("    timeout = {t}"));
                match kind {
                    Listener::Saver => self.lines.push("    on-timeout = darwan saver".into()),
                    Listener::ScreenOff => {
                        self.lines
                            .push(format!("    on-timeout = {}", dpms(lua, false)));
                        self.lines
                            .push(format!("    on-resume = {}", dpms(lua, true)));
                    }
                    Listener::Suspend => {
                        self.lines.push("    on-timeout = systemctl suspend".into())
                    }
                }
                self.lines.push("}".into());
            }
            (None, None) => {}
        }
    }

    // The screensaver's setup: darwan locks (also before sleep, waiting for the lock), `darwan resumed` after sleep,
    // the saver listener, and the screen turning off later. What the user already set is kept.
    pub fn enable(&mut self, lua: bool) {
        let h = parse(&self.to_string());
        if h.lock_cmd.is_none() {
            self.set_general("lock_cmd", Some(LOCK_CMD));
        }
        if h.before_sleep_cmd.is_none() {
            self.set_general("before_sleep_cmd", Some(BEFORE_SLEEP_CMD));
        }
        match &h.after_sleep_cmd {
            None => self.set_general("after_sleep_cmd", Some(&after_sleep(lua))),
            Some(c) if !runs_command(c, "resumed") => {
                self.set_general("after_sleep_cmd", Some(&format!("darwan resumed; {c}")));
            }
            Some(_) => {}
        }
        if h.lock_cmd
            .as_deref()
            .is_none_or(|c| runs_command(c, "lock"))
        {
            self.set_general("inhibit_sleep", Some("3"));
        }
        if h.saver_timeout.is_none() {
            self.set_listener(Listener::Saver, Some(DEFAULT_SAVER), lua);
        }
        if h.screen_off_timeout.is_none() {
            self.set_listener(Listener::ScreenOff, Some(DEFAULT_SCREEN_OFF), lua);
        }
    }

    pub fn set_lock_with_darwan(&mut self, on: bool) {
        if on {
            self.set_general("lock_cmd", Some(LOCK_CMD));
            self.set_general("inhibit_sleep", Some("3"));
        } else if self
            .value("general", 0, "lock_cmd")
            .is_some_and(|c| runs_command(c, "lock"))
        {
            self.set_general("lock_cmd", None);
        }
    }

    pub fn set_lock_before_sleep(&mut self, on: bool) {
        let ours = self
            .value("general", 0, "before_sleep_cmd")
            .is_some_and(|c| c.contains("lock-session") || runs_command(c, "lock"));
        if on && !ours {
            self.set_general("before_sleep_cmd", Some(BEFORE_SLEEP_CMD));
        } else if !on && ours {
            self.set_general("before_sleep_cmd", None);
        }
    }
}

impl std::fmt::Display for Conf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

fn config_home() -> PathBuf {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"),
    }
}

pub fn config_path() -> PathBuf {
    config_home().join("hypr/hypridle.conf")
}

pub fn read() -> Option<Hypridle> {
    std::fs::read_to_string(config_path())
        .ok()
        .map(|t| parse(&t))
}

// How hypridle is set up on this machine, for the GUI and doctor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    pub installed: bool,
    pub running: bool,
    // hypridle.service is enabled for the user (it starts with the session under uwsm).
    pub service_enabled: bool,
    pub uwsm: bool,
    // Hyprland reads ~/.config/hypr/hyprland.lua, so dispatchers use the Lua syntax.
    pub lua: bool,
    // Running under Hyprland, the one compositor whose screens darwan can turn off yet.
    pub hyprland: bool,
    // Shells and idle daemons running their own idle timers beside hypridle's.
    pub shells: Vec<crate::idle_shells::Shell>,
}

pub fn setup() -> Setup {
    let quiet = |program: &str, args: &[&str]| {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    let on_path = std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("hypridle").is_file()));
    Setup {
        installed: on_path,
        running: quiet("pgrep", &["-x", "hypridle"]),
        service_enabled: quiet(
            "systemctl",
            &["--user", "is-enabled", "--quiet", "hypridle.service"],
        ),
        uwsm: std::env::var_os("UWSM_WAIT_VARNAMES").is_some()
            || quiet(
                "systemctl",
                &["--user", "is-active", "--quiet", "wayland-wm@*.service"],
            ),
        lua: hyprland_lua(),
        hyprland: std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some(),
        shells: crate::idle_shells::find(&crate::wallpaper::Env::system()),
    }
}

// Hyprland reads ~/.config/hypr/hyprland.lua when it exists.
pub fn hyprland_lua() -> bool {
    Path::new(&config_home())
        .join("hypr/hyprland.lua")
        .is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: &str = r#"# mine
general {
    lock_cmd = darwan lock          # lock
    before_sleep_cmd = loginctl lock-session
    after_sleep_cmd = darwan resumed; hyprctl dispatch dpms on
    inhibit_sleep = 3
}

listener {
    timeout = 150
    on-timeout = brightnessctl -s set 10
}

listener {
    timeout = 300
    on-timeout = /usr/bin/darwan saver
}

listener {
    timeout = 330
    on-timeout = hyprctl dispatch 'hl.dsp.dpms({ action = "off" })'
    on-resume = hyprctl dispatch 'hl.dsp.dpms({ action = "on" })'
}
"#;

    #[test]
    fn the_saver_screen_off_and_general_commands_are_read_from_any_layout() {
        let h = parse(USER);
        assert_eq!(h.lock_cmd.as_deref(), Some("darwan lock"));
        assert_eq!(h.inhibit_sleep, Some(3));
        assert_eq!(
            h.saver_timeout,
            Some(300),
            "the brightness listener is not the saver's"
        );
        assert_eq!(h.screen_off_timeout, Some(330));
        assert_eq!(h.suspend_timeout, None);
        assert!(runs_command(
            h.after_sleep_cmd.as_deref().unwrap(),
            "resumed"
        ));
        assert!(!runs_command("darwan-gui saver", "saver"));
        assert!(!runs_command("echo darwan", "saver"));
    }

    #[test]
    fn edits_keep_the_users_own_lines_and_comments() {
        let mut c = Conf::parse(USER);
        c.set_listener(Listener::Saver, Some(120), true);
        c.set_listener(Listener::Suspend, Some(1800), true);
        c.set_listener(Listener::ScreenOff, None, true);
        let text = c.to_string();
        assert!(text.contains("# mine") && text.contains("# lock"));
        assert!(
            text.contains("brightnessctl -s set 10"),
            "a listener darwan doesn't manage stays"
        );
        let h = parse(&text);
        assert_eq!(h.saver_timeout, Some(120));
        assert_eq!(h.suspend_timeout, Some(1800));
        assert_eq!(h.screen_off_timeout, None);
        assert!(
            !text.contains("dpms({ action = \"off\" })"),
            "removed with its on-resume"
        );
    }

    #[test]
    fn enabling_from_nothing_writes_a_complete_setup_that_waits_for_the_lock() {
        let h = parse(&minimal(true));
        assert_eq!(h.lock_cmd.as_deref(), Some(LOCK_CMD));
        assert_eq!(h.before_sleep_cmd.as_deref(), Some(BEFORE_SLEEP_CMD));
        assert_eq!(
            h.inhibit_sleep,
            Some(3),
            "hypridle's default mode doesn't wait for darwan's lock before sleep"
        );
        assert_eq!(h.saver_timeout, Some(DEFAULT_SAVER));
        assert!(h.screen_off_timeout.unwrap() > h.saver_timeout.unwrap());
        let after = h.after_sleep_cmd.unwrap();
        assert!(runs_command(&after, "resumed"));
        assert!(
            after.contains(r#"action = "on""#),
            "a Lua config takes Lua dispatchers"
        );
        assert!(minimal(false).contains("hyprctl dispatch dpms off"));
    }

    #[test]
    fn enabling_keeps_another_lockers_commands_and_adds_resumed_to_the_users_wake_command() {
        let mut c = Conf::parse(
            "general {\n    lock_cmd = hyprlock\n    after_sleep_cmd = hyprctl dispatch dpms on\n}\n",
        );
        c.enable(false);
        let h = parse(&c.to_string());
        assert_eq!(h.lock_cmd.as_deref(), Some("hyprlock"));
        assert_eq!(
            h.inhibit_sleep, None,
            "mode 3 is only needed for darwan's lock"
        );
        assert_eq!(
            h.after_sleep_cmd.as_deref(),
            Some("darwan resumed; hyprctl dispatch dpms on")
        );
        assert_eq!(h.saver_timeout, Some(DEFAULT_SAVER));
    }

    #[test]
    fn lock_toggles_touch_only_darwans_commands() {
        let mut c = Conf::parse(USER);
        c.set_lock_with_darwan(false);
        c.set_lock_before_sleep(false);
        let h = parse(&c.to_string());
        assert_eq!(h.lock_cmd, None);
        assert_eq!(h.before_sleep_cmd, None);
        let mut other = Conf::parse("general {\n    lock_cmd = hyprlock\n}\n");
        other.set_lock_with_darwan(false);
        assert_eq!(
            parse(&other.to_string()).lock_cmd.as_deref(),
            Some("hyprlock")
        );
    }
}
