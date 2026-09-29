use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use darwan_core::config::UserConfig;
use darwan_core::hardware;
use darwan_core::host;
use darwan_core::paths::{self, Paths};
use darwan_core::saver::{self, Quality};
use rustix::fs::{FlockOperation, flock};
use rustix::io::{FdFlags, fcntl_setfd};
use rustix::process::{Pid, Signal, kill_process, kill_process_group};

use crate::session::WaylandSession;
use crate::style;
use crate::{overlay, qs};

pub const SUPERVISOR_ARG: &str = "lock-supervisor";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Lock,
    Saver,
    Sleep,
}

impl Start {
    fn env(self) -> &'static str {
        match self {
            Start::Lock => "lock",
            Start::Saver => "saver",
            Start::Sleep => "sleep",
        }
    }
}

pub fn run(
    paths: &Paths,
    id: Option<&str>,
    replace: bool,
    unlock_after: Option<u32>,
    for_sleep: bool,
) -> Result<ExitCode, String> {
    if unlock_after.is_some() && std::env::var_os("DARWAN_DATA_DIR").is_none() {
        return Err("--unlock-after is for testing from the repo and needs DARWAN_DATA_DIR".into());
    }
    let wayland = WaylandSession::discover()?;
    let prepared = overlay::prepare(paths, id, "lock.conf")?;
    let Some(pid_file) = acquire(&wayland, replace, true)? else {
        return Ok(ExitCode::SUCCESS);
    };
    let mut env = Vec::new();
    if let Some(secs) = unlock_after {
        env.push(("DARWAN_UNLOCK_AFTER", secs.to_string()));
        eprintln!("Test lock: it unlocks by itself after {secs} s.");
    }
    let start = if for_sleep { Start::Sleep } else { Start::Lock };
    let pid = spawn(paths, &wayland, &prepared, pid_file, start, &env)?;
    println!(
        "{} {} with {} {}",
        style::ok("Locked"),
        wayland.display,
        style::id(&prepared.theme.id),
        style::dim(format!("(supervisor pid {pid}).")),
    );
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        println!("Switch back to your graphical session to unlock.");
    }
    Ok(ExitCode::SUCCESS)
}

// The single-instance lock file. None when a supervisor already runs: it is asked to lock (`hand_off`), or else to
// go back to the screensaver, since hypridle's next timeout finds a lock that someone woke and then left.
pub fn acquire(
    wayland: &WaylandSession,
    replace: bool,
    hand_off: bool,
) -> Result<Option<File>, String> {
    let dir = wayland.runtime_dir.join("darwan");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let pid_path = dir.join("lock.pid");
    let mut pid_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&pid_path)
        .map_err(|e| format!("{}: {e}", pid_path.display()))?;

    if flock(&pid_file, FlockOperation::NonBlockingLockExclusive).is_ok() {
        return Ok(Some(pid_file));
    }
    let holder = read_pid(&mut pid_file);
    if replace {
        take_over(&pid_file, holder)?;
        return Ok(Some(pid_file));
    }
    match holder {
        Some(pid) if is_supervisor(pid) && !hand_off => {
            let raw = Pid::from_raw(pid).ok_or("invalid pid")?;
            kill_process(raw, Signal::USR2)
                .map_err(|e| format!("cannot reach the running supervisor (pid {pid}): {e}"))?;
            println!(
                "{}",
                style::dim(format!(
                    "A lock is running (pid {pid}); it goes back to the screensaver."
                ))
            );
        }
        Some(pid) if is_supervisor(pid) => {
            let raw = Pid::from_raw(pid).ok_or("invalid pid")?;
            kill_process(raw, Signal::USR1)
                .map_err(|e| format!("cannot reach the running supervisor (pid {pid}): {e}"))?;
            println!(
                "{} {}",
                style::ok("Locked"),
                style::dim(format!("(handed to the running supervisor, pid {pid}).")),
            );
        }
        _ => {
            let who = holder.map(|p| format!(" (pid {p})")).unwrap_or_default();
            println!(
                "Already locked{who}. {}",
                style::dim("Use --replace to take over a hung lock.")
            );
        }
    }
    Ok(None)
}

pub fn is_supervisor(pid: i32) -> bool {
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    locker_kind(&cmdline) == Some(Locker::Supervisor)
}

