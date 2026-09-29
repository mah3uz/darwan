use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockAfter {
    Never,
    Secs(u32),
}

impl LockAfter {
    // Ten seconds to come back without a password; 0 locks as the saver appears, "never" doesn't lock.
    pub const DEFAULT: Self = LockAfter::Secs(10);

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

// Seconds a revealed lock that nobody touches keeps its widgets before the screensaver comes back.
pub const RETURN_AFTER_DEFAULT: u32 = 30;
// Any shorter and the widgets would hide while someone is still reaching for the keyboard.
pub const RETURN_AFTER_MIN: u32 = 5;

pub fn check_return_after(secs: i64) -> Result<u32, String> {
    u32::try_from(secs)
        .ok()
        .filter(|&s| s >= RETURN_AFTER_MIN)
        .ok_or_else(|| {
            format!("saver.return_after is {RETURN_AFTER_MIN} or more seconds, not {secs}")
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Auto,
    Full,
    Eco,
    Still,
}

impl Quality {
    // Videos as shipped unless the user trades them for less GPU, power or memory.
    pub const DEFAULT: Self = Quality::Full;
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
            LockAfter::Secs(10),
            "an unconfigured saver locks, after a short grace"
        );
    }

    #[test]
    fn return_after_keeps_the_widgets_up_long_enough_to_use_them() {
        assert_eq!(check_return_after(30), Ok(30));
        assert_eq!(
            check_return_after(i64::from(RETURN_AFTER_MIN)),
            Ok(RETURN_AFTER_MIN)
        );
        assert!(
            check_return_after(0).is_err(),
            "0 would hide the widgets as soon as a click shows them"
        );
        assert!(check_return_after(-30).is_err());
        assert!(check_return_after(i64::from(u32::MAX) + 1).is_err());
    }

    #[test]
    fn quality_round_trips() {
        for q in Quality::ALL {
            assert_eq!(q.as_str().parse(), Ok(q));
        }
        assert!("high".parse::<Quality>().is_err());
    }

    #[test]
    fn a_saver_within_one_timeout_of_a_wake_declines() {
        let t = Duration::from_secs(300);
        assert!(declines_after_wake(Some(Duration::from_secs(40)), t));
        assert!(!declines_after_wake(Some(Duration::from_secs(301)), t));
        assert!(!declines_after_wake(None, t), "no wake recorded");
    }
}
