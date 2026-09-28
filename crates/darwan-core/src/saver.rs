use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockAfter {
    Never,
    Secs(u32),
}

impl LockAfter {
    // Locking as the saver appears is the safe default; a grace period or no lock is the user's choice.
    pub const DEFAULT: Self = LockAfter::Secs(0);

    // What lock_shell.qml reads from DARWAN_LOCK_AFTER: milliseconds, negative for never.
    pub fn millis(self) -> i64 {
        match self {
            LockAfter::Never => -1,
            LockAfter::Secs(s) => i64::from(s) * 1000,
        }
    }
}

impl FromStr for LockAfter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "never" => Ok(LockAfter::Never),
            _ => s.parse().map(LockAfter::Secs).map_err(|_| {
                format!("saver.lock_after is a number of seconds or \"never\", not {s:?}")
            }),
        }
    }
}

impl fmt::Display for LockAfter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockAfter::Never => f.write_str("never"),
            LockAfter::Secs(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Auto,
    Full,
    Eco,
    Still,
}

impl Quality {
    pub const ALL: [Quality; 4] = [Quality::Auto, Quality::Full, Quality::Eco, Quality::Still];

    pub fn as_str(self) -> &'static str {
        match self {
            Quality::Auto => "auto",
            Quality::Full => "full",
            Quality::Eco => "eco",
            Quality::Still => "still",
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Quality::Auto => "chosen from the hardware and power source",
            Quality::Full => "videos as shipped",
            Quality::Eco => "videos at up to 1080p and 30 fps",
            Quality::Still => "the first frame of each video",
        }
    }
}

impl FromStr for Quality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        Quality::ALL
            .into_iter()
            .find(|q| q.as_str() == s)
            .ok_or_else(|| format!("saver.quality is auto, full, eco or still, not {s:?}"))
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// The parts of hypridle.conf darwan relies on.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Hypridle {
    pub lock_cmd: Option<String>,
    pub before_sleep_cmd: Option<String>,
    pub after_sleep_cmd: Option<String>,
    pub inhibit_sleep: Option<u32>,
    // The timeout of the listener that starts `darwan saver`.
    pub saver_timeout: Option<u32>,
}

pub fn parse_hypridle(text: &str) -> Hypridle {
    let mut out = Hypridle::default();
    let mut block = String::new();
    let mut timeout: Option<u32> = None;
    let mut runs_saver = false;
    for raw in text.lines() {
        let line = raw.split_once('#').map_or(raw, |(l, _)| l).trim();
        if let Some(name) = line.strip_suffix('{') {
            block = name.trim().to_string();
            timeout = None;
            runs_saver = false;
            continue;
        }
        if line == "}" {
            if block == "listener" && runs_saver {
                out.saver_timeout = timeout;
            }
            block.clear();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim().to_string());
        match (block.as_str(), key) {
            ("general", "lock_cmd") => out.lock_cmd = Some(value),
            ("general", "before_sleep_cmd") => out.before_sleep_cmd = Some(value),
            ("general", "after_sleep_cmd") => out.after_sleep_cmd = Some(value),
            ("general", "inhibit_sleep") => out.inhibit_sleep = value.parse().ok(),
            ("listener", "timeout") => timeout = value.parse().ok(),
            ("listener", "on-timeout") => runs_saver = runs_command(&value, "saver"),
            _ => {}
        }
    }
    out
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

// hypridle's idle timer pauses across sleep, so a saver that fires within one timeout of a wake means no input since
// the wake (a lid opened and left): waking must never bring the saver back.
pub fn declines_after_wake(since_wake: Option<Duration>, saver_timeout: Duration) -> bool {
    since_wake.is_some_and(|d| d < saver_timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_after_takes_seconds_or_never_and_defaults_to_locking_at_once() {
        assert_eq!("never".parse(), Ok(LockAfter::Never));
        assert_eq!("30".parse(), Ok(LockAfter::Secs(30)));
        assert!("-5".parse::<LockAfter>().is_err());
        assert!("soon".parse::<LockAfter>().is_err());
        assert_eq!(
            LockAfter::Never.millis(),
            -1,
            "the shell reads negative as never"
        );
        assert_eq!(LockAfter::Secs(5).millis(), 5000);
        assert_eq!(
            LockAfter::DEFAULT,
            LockAfter::Secs(0),
            "an unconfigured saver must not leave the session open"
        );
    }

    #[test]
    fn quality_round_trips() {
        for q in Quality::ALL {
            assert_eq!(q.as_str().parse(), Ok(q));
        }
        assert!("high".parse::<Quality>().is_err());
    }

    #[test]
    fn hypridle_config_yields_the_saver_listener_and_the_general_commands() {
        let conf = r#"
# comment
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
"#;
        let h = parse_hypridle(conf);
        assert_eq!(h.lock_cmd.as_deref(), Some("darwan lock"));
        assert_eq!(h.inhibit_sleep, Some(3));
        assert_eq!(
            h.saver_timeout,
            Some(300),
            "the brightness listener is not the saver's"
        );
        assert!(runs_command(
            h.after_sleep_cmd.as_deref().unwrap(),
            "resumed"
        ));
        assert!(!runs_command("darwan-gui saver", "saver"));
        assert!(!runs_command("echo darwan", "saver"));
    }

    #[test]
    fn a_saver_within_one_timeout_of_a_wake_declines() {
        let t = Duration::from_secs(300);
        assert!(declines_after_wake(Some(Duration::from_secs(40)), t));
        assert!(!declines_after_wake(Some(Duration::from_secs(301)), t));
        assert!(!declines_after_wake(None, t), "no wake recorded");
    }
}
