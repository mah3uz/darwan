use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use darwan_core::config::UserConfig;
use darwan_core::paths::{self, Paths};
use darwan_core::saver::{self, LockAfter};

use crate::lock::{self, Start};
use crate::session::WaylandSession;
use crate::{overlay, style};

// Without a hypridle listener to read, assume its usual five minutes.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

pub fn run(paths: &Paths) -> Result<ExitCode, String> {
    let wayland = WaylandSession::discover()?;
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let bad = |e: String| format!("{}: {e}", config_path.display());
    let lock_after = config
        .saver_lock_after()
        .map_err(bad)?
        .unwrap_or(LockAfter::DEFAULT);
    // The lock reads saver.quality leniently; the saver reports a bad value to hypridle's log.
    config.saver_quality().map_err(bad)?;

    let timeout = hypridle_timeout().unwrap_or(DEFAULT_TIMEOUT);
    if saver::declines_after_wake(since_wake(&wayland), timeout) {
        println!(
            "{}",
            style::dim(
                "No saver: the machine woke less than one idle timeout ago with no input since."
            )
        );
        return Ok(ExitCode::SUCCESS);
    }

    let prepared = overlay::prepare(paths, None, "lock.conf")?;
    if !prepared.theme.manifest.supports.screensaver {
        return Err(format!(
            "{} has no screensaver mode; pick a theme that has one (darwan doctor lists them)",
            prepared.theme.id
        ));
    }
    let Some(pid_file) = lock::acquire(&wayland, false, false)? else {
        return Ok(ExitCode::SUCCESS);
    };
    let env = [("DARWAN_LOCK_AFTER", lock_after.millis().to_string())];
    lock::spawn(paths, &wayland, &prepared, pid_file, Start::Saver, &env)?;
    Ok(ExitCode::SUCCESS)
}

// hypridle's after_sleep_cmd runs this, so `darwan saver` can tell a wake with no input since.
pub fn resumed() -> Result<ExitCode, String> {
    let wayland = WaylandSession::discover()?;
    let path = stamp(&wayland);
    let dir = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(&path, "").map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(ExitCode::SUCCESS)
}

fn stamp(wayland: &WaylandSession) -> std::path::PathBuf {
    wayland.runtime_dir.join("darwan/resumed")
}

fn since_wake(wayland: &WaylandSession) -> Option<Duration> {
    let modified = std::fs::metadata(stamp(wayland)).ok()?.modified().ok()?;
    SystemTime::now().duration_since(modified).ok()
}

pub fn hypridle_config() -> std::path::PathBuf {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) if !v.is_empty() => std::path::PathBuf::from(v),
        _ => std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"),
    };
    base.join("hypr/hypridle.conf")
}

fn hypridle_timeout() -> Option<Duration> {
    let text = std::fs::read_to_string(hypridle_config()).ok()?;
    saver::parse_hypridle(&text)
        .saver_timeout
        .map(|s| Duration::from_secs(s.into()))
}
