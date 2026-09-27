use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use darwan_core::host;
use darwan_core::paths::{self, Paths};
use rustix::fs::{FlockOperation, flock};
use rustix::io::{FdFlags, fcntl_setfd};
use rustix::process::{Pid, Signal, kill_process, kill_process_group};

use crate::session::WaylandSession;
use crate::style;
use crate::{overlay, qs};

pub const SUPERVISOR_ARG: &str = "lock-supervisor";

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

    if flock(&pid_file, FlockOperation::NonBlockingLockExclusive).is_err() {
        let holder = read_pid(&mut pid_file);
        if !replace {
            let who = holder.map(|p| format!(" (pid {p})")).unwrap_or_default();
            println!(
                "Already locked{who}. {}",
                style::dim("Use --replace to take over a hung lock.")
            );
            return Ok(ExitCode::SUCCESS);
        }
        take_over(&pid_file, holder)?;
    }

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
    if let Some(secs) = unlock_after {
        cmd.env("DARWAN_UNLOCK_AFTER", secs.to_string());
        eprintln!("Test lock: it unlocks by itself after {secs} s.");
    }
    if for_sleep {
        cmd.env("DARWAN_FOR_SLEEP", "1");
    }
    cmd.env("DARWAN_USER", host::user_name())
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
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        let tail: Vec<&str> = log.lines().rev().take(8).collect();
        return Err(format!(
            "the lock stopped at startup ({status}); log {}:\n{}",
            log_path.display(),
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        ));
    }
    println!(
        "{} {} with {} {}",
        style::ok("Locked"),
        wayland.display,
        style::id(&prepared.theme.id),
        style::dim(format!("(supervisor pid {}).", child.id()))
    );
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        println!("Switch back to your graphical session to unlock.");
    }
    Ok(ExitCode::SUCCESS)
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
