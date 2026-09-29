use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use darwan_core::paths::Paths;
use rustix::process::{Pid, Signal, kill_process};

use crate::qs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Authenticated,
    // The unlocked saver ended on input, or before it showed.
    Dismissed,
    Failed,
}

impl Outcome {
    // lock_shell.qml quits with 0 only after a successful authentication, and with 4 when an unlocked saver ends.
    fn of(status: ExitStatus) -> Self {
        match status.code() {
            Some(0) => Self::Authenticated,
            Some(4) => Self::Dismissed,
            _ => Self::Failed,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Done,
    Restart(Duration),
    GiveUp(&'static str),
}

// A lock that never took hold protects nothing, so only then do restarts give up.
const TRIES_BEFORE_SECURE: u32 = 3;
const STABLE_RUN: Duration = Duration::from_secs(30);

#[derive(Debug, Default)]
pub struct Policy {
    failures: u32,
    pub ever_secure: bool,
    // Started as the screensaver rather than as a lock.
    pub saver: bool,
    // The saver locks by itself after saver.lock_after.
    pub saver_locks: bool,
    // `darwan lock` (or loginctl lock-session) asked a running saver to lock.
    pub lock_requested: bool,
}

impl Policy {
    // Whether the session is, or may by now be, locked: then nothing may end without authentication.
    pub fn locking(&self) -> bool {
        !self.saver || self.saver_locks || self.lock_requested
    }

    pub fn after_exit(
        &mut self,
        outcome: Outcome,
        ran_for: Duration,
        session_open: bool,
        compositor_alive: bool,
    ) -> Decision {
        match outcome {
            Outcome::Authenticated => return Decision::Done,
            // A lock asked for as the saver was leaving must still happen; a lock never ends by dismissal.
            Outcome::Dismissed if self.saver && !self.lock_requested => return Decision::Done,
            Outcome::Failed if !self.locking() => return Decision::Done,
            _ => {}
        }
        if !session_open {
            return Decision::GiveUp("the login session has ended");
        }
        if !compositor_alive {
            return Decision::GiveUp("the compositor is gone");
        }
        if ran_for >= STABLE_RUN {
            self.failures = 0;
        }
        self.failures += 1;
        if !self.ever_secure && !self.saver && self.failures > TRIES_BEFORE_SECURE {
            return Decision::GiveUp("the lock never took hold");
        }
        Decision::Restart(backoff(self.failures))
    }
}

fn backoff(failures: u32) -> Duration {
    let ms = 100u64 << failures.saturating_sub(1).min(5);
    Duration::from_millis(ms.min(2000))
}

#[derive(Debug, PartialEq)]
pub struct Health {
    pub locked: bool,
    pub secure: bool,
    pub surfaces: Vec<Surface>,
}

#[derive(Debug, PartialEq)]
pub struct Surface {
    pub screen: String,
    pub frames: u64,
    pub focused: bool,
}

pub fn parse_health(text: &str) -> Result<Health, String> {
    let v: serde_json::Value =
        serde_json::from_str(text.trim()).map_err(|e| format!("bad health report: {e}"))?;
    let flag = |k: &str| {
        v.get(k)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    let surfaces = v
        .get("surfaces")
        .and_then(serde_json::Value::as_array)
        .map(|a| {
            a.iter()
                .map(|s| Surface {
                    screen: s
                        .get("screen")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    frames: s
                        .get("frames")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    focused: s
                        .get("focused")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Health {
        locked: flag("locked"),
        secure: flag("secure"),
        surfaces,
    })
}

// Frames and focus are only meaningful after the outputs came on and resetFrames was called.
pub fn verdict(h: &Health) -> Result<(), String> {
    if !h.locked || !h.secure {
        return Err("the session is not locked".into());
    }
    if h.surfaces.is_empty() {
        return Err("there are no lock surfaces".into());
    }
    if let Some(s) = h.surfaces.iter().find(|s| s.frames == 0) {
        return Err(format!("the lock surface on {} has not drawn", s.screen));
    }
    if !h.surfaces.iter().any(|s| s.focused) {
        return Err("no lock surface has keyboard focus".into());
    }
    Ok(())
}

enum Event {
    Exited(ExitStatus),
    SleepStart(Sender<()>),
    SleepEnd,
    OutputOn,
    OutputsOff,
    LockRequest,
    SaverRequest,
    Activity,
    LockedIdle,
    LockedActive,
}

#[derive(Clone, Copy, PartialEq)]
enum Check {
    // Right after the lock starts: only learns whether the lock took hold.
    Startup(u8),
    // After outputs came on or wake: a failed second check restarts the locker.
    Wake(u8),
}

pub fn run() -> ExitCode {
    let start = Instant::now();
    let log = move |msg: String| {
        eprintln!(
            "darwan supervisor [{:.1}s]: {msg}",
            start.elapsed().as_secs_f32()
        )
    };
    let paths = Paths::detect();
    let (tx, rx) = mpsc::channel::<Event>();
    signals::watch(tx.clone());
    logind::watch_sleep(tx.clone(), log);
    outputs::watch(tx.clone(), log);

    let saver = std::env::var("DARWAN_START").as_deref() == Ok("saver");
    let lock_after = std::env::var("DARWAN_LOCK_AFTER")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    if saver {
        idle::watch(tx.clone(), log);
    }
    let mut policy = Policy {
        saver,
        saver_locks: lock_after >= 0,
        ..Policy::default()
    };
    let screen = ScreenOff::from_env();
    let mut watching_locked = false;
    // Darwan turned the outputs off, so Darwan turns them on again.
    let mut screens_off = false;
    let mut first = true;
    loop {
        let mut cmd = qs::command(&paths, "lock_shell.qml", &[]);
        // A restart is always a lock, showing its widgets: the saver or the sleep lock were the first run's.
        if !first {
            cmd.env("DARWAN_START", "lock");
        }
        signals::unblock_in_child(&mut cmd);
        let mut child = match cmd.stdin(Stdio::null()).spawn() {
            Ok(c) => c,
            Err(e) => {
                log(format!("cannot start quickshell: {e}"));
                return ExitCode::FAILURE;
            }
        };
        let pid = child.id();
        log(format!("locker started (pid {pid})"));
        let started = Instant::now();
        let waiter = tx.clone();
        std::thread::spawn(move || {
            if let Ok(status) = child.wait() {
                let _ = waiter.send(Event::Exited(status));
            }
        });

        // The saver locks on its own lock_after timer, so its lock is checked a moment after that.
        let first_check = match (first && saver, lock_after) {
            (false, _) => Some(Duration::from_secs(1)),
            (true, ms) if ms >= 0 => Some(Duration::from_millis(ms as u64 + 3000)),
            (true, _) => None,
        };
        first = false;
        let mut check = first_check.map(|d| (Instant::now() + d, Check::Startup(0)));
        let status = loop {
            let wait = check.map_or(Duration::from_secs(3600), |(at, _)| {
                at.saturating_duration_since(Instant::now())
            });
            match rx.recv_timeout(wait) {
                Ok(Event::Exited(status)) => break status,
                Ok(Event::LockRequest) => {
                    log("lock requested".into());
                    policy.lock_requested = true;
                    ipc(pid, "lock");
                    check = Some((Instant::now() + Duration::from_secs(1), Check::Startup(0)));
                }
                Ok(Event::SaverRequest) => {
                    let answer = ipc(pid, "ambient").unwrap_or_default();
                    log(format!("screensaver requested: {}", answer.trim()));
                }
                Ok(Event::Activity) => {
                    let answer = ipc(pid, "activity").unwrap_or_default();
                    log(format!("activity while loading: {}", answer.trim()));
                }
                Ok(Event::OutputsOff) | Ok(Event::SleepStart(_)) if !policy.locking() => {
                    // Nothing to protect: waking shows the desktop, never the saver.
                    log("outputs off or sleep with an unlocked saver; ending it".into());
                    if let Some(p) = Pid::from_raw(pid as i32) {
                        let _ = kill_process(p, Signal::TERM);
                    }
                    policy.saver_locks = false;
                }
                Ok(Event::OutputsOff) => {
                    log("outputs off: locking and unloading the theme".into());
                    ipc(pid, "lock");
                    ipc(pid, "unloadTheme");
                    check = None;
                }
                Ok(Event::SleepStart(ack)) => {
                    ipc(pid, "lock");
                    ipc(pid, "unloadTheme");
                    let _ = ack.send(());
                    check = None;
                }
                Ok(Event::LockedIdle) => match &screen.commands {
                    Some((off, _)) => {
                        log(format!(
                            "locked and untouched for {} s: turning the screens off",
                            screen.after_ms.unwrap_or_default() / 1000
                        ));
                        screens_off = run_shell(off);
                    }
                    None => log(
                        "locked and untouched, but only Hyprland's screens can be turned off"
                            .into(),
                    ),
                },
                Ok(Event::LockedActive) => {
                    if screens_off && let Some((_, on)) = &screen.commands {
                        log("input: turning the screens on".into());
                        run_shell(on);
                        screens_off = false;
                    }
                }
                Ok(Event::OutputOn) if policy.locking() => {
                    ipc(pid, "resetFrames");
                    check = Some((Instant::now() + Duration::from_secs(3), Check::Wake(0)));
                }
                Ok(Event::OutputOn) => {}
                Ok(Event::SleepEnd) => {
                    ipc(pid, "loadTheme");
                    ipc(pid, "resetFrames");
                    check = Some((Instant::now() + Duration::from_secs(3), Check::Wake(0)));
                }
                Err(RecvTimeoutError::Timeout) => {
                    let Some((_, kind)) = check.take() else {
                        continue;
                    };
                    let health = ipc(pid, "health")
                        .ok_or_else(|| "no answer over IPC".to_string())
                        .and_then(|t| parse_health(&t));
                    match kind {
                        Check::Startup(n) => match health {
                            Ok(h) if h.locked && h.secure => {
                                policy.ever_secure = true;
                                log("the session is locked".into());
                                if !watching_locked && let Some(ms) = screen.after_ms {
                                    watching_locked = true;
                                    locked_idle::watch(tx.clone(), ms, log);
                                }
                            }
                            _ if n < 9 => {
                                check = Some((
                                    Instant::now() + Duration::from_secs(1),
                                    Check::Startup(n + 1),
                                ));
                            }
                            _ => log("the lock has not taken hold after 10 s".into()),
                        },
                        Check::Wake(n) => match health.and_then(|h| verdict(&h)) {
                            Ok(()) => log("health check passed".into()),
                            Err(why) if n == 0 => {
                                log(format!("health check: {why}; checking again"));
                                check =
                                    Some((Instant::now() + Duration::from_secs(2), Check::Wake(1)));
                            }
                            Err(why) => {
                                log(format!(
                                    "health check failed again ({why}); restarting the locker"
                                ));
                                if let Some(p) = Pid::from_raw(pid as i32) {
                                    let _ = kill_process(p, Signal::KILL);
                                }
                            }
                        },
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    unreachable!("the supervisor keeps a sender")
                }
            }
        };

        let outcome = Outcome::of(status);
        log(format!("locker exited ({status})"));
        match policy.after_exit(
            outcome,
            started.elapsed(),
            logind::session_open(),
            compositor_alive(),
        ) {
            Decision::Done => {
                if screens_off && let Some((_, on)) = &screen.commands {
                    run_shell(on);
                }
                log(match outcome {
                    Outcome::Authenticated => "unlocked".into(),
                    _ => "the saver ended".into(),
                });
                return ExitCode::SUCCESS;
            }
            Decision::GiveUp(why) => {
                log(format!("not restarting: {why}"));
                return ExitCode::FAILURE;
            }
            Decision::Restart(delay) => {
                log(format!("restarting in {} ms", delay.as_millis()));
                policy.saver = false;
                std::thread::sleep(delay);
            }
        }
    }
}

// A hung locker must not hang the supervisor, so every call has a deadline.
// saver.screen_off_locked, from the environment `darwan lock` gives the supervisor, and the commands for it.
struct ScreenOff {
    after_ms: Option<u32>,
    commands: Option<(String, String)>,
}

impl ScreenOff {
    fn from_env() -> Self {
        let setting = std::env::var("DARWAN_SCREEN_OFF_LOCKED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(darwan_core::saver::ScreenOffLocked::DEFAULT);
        let hyprland = std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some();
        Self::new(setting, hyprland, darwan_core::hypridle::hyprland_lua())
    }

    fn new(setting: darwan_core::saver::ScreenOffLocked, hyprland: bool, lua: bool) -> Self {
        use darwan_core::hypridle::dpms;
        Self {
            after_ms: setting.secs().map(|s| s.saturating_mul(1000)),
            commands: hyprland.then(|| (dpms(lua, false), dpms(lua, true))),
        }
    }
}

fn run_shell(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", cmd])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn ipc(pid: u32, function: &str) -> Option<String> {
    let mut child = Command::new("quickshell")
        .args(["ipc", "--pid", &pid.to_string(), "call", "lock", function])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if Instant::now() > deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
    let mut out = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take()?, &mut out).ok()?;
    Some(out)
}

fn compositor_alive() -> bool {
    let (Some(dir), Some(display)) = (
        std::env::var_os("XDG_RUNTIME_DIR"),
        std::env::var_os("WAYLAND_DISPLAY"),
    ) else {
        return false;
    };
    UnixStream::connect(PathBuf::from(dir).join(display)).is_ok()
}

mod logind {
    use std::sync::mpsc::{self, Sender};
    use std::time::Duration;

    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{OwnedFd, OwnedObjectPath};

    use super::Event;

    const DEST: &str = "org.freedesktop.login1";

    fn manager(conn: &Connection) -> zbus::Result<Proxy<'_>> {
        Proxy::new(
            conn,
            DEST,
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )
    }

    fn inhibit(mgr: &Proxy<'_>) -> Option<OwnedFd> {
        mgr.call(
            "Inhibit",
            &(
                "sleep",
                "darwan",
                "Prepare the lock screen for sleep",
                "delay",
            ),
        )
        .ok()
    }

    // Holds a delay inhibitor while alive, as systemd's inhibitor-lock guidance asks of lockers.
    pub fn watch_sleep(tx: Sender<Event>, log: impl Fn(String) + Send + 'static) {
        std::thread::spawn(move || {
            let run = || -> zbus::Result<()> {
                let conn = Connection::system()?;
                let mgr = manager(&conn)?;
                let signals = mgr.receive_signal("PrepareForSleep")?;
                let mut held = inhibit(&mgr);
                for msg in signals {
                    let sleeping: bool = msg.body().deserialize()?;
                    if sleeping {
                        let (ack_tx, ack_rx) = mpsc::channel();
                        if tx.send(Event::SleepStart(ack_tx)).is_err() {
                            break;
                        }
                        let _ = ack_rx.recv_timeout(Duration::from_secs(3));
                        held = None;
                    } else {
                        held = held.or_else(|| inhibit(&mgr));
                        if tx.send(Event::SleepEnd).is_err() {
                            break;
                        }
                    }
                }
                drop(held);
                Ok(())
            };
            if let Err(e) = run() {
                log(format!("logind unavailable, sleep handling off: {e}"));
            }
        });
    }

    // Unknown means open: never stop protecting a session because D-Bus hiccupped.
    pub fn session_open() -> bool {
        let state = || -> zbus::Result<String> {
            let conn = Connection::system()?;
            let mgr = manager(&conn)?;
            let uid = rustix::process::getuid().as_raw();
            let user: OwnedObjectPath = mgr.call("GetUser", &(uid,))?;
            let user = Proxy::new(&conn, DEST, user, "org.freedesktop.login1.User")?;
            let (_, session): (String, OwnedObjectPath) = user.get_property("Display")?;
            let session = Proxy::new(&conn, DEST, session, "org.freedesktop.login1.Session")?;
            session.get_property("State")
        };
        state().map_or(true, |s| s != "closing")
    }
}

mod outputs {
    use std::collections::HashMap;
    use std::sync::mpsc::Sender;

    use wayland_client::protocol::{wl_output, wl_registry};
    use wayland_client::{Connection, Dispatch, QueueHandle};
    use wayland_protocols_wlr::output_power_management::v1::client::{
        zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1,
        zwlr_output_power_v1::{self, Mode, ZwlrOutputPowerV1},
    };

    use super::Event;

    struct State {
        tx: Sender<Event>,
        manager: Option<ZwlrOutputPowerManagerV1>,
        pending: Vec<wl_output::WlOutput>,
        on: HashMap<u32, bool>,
        all_off: bool,
    }

    impl Dispatch<wl_registry::WlRegistry, ()> for State {
        fn event(
            s: &mut Self,
            reg: &wl_registry::WlRegistry,
            ev: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            let wl_registry::Event::Global {
                name,
                interface,
                version,
            } = ev
            else {
                return;
            };
            match interface.as_str() {
                "zwlr_output_power_manager_v1" => {
                    let mgr: ZwlrOutputPowerManagerV1 = reg.bind(name, 1, qh, ());
                    for o in s.pending.drain(..) {
                        mgr.get_output_power(&o, qh, name_of(&o));
                    }
                    s.manager = Some(mgr);
                }
                "wl_output" => {
                    let o: wl_output::WlOutput = reg.bind(name, version.min(4), qh, ());
                    match &s.manager {
                        Some(mgr) => {
                            mgr.get_output_power(&o, qh, name_of(&o));
                        }
                        None => s.pending.push(o),
                    }
                }
                _ => {}
            }
        }
    }

    fn name_of(o: &wl_output::WlOutput) -> u32 {
        wayland_client::Proxy::id(o).protocol_id()
    }

    impl Dispatch<wl_output::WlOutput, ()> for State {
        fn event(
            _: &mut Self,
            _: &wl_output::WlOutput,
            _: wl_output::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
        }
    }

    impl Dispatch<ZwlrOutputPowerManagerV1, ()> for State {
        fn event(
            _: &mut Self,
            _: &ZwlrOutputPowerManagerV1,
            _: <ZwlrOutputPowerManagerV1 as wayland_client::Proxy>::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
        }
    }

    // The first report for an output only records its state; later transitions to On are events, and so is the moment
    // every output is off (a saver starting on dark outputs included).
    impl Dispatch<ZwlrOutputPowerV1, u32> for State {
        fn event(
            s: &mut Self,
            _: &ZwlrOutputPowerV1,
            ev: zwlr_output_power_v1::Event,
            id: &u32,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let zwlr_output_power_v1::Event::Mode { mode } = ev {
                let now_on = matches!(mode.into_result(), Ok(Mode::On));
                let was_on = s.on.insert(*id, now_on);
                if now_on && was_on == Some(false) {
                    let _ = s.tx.send(Event::OutputOn);
                }
                let all_off = s.on.values().all(|on| !on);
                if all_off && !s.all_off {
                    let _ = s.tx.send(Event::OutputsOff);
                }
                s.all_off = all_off;
            }
        }
    }

    pub fn watch(tx: Sender<Event>, log: impl Fn(String) + Send + 'static) {
        std::thread::spawn(move || {
            let conn = match Connection::connect_to_env() {
                Ok(c) => c,
                Err(e) => return log(format!("cannot watch outputs: {e}")),
            };
            let mut queue = conn.new_event_queue();
            let qh = queue.handle();
            conn.display().get_registry(&qh, ());
            let mut state = State {
                tx,
                manager: None,
                pending: Vec::new(),
                on: HashMap::new(),
                all_off: false,
            };
            if queue.roundtrip(&mut state).is_err() {
                return;
            }
            if state.manager.is_none() {
                return log(
                    "the compositor has no output power protocol; post-DPMS health checks off"
                        .into(),
                );
            }
            while queue.blocking_dispatch(&mut state).is_ok() {}
        });
    }
}

mod signals {
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    use std::sync::mpsc::Sender;

    use super::Event;

    fn ours() -> libc::sigset_t {
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            libc::sigaddset(&mut set, libc::SIGUSR1);
            libc::sigaddset(&mut set, libc::SIGUSR2);
            set
        }
    }

    // `darwan lock` sends SIGUSR1 to hand a lock request to a running saver, `darwan saver` SIGUSR2 to send a running
    // lock back to ambient. Called before any other thread starts, so every thread inherits the block and only this
    // one takes the signals.
    pub fn watch(tx: Sender<Event>) {
        let set = ours();
        unsafe {
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
        }
        std::thread::spawn(move || {
            loop {
                let mut sig = 0;
                if unsafe { libc::sigwait(&set, &mut sig) } != 0 {
                    return;
                }
                let event = if sig == libc::SIGUSR2 {
                    Event::SaverRequest
                } else {
                    Event::LockRequest
                };
                if tx.send(event).is_err() {
                    return;
                }
            }
        });
    }

    // Children inherit the blocked mask; Quickshell must not.
    pub fn unblock_in_child(cmd: &mut Command) {
        unsafe {
            cmd.pre_exec(|| {
                let set = ours();
                libc::pthread_sigmask(libc::SIG_UNBLOCK, &set, std::ptr::null_mut());
                Ok(())
            });
        }
    }
}

// The first input after the saver started, seen by the compositor. The saver's surfaces take no input while its theme
// loads, so this is how a user who comes straight back cancels it.
mod idle {
    use std::sync::mpsc::Sender;

    use wayland_client::protocol::{wl_registry, wl_seat};
    use wayland_client::{Connection, Dispatch, QueueHandle};
    use wayland_protocols::ext::idle_notify::v1::client::{
        ext_idle_notification_v1::{self, ExtIdleNotificationV1},
        ext_idle_notifier_v1::ExtIdleNotifierV1,
    };

    use super::Event;

    #[derive(Default)]
    struct State {
        notifier: Option<ExtIdleNotifierV1>,
        seat: Option<wl_seat::WlSeat>,
        resumed: bool,
    }

    impl Dispatch<wl_registry::WlRegistry, ()> for State {
        fn event(
            s: &mut Self,
            reg: &wl_registry::WlRegistry,
            ev: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            if let wl_registry::Event::Global {
                name, interface, ..
            } = ev
            {
                match interface.as_str() {
                    "ext_idle_notifier_v1" => s.notifier = Some(reg.bind(name, 1, qh, ())),
                    "wl_seat" if s.seat.is_none() => s.seat = Some(reg.bind(name, 1, qh, ())),
                    _ => {}
                }
            }
        }
    }

    impl Dispatch<ExtIdleNotificationV1, ()> for State {
        fn event(
            s: &mut Self,
            _: &ExtIdleNotificationV1,
            ev: ext_idle_notification_v1::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if matches!(ev, ext_idle_notification_v1::Event::Resumed) {
                s.resumed = true;
            }
        }
    }

    wayland_client::delegate_noop!(State: ignore ExtIdleNotifierV1);
    wayland_client::delegate_noop!(State: ignore wl_seat::WlSeat);

    pub fn watch(tx: Sender<Event>, log: impl Fn(String) + Send + 'static) {
        std::thread::spawn(move || {
            let Ok(conn) = Connection::connect_to_env() else {
                return;
            };
            let mut queue = conn.new_event_queue();
            let qh = queue.handle();
            conn.display().get_registry(&qh, ());
            let mut state = State::default();
            if queue.roundtrip(&mut state).is_err() {
                return;
            }
            let (Some(notifier), Some(seat)) = (&state.notifier, &state.seat) else {
                return log("the compositor has no idle notifications; the saver can't be cancelled while it loads".into());
            };
            let notification = notifier.get_idle_notification(0, seat, &qh, ());
            while !state.resumed {
                if queue.blocking_dispatch(&mut state).is_err() {
                    return;
                }
            }
            notification.destroy();
            let _ = tx.send(Event::Activity);
        });
    }
}

// saver.screen_off_locked: idle counted from the moment the session is locked, and restarted by any input. Built from
// get_input_idle_notification where the compositor has it, so a video playing behind the lock doesn't keep the
// screens on.
mod locked_idle {
    use std::sync::mpsc::Sender;

    use wayland_client::protocol::{wl_registry, wl_seat};
    use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
    use wayland_protocols::ext::idle_notify::v1::client::{
        ext_idle_notification_v1::{self, ExtIdleNotificationV1},
        ext_idle_notifier_v1::ExtIdleNotifierV1,
    };

    use super::Event;

    struct State {
        notifier: Option<ExtIdleNotifierV1>,
        seat: Option<wl_seat::WlSeat>,
        tx: Sender<Event>,
    }

    impl Dispatch<wl_registry::WlRegistry, ()> for State {
        fn event(
            s: &mut Self,
            reg: &wl_registry::WlRegistry,
            ev: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            if let wl_registry::Event::Global {
                name,
                interface,
                version,
            } = ev
            {
                match interface.as_str() {
                    "ext_idle_notifier_v1" => {
                        s.notifier = Some(reg.bind(name, version.min(2), qh, ()))
                    }
                    "wl_seat" if s.seat.is_none() => s.seat = Some(reg.bind(name, 1, qh, ())),
                    _ => {}
                }
            }
        }
    }

    impl Dispatch<ExtIdleNotificationV1, ()> for State {
        fn event(
            s: &mut Self,
            _: &ExtIdleNotificationV1,
            ev: ext_idle_notification_v1::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            let _ = s.tx.send(match ev {
                ext_idle_notification_v1::Event::Idled => Event::LockedIdle,
                _ => Event::LockedActive,
            });
        }
    }

    wayland_client::delegate_noop!(State: ignore ExtIdleNotifierV1);
    wayland_client::delegate_noop!(State: ignore wl_seat::WlSeat);

    pub fn watch(tx: Sender<Event>, timeout_ms: u32, log: impl Fn(String) + Send + 'static) {
        std::thread::spawn(move || {
            let Ok(conn) = Connection::connect_to_env() else {
                return;
            };
            let mut queue = conn.new_event_queue();
            let qh = queue.handle();
            conn.display().get_registry(&qh, ());
            let mut state = State {
                notifier: None,
                seat: None,
                tx,
            };
            if queue.roundtrip(&mut state).is_err() {
                return;
            }
            let (Some(notifier), Some(seat)) = (&state.notifier, &state.seat) else {
                return log(
                    "the compositor has no idle notifications; a locked screen stays on".into(),
                );
            };
            log(format!(
                "the screens turn off after {} s untouched while locked{}",
                timeout_ms / 1000,
                if notifier.version() >= 2 {
                    ""
                } else {
                    " (idle inhibitors keep them on)"
                }
            ));
            let _notification = if notifier.version() >= 2 {
                notifier.get_input_idle_notification(timeout_ms, seat, &qh, ())
            } else {
                notifier.get_idle_notification(timeout_ms, seat, &qh, ())
            };
            while queue.blocking_dispatch(&mut state).is_ok() {}
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exit_code(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }

    // Against the running compositor; changes nothing on screen. `cargo test -- --ignored locked_idle`, hands off.
    #[test]
    #[ignore]
    fn locked_idle_reports_idle_after_its_timeout() {
        let (tx, rx) = mpsc::channel();
        locked_idle::watch(tx, 1500, |m| eprintln!("{m}"));
        let started = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_secs(20)) {
                Ok(Event::LockedIdle) => break,
                Ok(_) => continue,
                Err(e) => panic!("no idle event: {e:?}"),
            }
        }
        assert!(
            started.elapsed() >= Duration::from_millis(1400),
            "counted from creation"
        );
    }

    #[test]
    fn a_locked_screen_turns_off_through_hyprland_in_its_own_syntax_and_nowhere_else() {
        use darwan_core::saver::ScreenOffLocked;
        let s = ScreenOff::new(ScreenOffLocked::DEFAULT, true, false);
        assert_eq!(
            s.after_ms,
            Some(300_000),
            "five minutes after the lock by default"
        );
        assert_eq!(
            s.commands,
            Some((
                "hyprctl dispatch dpms off".into(),
                "hyprctl dispatch dpms on".into()
            ))
        );
        let lua = ScreenOff::new(ScreenOffLocked::Secs(60), true, true);
        assert!(lua.commands.unwrap().0.contains("hl.dsp.dpms"));
        assert_eq!(
            ScreenOff::new(ScreenOffLocked::Never, true, false).after_ms,
            None,
            "never: no watcher at all"
        );
        assert_eq!(
            ScreenOff::new(ScreenOffLocked::DEFAULT, false, false).commands,
            None,
            "no way to turn another compositor's screens off yet"
        );
    }

    #[test]
    fn only_a_clean_exit_counts_as_an_authenticated_unlock() {
        assert_eq!(Outcome::of(exit_code(0)), Outcome::Authenticated);
        assert_eq!(Outcome::of(exit_code(4)), Outcome::Dismissed);
        assert_eq!(Outcome::of(exit_code(3)), Outcome::Failed);
        assert_eq!(Outcome::of(exit_code(255)), Outcome::Failed);
    }

    #[test]
    fn an_authenticated_unlock_ends_the_supervisor() {
        let mut p = Policy::default();
        assert_eq!(
            p.after_exit(Outcome::Authenticated, Duration::ZERO, true, true),
            Decision::Done
        );
    }

    #[test]
    fn a_saver_that_never_locks_ends_quietly_even_when_it_crashes() {
        let mut p = Policy {
            saver: true,
            ..Policy::default()
        };
        assert!(!p.locking());
        assert_eq!(
            p.after_exit(Outcome::Dismissed, Duration::ZERO, true, true),
            Decision::Done
        );
        assert_eq!(
            p.after_exit(Outcome::Failed, Duration::ZERO, true, true),
            Decision::Done,
            "an unlocked saver protected nothing, so it isn't brought back"
        );
    }

    #[test]
    fn a_saver_that_may_have_locked_is_restarted_as_a_lock_after_a_crash() {
        // It may have locked on its own timer just before dying; guessing "unlocked" would open the session.
        let mut p = Policy {
            saver: true,
            saver_locks: true,
            ..Policy::default()
        };
        assert!(p.locking());
        assert!(matches!(
            p.after_exit(Outcome::Failed, Duration::ZERO, true, true),
            Decision::Restart(_)
        ));
    }

    #[test]
    fn a_lock_requested_while_the_saver_was_leaving_still_happens() {
        let mut p = Policy {
            saver: true,
            lock_requested: true,
            ..Policy::default()
        };
        assert!(matches!(
            p.after_exit(Outcome::Dismissed, Duration::ZERO, true, true),
            Decision::Restart(_)
        ));
    }

    #[test]
    fn a_lock_never_ends_by_dismissal() {
        let mut p = Policy::default();
        assert!(matches!(
            p.after_exit(Outcome::Dismissed, Duration::ZERO, true, true),
            Decision::Restart(_)
        ));
    }

    #[test]
    fn a_crashed_lock_is_restarted_for_as_long_as_the_session_lives() {
        // NVIDIA resume churns outputs for ~26 s; a supervisor that gives up leaves the user locked out.
        let mut p = Policy {
            ever_secure: true,
            ..Policy::default()
        };
        for _ in 0..50 {
            assert!(matches!(
                p.after_exit(Outcome::Failed, Duration::from_millis(200), true, true),
                Decision::Restart(_)
            ));
        }
    }

    #[test]
    fn restarts_stop_when_the_session_or_compositor_is_gone() {
        let mut p = Policy {
            ever_secure: true,
            ..Policy::default()
        };
        assert!(matches!(
            p.after_exit(Outcome::Failed, Duration::ZERO, false, true),
            Decision::GiveUp(_)
        ));
        assert!(matches!(
            p.after_exit(Outcome::Failed, Duration::ZERO, true, false),
            Decision::GiveUp(_)
        ));
    }

    #[test]
    fn a_lock_that_never_took_hold_gives_up_after_a_few_tries() {
        let mut p = Policy::default();
        for _ in 0..TRIES_BEFORE_SECURE {
            assert!(matches!(
                p.after_exit(Outcome::Failed, Duration::ZERO, true, true),
                Decision::Restart(_)
            ));
        }
        assert!(matches!(
            p.after_exit(Outcome::Failed, Duration::ZERO, true, true),
            Decision::GiveUp(_)
        ));
    }

    #[test]
    fn backoff_grows_to_two_seconds_and_resets_after_a_stable_run() {
        let mut p = Policy {
            ever_secure: true,
            ..Policy::default()
        };
        let delays: Vec<Decision> = (0..8)
            .map(|_| p.after_exit(Outcome::Failed, Duration::ZERO, true, true))
            .collect();
        assert_eq!(delays[0], Decision::Restart(Duration::from_millis(100)));
        assert_eq!(delays[1], Decision::Restart(Duration::from_millis(200)));
        assert_eq!(delays[7], Decision::Restart(Duration::from_secs(2)));
        assert_eq!(
            p.after_exit(Outcome::Failed, STABLE_RUN, true, true),
            Decision::Restart(Duration::from_millis(100))
        );
    }

    #[test]
    fn health_reports_parse_and_judge_drawing_and_focus() {
        let ok = parse_health(r#"{"locked":true,"secure":true,"authenticated":false,"themeLoaded":true,"surfaces":[{"screen":"DP-1","frames":12,"focused":true},{"screen":"DP-2","frames":3,"focused":false}]}"#).unwrap();
        assert_eq!(verdict(&ok), Ok(()));

        let stale = parse_health(r#"{"locked":true,"secure":true,"surfaces":[{"screen":"DP-1","frames":0,"focused":true}]}"#).unwrap();
        assert!(
            verdict(&stale).unwrap_err().contains("DP-1"),
            "a surface that never redraws after resume (#16315)"
        );

        let unfocused = parse_health(r#"{"locked":true,"secure":true,"surfaces":[{"screen":"DP-1","frames":5,"focused":false}]}"#).unwrap();
        assert!(
            verdict(&unfocused).unwrap_err().contains("focus"),
            "no keyboard after resume (#11208)"
        );

        let unlocked = parse_health(r#"{"locked":false,"secure":false,"surfaces":[]}"#).unwrap();
        assert!(verdict(&unlocked).is_err());
        assert!(parse_health("not json").is_err());
    }
}