// Starts the supervisor, which holds the lock file from here on; returns its pid.
pub fn spawn(
    paths: &Paths,
    wayland: &WaylandSession,
    prepared: &overlay::Prepared,
    mut pid_file: File,
    start: Start,
    extra_env: &[(&str, String)],
) -> Result<u32, String> {
    if let Some(sig) = &wayland.hyprland {
        ensure_lock_restore(sig)?;
    }

    let log_path = paths::state_dir().join("lock.log");
    let log = File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot find the darwan binary: {e}"))?;
    let mut cmd = Command::new(exe);
    cmd.arg(SUPERVISOR_ARG);
    qs::runtime_env(&mut cmd, paths);
    qs::theme_env(
        &mut cmd,
        &prepared.theme,
        &prepared.theme.dir,
        Some(&prepared.overlay),
    );
    wayland.apply(&mut cmd);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    tune(&mut cmd);
    if let Some(output) = wayland.hyprland.as_deref().and_then(focused_output) {
        cmd.env("DARWAN_PRIMARY_OUTPUT", output);
    }
    cmd.env("DARWAN_START", start.env())
        .env("DARWAN_USER", host::user_name())
        .env("DARWAN_SESSIONS", host::sessions_json())
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    // The supervisor and its lockers inherit the flock, so it is held while either lives.
    fcntl_setfd(&pid_file, FdFlags::empty()).map_err(|e| e.to_string())?;
    unsafe {
        cmd.pre_exec(|| rustix::process::setsid().map(drop).map_err(Into::into));
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start the lock supervisor: {e}"))?;

    pid_file
        .set_len(0)
        .and_then(|()| pid_file.rewind())
        .map_err(|e| e.to_string())?;
    write!(pid_file, "{}", child.id()).map_err(|e| e.to_string())?;

    std::thread::sleep(Duration::from_millis(1500));
    if let Ok(Some(status)) = child.try_wait() {
        // A saver that ended by itself this quickly was cancelled by the user or the outputs, which is fine.
        if start == Start::Saver && status.success() {
            return Ok(child.id());
        }
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        let tail: Vec<&str> = log.lines().rev().take(8).collect();
        return Err(format!(
            "the lock stopped at startup ({status}); log {}:\n{}",
            log_path.display(),
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        ));
    }
    Ok(child.id())
}

// saver.return_after for lock_shell.qml and the saver preview.
pub fn return_after_ms(config: Option<&UserConfig>) -> String {
    let secs = config
        .and_then(|c| c.saver_return_after().ok().flatten())
        .unwrap_or(saver::RETURN_AFTER_DEFAULT);
    (u64::from(secs) * 1000).to_string()
}

// What the probe says this machine can afford (saver.quality), and Qt's shader cache so the first frames don't compile.
// A bad setting falls back to its default: the lock must start whatever the config says.
fn tune(cmd: &mut Command) {
    let config = UserConfig::load(&paths::config_file()).ok();
    cmd.env("DARWAN_RETURN_AFTER", return_after_ms(config.as_ref()));
    let quality = config
        .as_ref()
        .and_then(|c| c.saver_quality().ok().flatten())
        .unwrap_or(Quality::DEFAULT);
    let facts = hardware::probe();
    let (tier, _) = hardware::tier(quality, &facts);
    cmd.env("DARWAN_MEDIA_TIER", tier.as_str());
    // Every output decodes its own copy, so only the one the user looks at plays video unless they asked for full.
    let secondary = if quality == Quality::Full {
        tier
    } else {
        hardware::Tier::Still
    };
    cmd.env("DARWAN_SECONDARY_TIER", secondary.as_str());
    // Qt keeps NVIDIA on its single-threaded loop over an old resize bug; lock and saver surfaces never resize, and
    // the threaded loop keeps rendering and video uploads off the thread that handles the waking input.
    if std::env::var_os("QSG_RENDER_LOOP").is_none()
        && facts
            .gpu
            .as_ref()
            .is_some_and(|g| g.vendor == hardware::Vendor::Nvidia)
    {
        cmd.env("QSG_RENDER_LOOP", "threaded");
    }
    let cache = paths::cache_dir();
    if std::fs::create_dir_all(&cache).is_ok() {
        let file = cache.join("pipeline-cache.bin");
        cmd.env("QSG_RHI_PIPELINE_CACHE_SAVE", &file)
            .env("QSG_RHI_PIPELINE_CACHE_LOAD", &file);
    }
}

fn focused_output(signature: &str) -> Option<String> {
    let out = Command::new("hyprctl")
        .args(["--instance", signature, "-j", "monitors"])
        .output()
        .ok()?;
    let monitors: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    monitors
        .as_array()?
        .iter()
        .find(|m| m.get("focused").and_then(serde_json::Value::as_bool) == Some(true))
        .and_then(|m| m.get("name")?.as_str())
        .map(str::to_string)
}

// Without this option a crashed locker leaves the session locked with no way back in.
fn ensure_lock_restore(signature: &str) -> Result<(), String> {
    const OPTION: &str = "misc:allow_session_lock_restore";
    let enabled = || -> Option<bool> {
        let out = Command::new("hyprctl")
            .args(["--instance", signature, "-j", "getoption", OPTION])
            .output()
            .ok()?;
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        json.get("bool")
            .and_then(serde_json::Value::as_bool)
            .or_else(|| {
                json.get("int")
                    .and_then(serde_json::Value::as_i64)
                    .map(|i| i != 0)
            })
    };
    match enabled() {
        Some(true) => return Ok(()),
        None => {
            return Err(format!(
                "cannot read Hyprland's {OPTION}; refusing to lock without a recovery path"
            ));
        }
        Some(false) => {}
    }
    let _ = Command::new("hyprctl")
        .args(["--instance", signature, "keyword", OPTION, "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if enabled() == Some(true) {
        return Ok(());
    }
    Err(format!(
        "Hyprland's {OPTION} is off and could not be turned on (Lua configs reject `hyprctl keyword`). \
         Enable it in your Hyprland config, reload, and lock again; without it a crashed lock can't be recovered"
    ))
}

fn read_pid(file: &mut File) -> Option<i32> {
    let mut s = String::new();
    file.rewind().ok()?;
    file.read_to_string(&mut s).ok()?;
    s.trim().parse().ok()
}

// Kill only a process that is provably ours: a stale pid may belong to anything by now.
fn take_over(pid_file: &File, holder: Option<i32>) -> Result<(), String> {
    let Some(pid) = holder else {
        return Err("the lock is held but records no pid; not killing anything".into());
    };
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    let raw = Pid::from_raw(pid).ok_or("invalid pid")?;
    match locker_kind(&cmdline) {
        // The supervisor leads its own session, so its process group holds every locker it started.
        Some(Locker::Supervisor) => kill_process_group(raw, Signal::KILL),
        Some(Locker::Quickshell) => kill_process(raw, Signal::KILL),
        None => {
            return Err(format!(
                "pid {pid} holds the lock but is not darwan's locker; not killing it"
            ));
        }
    }
    .map_err(|e| format!("cannot kill pid {pid}: {e}"))?;
    println!("Stopped the old locker (pid {pid}).");
    let deadline = Instant::now() + Duration::from_secs(3);
    while flock(pid_file, FlockOperation::NonBlockingLockExclusive).is_err() {
        if Instant::now() > deadline {
            return Err("the old locker did not release the lock within 3 s".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum Locker {
    Supervisor,
    Quickshell,
}

fn locker_kind(cmdline: &[u8]) -> Option<Locker> {
    let args: Vec<&str> = cmdline
        .split(|b| *b == 0)
        .filter(|a| !a.is_empty())
        .map(|a| std::str::from_utf8(a).unwrap_or(""))
        .collect();
    let name = |a: &str| {
        Path::new(a)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    };
    let exe = name(args.first()?);
    if exe == "darwan" && args.get(1) == Some(&SUPERVISOR_ARG) {
        return Some(Locker::Supervisor);
    }
    ((exe == "quickshell" || exe == "qs") && args[1..].iter().any(|a| name(a) == "lock_shell.qml"))
        .then_some(Locker::Quickshell)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_darwans_supervisor_or_a_quickshell_running_lock_shell_is_ours() {
        assert_eq!(
            locker_kind(b"/usr/bin/darwan\0lock-supervisor\0"),
            Some(Locker::Supervisor)
        );
        assert_eq!(
            locker_kind(b"quickshell\0--no-color\0-p\0/usr/share/darwan/runtime/lock_shell.qml\0"),
            Some(Locker::Quickshell),
            "a locker started by an older darwan can still be replaced"
        );
        assert_eq!(
            locker_kind(b"quickshell\0-p\0/home/u/.config/quickshell/shell.qml\0"),
            None,
            "the user's bar must survive --replace"
        );
        assert_eq!(
            locker_kind(b"/usr/bin/darwan\0lock\0"),
            None,
            "a darwan command is not the supervisor"
        );
        assert_eq!(locker_kind(b"vim\0lock_shell.qml\0"), None);
        assert_eq!(locker_kind(b""), None);
    }
}
