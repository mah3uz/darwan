use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use rustix::fs::{FlockOperation, flock};
use rustix::io::{FdFlags, fcntl_setfd};
use rustix::process::{Pid, Signal, kill_process};

use crate::paths::{self, Paths};
use crate::session::{self, WaylandSession};
use crate::{overlay, qs};

pub fn run(
    paths: &Paths,
    id: Option<&str>,
    replace: bool,
    unlock_after: Option<u32>,
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
            println!("Already locked{who}. Use --replace to take over a hung lock.");
            return Ok(ExitCode::SUCCESS);
        }
        take_over(&pid_file, holder)?;
    }

    if let Some(sig) = &wayland.hyprland {
        ensure_lock_restore(sig)?;
    }

    let log_path = paths::state_dir().join("lock.log");
    let log = File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
    let mut cmd = qs::command(paths, "lock_shell.qml", &[]);
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
    cmd.env("DARWAN_USER", session::user_name())
        .env("DARWAN_SESSIONS", session::sessions_json())
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    // The locker inherits the flock, so the lock lives exactly as long as it does.
    fcntl_setfd(&pid_file, FdFlags::empty()).map_err(|e| e.to_string())?;
    unsafe {
        cmd.pre_exec(|| rustix::process::setsid().map(drop).map_err(Into::into));
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start quickshell: {e}"))?;

    pid_file
        .set_len(0)
        .and_then(|()| pid_file.rewind())
        .map_err(|e| e.to_string())?;
    write!(pid_file, "{}", child.id()).map_err(|e| e.to_string())?;

    std::thread::sleep(Duration::from_millis(800));
    if let Ok(Some(status)) = child.try_wait() {
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        let tail: Vec<&str> = log.lines().rev().take(8).collect();
        return Err(format!(
            "the locker exited at startup ({status}); log {}:\n{}",
            log_path.display(),
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        ));
    }
    println!(
        "Locked {} with {} (locker pid {}).",
        wayland.display,
        prepared.theme.id,
        child.id()
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

// Kill only a process that is provably our locker: a stale pid may belong to anything by now.
fn take_over(pid_file: &File, holder: Option<i32>) -> Result<(), String> {
    let Some(pid) = holder else {
        return Err("the lock is held but records no pid; not killing anything".into());
    };
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    if !is_our_locker(&cmdline) {
        return Err(format!(
            "pid {pid} holds the lock but is not darwan's locker; not killing it"
        ));
    }
    let pid = Pid::from_raw(pid).ok_or("invalid pid")?;
    kill_process(pid, Signal::KILL)
        .map_err(|e| format!("cannot kill pid {}: {e}", pid.as_raw_pid()))?;
    println!("Stopped the old locker (pid {}).", pid.as_raw_pid());
    let deadline = Instant::now() + Duration::from_secs(3);
    while flock(pid_file, FlockOperation::NonBlockingLockExclusive).is_err() {
        if Instant::now() > deadline {
            return Err("the old locker did not release the lock within 3 s".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

fn is_our_locker(cmdline: &[u8]) -> bool {
    let mut args = cmdline.split(|b| *b == 0).filter(|a| !a.is_empty());
    let exe = args
        .next()
        .map(|a| Path::new(std::str::from_utf8(a).unwrap_or("")));
    exe.and_then(Path::file_name)
        .is_some_and(|n| n == "quickshell" || n == "qs")
        && args.any(|a| {
            Path::new(std::str::from_utf8(a).unwrap_or(""))
                .file_name()
                .is_some_and(|n| n == "lock_shell.qml")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_quickshell_running_lock_shell_is_ours() {
        assert!(is_our_locker(
            b"quickshell\0--no-color\0-p\0/usr/share/darwan/runtime/lock_shell.qml\0"
        ));
        assert!(
            !is_our_locker(b"quickshell\0-p\0/home/u/.config/quickshell/shell.qml\0"),
            "the user's bar must survive --replace"
        );
        assert!(!is_our_locker(b"vim\0lock_shell.qml\0"));
        assert!(!is_our_locker(b""));
    }
}
