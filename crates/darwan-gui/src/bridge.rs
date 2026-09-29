#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, revision, READ, NOTIFY)]
        #[qproperty(bool, busy, READ, NOTIFY)]
        #[qproperty(bool, dirty, READ, NOTIFY)]
        #[qproperty(QString, status, READ, NOTIFY)]
        #[qproperty(bool, status_ok, READ, NOTIFY)]
        #[qproperty(QString, runtime_dir, READ, CONSTANT)]
        #[qproperty(QString, overlay_path, READ, CONSTANT)]
        #[qproperty(QString, user_name, READ, CONSTANT)]
        #[qproperty(QString, host_name, READ, CONSTANT)]
        #[qproperty(QString, sessions, READ, CONSTANT)]
        #[qproperty(QString, icon_path, READ, CONSTANT)]
        #[qproperty(i32, wall_revision, READ, NOTIFY)]
        #[qproperty(i32, wall_feed_revision, READ, NOTIFY)]
        #[qproperty(bool, wall_busy, READ, NOTIFY)]
        type Backend = super::BackendRust;

        #[qinvokable]
        fn details(self: &Backend, id: &QString) -> QString;

        #[qinvokable]
        fn fields(self: &Backend, id: &QString) -> QString;

        #[qinvokable]
        fn wall(self: &Backend, query: &QString, filter: &QString) -> QString;

        #[qinvokable]
        fn form(self: &Backend, id: &QString) -> QString;

        #[qinvokable]
        fn globals(self: &Backend, id: &QString) -> QString;

        #[qinvokable]
        fn saver_panel(self: &Backend) -> QString;

        #[qinvokable]
        fn value(self: &Backend, key: &QString) -> QString;

        #[qinvokable]
        fn health(self: &Backend) -> QString;

        #[qinvokable]
        fn saver_do(self: Pin<&mut Backend>, action: &QString, value: &QString);

        #[qinvokable]
        fn availability(self: &Backend) -> QString;

        #[qinvokable]
        fn desktop_wallpaper(self: &Backend) -> QString;

        #[qinvokable]
        fn report(self: &Backend, output: &QString) -> QString;

        #[qinvokable]
        fn set_value(self: Pin<&mut Backend>, key: &QString, value: &QString);

        #[qinvokable]
        fn reset_value(self: Pin<&mut Backend>, key: &QString);

        #[qinvokable]
        fn reset_theme(self: Pin<&mut Backend>, id: &QString);

        #[qinvokable]
        fn set_value_now(self: Pin<&mut Backend>, key: &QString, value: &QString);

        #[qinvokable]
        fn save_changes(self: Pin<&mut Backend>) -> bool;

        #[qinvokable]
        fn discard_changes(self: Pin<&mut Backend>);

        #[qinvokable]
        fn write_overlay(self: Pin<&mut Backend>, id: &QString) -> bool;

        #[qinvokable]
        fn run(self: Pin<&mut Backend>, args: &QString);

        #[qinvokable]
        fn idle_change(self: Pin<&mut Backend>, action: &QString, value: &QString);

        #[qinvokable]
        fn idle_start(self: Pin<&mut Backend>);

        #[qsignal]
        fn finished(self: Pin<&mut Backend>, ok: bool, command: QString, output: QString);

        #[qinvokable]
        fn wall_library(self: Pin<&mut Backend>, text: &QString, colour: &QString) -> QString;

        #[qinvokable]
        fn wall_online(self: &Backend, channel: &QString) -> QString;

        #[qinvokable]
        fn wall_explore(self: &Backend) -> QString;

        #[qinvokable]
        fn wall_more(self: Pin<&mut Backend>, channel: &QString);

        #[qinvokable]
        fn wall_full(self: Pin<&mut Backend>, key: &QString, width: i32);

        #[qinvokable]
        fn wall_stamp(self: &Backend, channel: &QString) -> i32;

        #[qinvokable]
        fn wall_hide(self: Pin<&mut Backend>, key: &QString);

        #[qinvokable]
        fn wall_add(self: Pin<&mut Backend>, files: &QString) -> QString;

        #[qsignal]
        fn wall_full_ready(self: Pin<&mut Backend>, key: QString, path: QString);

        #[qinvokable]
        fn wall_owner(self: &Backend) -> QString;

        #[qinvokable]
        fn wall_scan(self: Pin<&mut Backend>);

        #[qinvokable]
        fn wall_search(self: Pin<&mut Backend>, channel: &QString, request: &QString);

        #[qinvokable]
        fn wall_apply(self: Pin<&mut Backend>, request: &QString);

        #[qinvokable]
        fn wall_folder(self: Pin<&mut Backend>, folder: &QString);

        #[qinvokable]
        fn wall_allow(self: Pin<&mut Backend>, group: &QString, on: bool);

        #[qinvokable]
        fn wall_allow_restart(self: Pin<&mut Backend>, tool: &QString);

        #[qinvokable]
        fn wall_colours(self: Pin<&mut Backend>, generator: &QString, on: bool);

        #[qinvokable]
        fn wall_lock_too(self: Pin<&mut Backend>, path: &QString) -> QString;

        #[qsignal]
        fn wall_progress(self: Pin<&mut Backend>, text: QString, image: QString);

        #[qsignal]
        fn wall_done(self: Pin<&mut Backend>, ok: bool, text: QString, path: QString);

        #[qsignal]
        fn wall_consent(self: Pin<&mut Backend>, tool: QString, request: QString);
    }

    impl cxx_qt::Threading for Backend {}
}

use std::pin::Pin;
use std::process::{Command, Stdio};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use darwan_core::catalog::Catalog;
use darwan_core::config::UserConfig;
use darwan_core::environment::Environment;
use darwan_core::paths::{self, Paths};
use darwan_core::settings::{self, Key};
use darwan_core::{host, ini, resolve};

use crate::{idle, model, walls};

pub struct BackendRust {
    paths: Paths,
    catalog: Catalog,
    // The draft the preview shows; `saved` is what is on disk, which the lock, SDDM and every command read.
    config: UserConfig,
    saved: UserConfig,
    // hypridle.conf as the draft has it, when that differs from the file; written on Save like the rest.
    idle_draft: Option<String>,
    // Whether hypridle is installed and running: asking costs processes (~25 ms), so it is asked on load, after a
    // save or a screensaver action, and on Check again; not on every revision.
    idle_setup: darwan_core::hypridle::Setup,
    env: Environment,
    revision: i32,
    busy: bool,
    dirty: bool,
    status: QString,
    // A confirmation, which fades; a problem stays until something replaces it.
    status_ok: bool,
    runtime_dir: QString,
    overlay_path: QString,
    user_name: QString,
    host_name: QString,
    sessions: QString,
    icon_path: QString,
    wall_revision: i32,
    wall_feed_revision: i32,
    wall_busy: bool,
    walls: walls::State,
}

impl Default for BackendRust {
    fn default() -> Self {
        let paths = Paths::detect();
        let mut this = Self {
            runtime_dir: QString::from(paths.runtime().display().to_string()),
            overlay_path: QString::from(paths::state_dir().join("gui.conf").display().to_string()),
            user_name: QString::from(host::user_name()),
            host_name: QString::from(host::host_name()),
            sessions: QString::from(host::sessions_json()),
            icon_path: QString::from(icon(&paths).display().to_string()),
            paths,
            catalog: Catalog::default(),
            config: UserConfig::default(),
            saved: UserConfig::default(),
            idle_draft: None,
            idle_setup: darwan_core::hypridle::setup(),
            env: environment(),
            revision: 0,
            wall_revision: 0,
            wall_feed_revision: 0,
            wall_busy: false,
            walls: walls::State::default(),
            busy: false,
            dirty: false,
            status: QString::default(),
            status_ok: true,
        };
        if let Err(e) = this.load() {
            this.status = QString::from(e);
            this.status_ok = false;
        }
        this
    }
}

// The GUI runs inside the Wayland session its actions use.
fn environment() -> Environment {
    Environment::detect(Ok(()))
}

impl BackendRust {
    fn load(&mut self) -> Result<(), String> {
        let themes = self.paths.themes();
        let (catalog, _) =
            Catalog::load(&themes).map_err(|e| format!("{}: {e}", themes.display()))?;
        let path = paths::config_file();
        let config = UserConfig::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        (self.catalog, self.saved, self.config) = (catalog, config.clone(), config);
        self.idle_setup = darwan_core::hypridle::setup();
        Ok(())
    }
}

// Every save may have changed the lock theme or its background; `darwan prepare-media` has nothing to do otherwise.
fn write(config: &UserConfig) -> Result<(), String> {
    let path = paths::config_file();
    config
        .save(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let darwan = std::env::current_exe()
        .map(|exe| exe.with_file_name("darwan"))
        .unwrap_or_else(|_| "darwan".into());
    darwan_core::media::spawn_prepare(&darwan);
    Ok(())
}

// hypridle reads its file once, so a running one is restarted to pick the change up.
fn write_idle(text: &str) -> Result<(), String> {
    let path = darwan_core::hypridle::config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    if darwan_core::hypridle::setup().running {
        idle::restart()?;
    }
    Ok(())
}

// The app icon: a checkout's source, else where the package installs it.
fn icon(paths: &Paths) -> std::path::PathBuf {
    let checkout = paths.data.join("packaging/arch/darwan.svg");
    if checkout.is_file() {
        checkout
    } else {
        "/usr/share/icons/hicolor/scalable/apps/darwan.svg".into()
    }
}

fn json(v: serde_json::Value) -> QString {
    QString::from(v.to_string())
}

impl qobject::Backend {
    fn details(&self, id: &QString) -> QString {
        let r = self.rust();
        match r.catalog.get(&id.to_string()) {
            Some(t) => json(model::details(t, &r.config)),
            None => QString::from("null"),
        }
    }

    fn fields(&self, id: &QString) -> QString {
        json(self.field_list(&id.to_string()))
    }

    fn field_list(&self, id: &str) -> serde_json::Value {
        let r = self.rust();
        match r.catalog.get(id) {
            Some(t) => {
                let host = darwan_core::system::SystemHost::new();
                let overlay = resolve::resolve(t, &r.config, &host).overlay;
                let image = resolve::image_palette(t, &r.config, &host);
                model::fields(t, &r.config, &overlay, image.as_ref())
            }
            None => serde_json::json!([]),
        }
    }

    fn wall(&self, query: &QString, filter: &QString) -> QString {
        let r = self.rust();
        json(model::wall(
            &r.catalog,
            &r.config,
            &query.to_string(),
            &filter.to_string(),
        ))
    }

    fn form(&self, id: &QString) -> QString {
        json(model::form(&self.field_list(&id.to_string())))
    }

    // Global settings are shown as a theme applies them; with no theme open, as the lock theme does.
    fn globals(&self, id: &QString) -> QString {
        let r = self.rust();
        let id = Some(id.to_string())
            .filter(|id| r.catalog.get(id).is_some())
            .or_else(|| {
                r.config
                    .theme(darwan_core::config::Target::Lock)
                    .ok()
                    .flatten()
                    .map(str::to_string)
            })
            .or_else(|| r.catalog.themes().first().map(|t| t.id.clone()))
            .unwrap_or_default();
        json(model::globals(&self.field_list(&id)))
    }

    fn health(&self) -> QString {
        let r = self.rust();
        let conf = darwan_core::hypridle::read();
        let panel = idle::panel(&r.idle_setup, conf.as_ref(), &r.saved);
        json(model::health(
            &r.env,
            &r.catalog,
            panel["problem"].as_str().unwrap_or(""),
            &r.idle_setup
                .shells
                .iter()
                .map(|s| s.name)
                .collect::<Vec<_>>(),
        ))
    }

    // A setting as the draft has it, empty when unset; for settings the GUI itself follows, like gui.look.
    fn value(&self, key: &QString) -> QString {
        let value = Key::parse(&key.to_string())
            .ok()
            .and_then(|key| settings::get(&self.rust().config, &key).ok().flatten());
        QString::from(value.unwrap_or_default())
    }

    // The draft's view, so a change shows before it is saved.
    fn saver_panel(&self) -> QString {
        let r = self.rust();
        let conf = match &r.idle_draft {
            Some(text) => Some(darwan_core::hypridle::parse(text)),
            None => darwan_core::hypridle::read(),
        };
        json(idle::panel(&r.idle_setup, conf.as_ref(), &r.config))
    }

    // Every change the Screensaver panel makes: darwan's own saver settings, hypridle's, and starting hypridle.
    fn saver_do(self: Pin<&mut Self>, action: &QString, value: &QString) {
        let on = value.to_string() == "true";
        match action.to_string().as_str() {
            "lock" => self.set_value(&QString::from("saver.lock_after"), value),
            "returnAfter" => self.set_value(&QString::from("saver.return_after"), value),
            "screenOffLocked" => self.set_value(&QString::from("saver.screen_off_locked"), value),
            "quality" => self.set_value(&QString::from("saver.quality"), value),
            "start" => self.idle_start(),
            "check" => self.recheck_idle(),
            "saverOn" if on => self.idle_change(&QString::from("saver"), &QString::from("")),
            "saverOn" => self.idle_change(&QString::from("disable"), &QString::from("")),
            other => self.idle_change(&QString::from(other), value),
        }
    }

    fn availability(&self) -> QString {
        json(model::availability(&self.rust().env))
    }

    // What "use my desktop wallpaper" would show, so the GUI can name it before it's chosen.
    fn desktop_wallpaper(&self) -> QString {
        use darwan_core::custom::Host;
        let found = darwan_core::system::SystemHost::new().desktop_wallpaper(None);
        json(match found {
            Some(w) => {
                serde_json::json!({ "path": w.path.display().to_string(), "source": w.source })
            }
            None => serde_json::Value::Null,
        })
    }

    fn report(&self, output: &QString) -> QString {
        json(model::report(&output.to_string()))
    }

    fn set_value(self: Pin<&mut Self>, key: &QString, value: &QString) {
        self.change(key, Some(&value.to_string()));
    }

    fn reset_value(self: Pin<&mut Self>, key: &QString) {
        self.change(key, None);
    }

    fn change(mut self: Pin<&mut Self>, key: &QString, value: Option<&str>) {
        let result = Key::parse(&key.to_string()).and_then(|key| {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            match value {
                Some(v) => settings::set(&mut r.config, &r.catalog, &key, v)
                    .map(|()| format!("{key} = {v} (not saved yet)")),
                None if settings::unset(&mut r.config, &key) => {
                    Ok(format!("{key} is back to its default (not saved yet)"))
                }
                None => Ok(format!("{key} was already the default")),
            }
        });
        let ok = result.is_ok();
        let status = result.unwrap_or_else(|e| e);
        self.as_mut().set_status(QString::from(status), ok);
        self.as_mut().sync_dirty();
        self.bump();
    }

    // Into the draft like any change, so Discard brings the theme's settings back.
    fn reset_theme(mut self: Pin<&mut Self>, id: &QString) {
        let id = id.to_string();
        let removed = {
            let mut r = self.as_mut().rust_mut();
            r.config.remove_theme(&id)
        };
        let status = if removed {
            format!("every setting of {id} is back to its default (not saved yet)")
        } else {
            format!("{id} has no settings to reset")
        };
        self.as_mut().set_status(QString::from(status), true);
        self.as_mut().sync_dirty();
        self.bump();
    }

    // An action rather than a customisation: written at once, into the saved file and the draft alike.
    fn set_value_now(mut self: Pin<&mut Self>, key: &QString, value: &QString) {
        let value = value.to_string();
        let result = Key::parse(&key.to_string()).and_then(|key| {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            settings::set(&mut r.saved, &r.catalog, &key, &value)?;
            settings::set(&mut r.config, &r.catalog, &key, &value)?;
            write(&r.saved).map(|()| format!("saved: {key} = {value}"))
        });
        let ok = result.is_ok();
        let status = result.unwrap_or_else(|e| e);
        self.as_mut().set_status(QString::from(status), ok);
        self.as_mut().sync_dirty();
        self.bump();
    }

    fn save_changes(mut self: Pin<&mut Self>) -> bool {
        let result = {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            write(&r.config)
                .map(|()| r.saved = r.config.clone())
                .and_then(|()| match r.idle_draft.take() {
                    Some(text) => write_idle(&text).inspect_err(|_| r.idle_draft = Some(text)),
                    None => Ok(()),
                })
        };
        let ok = result.is_ok();
        let status = result.map_or_else(|e| e, |()| "saved".to_string());
        self.as_mut().rust_mut().idle_setup = darwan_core::hypridle::setup();
        self.as_mut().set_status(QString::from(status), ok);
        self.as_mut().sync_dirty();
        self.bump();
        ok
    }

    fn discard_changes(mut self: Pin<&mut Self>) {
        {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            r.config = r.saved.clone();
            r.idle_draft = None;
        }
        self.as_mut()
            .set_status(QString::from("unsaved changes discarded"), true);
        self.as_mut().sync_dirty();
        self.bump();
    }

    fn sync_dirty(mut self: Pin<&mut Self>) {
        let dirty = {
            let r = self.rust();
            r.config.to_string() != r.saved.to_string() || r.idle_draft.is_some()
        };
        if dirty != self.rust().dirty {
            self.as_mut().rust_mut().dirty = dirty;
            self.dirty_changed();
        }
    }

    // Into the draft, like every other setting: hypridle.conf is written and hypridle restarted on Save.
    fn idle_change(mut self: Pin<&mut Self>, action: &QString, value: &QString) {
        let on_disk = std::fs::read_to_string(darwan_core::hypridle::config_path()).ok();
        let current = self.rust().idle_draft.clone().or_else(|| on_disk.clone());
        let result = idle::apply(
            current.as_deref(),
            &action.to_string(),
            &value.to_string(),
            self.rust().idle_setup.lua,
        );
        let ok = result.is_ok();
        let status = match result {
            Ok(text) => {
                self.as_mut().rust_mut().idle_draft =
                    (Some(&text) != on_disk.as_ref()).then_some(text);
                "hypridle settings changed (not saved yet)".to_string()
            }
            Err(e) => e,
        };
        self.as_mut().set_status(QString::from(status), ok);
        self.as_mut().sync_dirty();
        self.bump();
    }

    fn recheck_idle(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().idle_setup = darwan_core::hypridle::setup();
        self.bump();
    }

    fn idle_start(mut self: Pin<&mut Self>) {
        let result = idle::start().map(|()| "hypridle started".to_string());
        self.as_mut().rust_mut().idle_setup = darwan_core::hypridle::setup();
        let ok = result.is_ok();
        let status = result.unwrap_or_else(|e| e);
        self.as_mut().set_status(QString::from(status), ok);
        self.bump();
    }

    fn bump(mut self: Pin<&mut Self>) {
        let next = self.rust().revision.wrapping_add(1);
        self.as_mut().rust_mut().revision = next;
        self.revision_changed();
    }

    fn set_status(mut self: Pin<&mut Self>, status: QString, ok: bool) {
        if self.rust().status_ok != ok {
            self.as_mut().rust_mut().status_ok = ok;
            self.as_mut().status_ok_changed();
        }
        self.as_mut().rust_mut().status = status;
        self.status_changed();
    }

    fn set_busy(mut self: Pin<&mut Self>, busy: bool) {
        self.as_mut().rust_mut().busy = busy;
        self.busy_changed();
    }

    // Lenient like `darwan preview`: settings the resolver rejects are left out and reported.
    fn write_overlay(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let r = self.rust();
        let Some(theme) = r.catalog.get(&id.to_string()) else {
            return false;
        };
        let resolved = resolve::resolve(theme, &r.config, &darwan_core::system::SystemHost::new());
        let path = r.overlay_path.to_string();
        let written = ini::write_general(&resolved.overlay)
            .map_err(|e| e.to_string())
            .and_then(|text| {
                std::fs::create_dir_all(paths::state_dir())
                    .and_then(|()| std::fs::write(&path, text))
                    .map_err(|e| format!("{path}: {e}"))
            });
        let problem = match (&written, resolved.issues.first()) {
            (Err(e), _) => Some(e.clone()),
            (Ok(()), Some(issue)) => Some(format!("ignoring {}: {}", issue.key, issue.message)),
            (Ok(()), None) => None,
        };
        if let Some(p) = problem {
            self.as_mut().set_status(QString::from(p), false);
        }
        written.is_ok()
    }

    fn run(mut self: Pin<&mut Self>, args: &QString) {
        if self.rust().busy {
            return;
        }
        let args: Vec<String> = match serde_json::from_str(&args.to_string()) {
            Ok(a) => a,
            Err(e) => {
                self.as_mut()
                    .set_status(QString::from(format!("bad command: {e}")), false);
                return;
            }
        };
        let command = format!("darwan {}", args.join(" "));
        self.as_mut().set_busy(true);
        self.as_mut()
            .set_status(QString::from(format!("running {command}")), true);
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let (ok, output) = match Command::new(model::darwan_exe())
                .args(&args)
                .stdin(Stdio::null())
                .output()
            {
                Ok(o) => (
                    o.status.success(),
                    format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    ),
                ),
                Err(e) => (false, format!("cannot run darwan: {e}")),
            };
            // Only fails when the window is already gone, and then nobody is left to tell.
            let _ = thread.queue(move |mut qobject| {
                let reloaded = {
                    let mut r = qobject.as_mut().rust_mut();
                    r.env = environment();
                    r.load()
                };
                let status = match (&reloaded, ok) {
                    (Err(e), _) => e.clone(),
                    (Ok(()), true) => format!("done: {command}"),
                    (Ok(()), false) => format!("failed: {command}"),
                };
                qobject
                    .as_mut()
                    .set_status(QString::from(status), ok && reloaded.is_ok());
                qobject.as_mut().sync_dirty();
                qobject.as_mut().set_busy(false);
                qobject.as_mut().bump();
                qobject
                    .as_mut()
                    .finished(ok, QString::from(&command), QString::from(&output));
            });
        });
    }
}

fn home() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

fn allowed(config: &UserConfig) -> darwan_core::wallpaper::filter::Allowed {
    darwan_core::wallpaper::filter::Allowed(
        config.wallpaper_allow().ok().flatten().unwrap_or_default(),
    )
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

type Thread = cxx_qt::CxxQtThread<qobject::Backend>;

// Thumbnails and facts for every wallpaper, a few at a time, handed to the GUI in batches so the grid fills without a
// redraw per file. Stops when a newer scan starts.
fn prepare(
    items: Vec<darwan_core::wallpaper::library::Item>,
    ticket: u64,
    cancel: std::sync::Arc<std::sync::atomic::AtomicU64>,
    thread: Thread,
) {
    use darwan_core::wallpaper::library::{self, Size};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::{Duration, Instant};

    let cache_home = paths::cache_home();
    let facts_dir = paths::cache_dir().join("wallpapers");
    let workers = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).clamp(1, 4));
    let next = AtomicUsize::new(0);
    let (tx, rx) = mpsc::channel();
    let flush = |batch: Vec<(std::path::PathBuf, std::path::PathBuf, library::Facts)>| {
        let _ = thread.queue(move |mut q| {
            if q.rust().walls.scan.load(Ordering::Relaxed) != ticket {
                return;
            }
            {
                let mut r = q.as_mut().rust_mut();
                for (path, thumb, facts) in batch {
                    r.walls.ready.insert(path, (thumb, facts));
                }
            }
            q.as_mut().wall_bump();
        });
    };
    std::thread::scope(|s| {
        for _ in 0..workers {
            let tx = tx.clone();
            let (items, next, cancel, cache_home, facts_dir) =
                (&items, &next, &cancel, &cache_home, &facts_dir);
            s.spawn(move || {
                while cancel.load(Ordering::Relaxed) == ticket {
                    let Some(item) = items.get(next.fetch_add(1, Ordering::Relaxed)) else {
                        break;
                    };
                    if let Ok(thumb) = library::thumbnail(cache_home, item, Size::XLarge)
                        && let Ok(facts) = library::facts(facts_dir, item, &thumb)
                    {
                        let _ = tx.send((item.path.clone(), thumb, facts));
                    }
                }
            });
        }
        drop(tx);
        let mut batch = Vec::new();
        let mut sent = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_millis(120)) {
                Ok(done) => batch.push(done),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if !batch.is_empty() && sent.elapsed() > Duration::from_millis(250) {
                flush(std::mem::take(&mut batch));
                sent = Instant::now();
            }
        }
        if !batch.is_empty() {
            flush(batch);
        }
    });
}

impl qobject::Backend {
    // The Library, what draws the wallpaper, the switches changed: announced within ~120 ms, once however many
    // changes came meanwhile.
    fn wall_bump(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().walls.library_json = None;
        if self.rust().walls.library_pending {
            return;
        }
        self.as_mut().rust_mut().walls.library_pending = true;
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(120));
            let _ = thread.queue(|mut q| {
                q.as_mut().rust_mut().walls.library_pending = false;
                let n = q.rust().wall_revision.wrapping_add(1);
                q.as_mut().rust_mut().wall_revision = n;
                q.as_mut().wall_revision_changed();
            });
        });
    }

    // Online results and their thumbnails: the same, on their own revision, so the Library isn't read again.
    fn feed_bump(mut self: Pin<&mut Self>) {
        if self.rust().walls.feed_pending {
            return;
        }
        self.as_mut().rust_mut().walls.feed_pending = true;
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(120));
            let _ = thread.queue(|mut q| {
                q.as_mut().rust_mut().walls.feed_pending = false;
                let n = q.rust().wall_feed_revision.wrapping_add(1);
                q.as_mut().rust_mut().wall_feed_revision = n;
                q.as_mut().wall_feed_revision_changed();
            });
        });
    }

    fn refresh_owner(mut self: Pin<&mut Self>) {
        let text = {
            let r = self.rust();
            let env = darwan_core::wallpaper::Env::system();
            let found = darwan_core::wallpaper::colours::found(&env, r.walls.owner.as_ref());
            walls::owner(
                r.walls.owner.as_ref(),
                &r.config.wallpaper_restart(),
                &found,
                &r.config.wallpaper_colours(),
            )
            .to_string()
        };
        self.as_mut().rust_mut().walls.owner_json = text;
    }

    fn wall_hide(mut self: Pin<&mut Self>, key: &QString) {
        {
            let mut r = self.as_mut().rust_mut();
            r.walls.hidden.insert(key.to_string());
            for c in r.walls.channels.values_mut() {
                c.stamp = c.stamp.wrapping_add(1);
            }
        }
        self.as_mut().feed_bump();
        self.wall_bump();
    }

    fn set_wall_busy(mut self: Pin<&mut Self>, busy: bool) {
        self.as_mut().rust_mut().wall_busy = busy;
        self.wall_busy_changed();
    }

    // Read at every revision, so the whole Library is made into JSON once per change, not once per read.
    fn wall_library(mut self: Pin<&mut Self>, text: &QString, colour: &QString) -> QString {
        let plain = text.is_empty() && colour.is_empty();
        if plain && let Some(cached) = &self.rust().walls.library_json {
            return QString::from(cached);
        }
        let made = {
            let w = &self.rust().walls;
            let view = walls::LibraryView {
                folder: &w.folder,
                items: &w.items,
                ready: &w.ready,
                credits: &w.credits,
                current: w.current.as_deref(),
                now: now(),
                hidden: &w.hidden,
            };
            walls::library(&view, &text.to_string(), &colour.to_string()).to_string()
        };
        if plain {
            self.as_mut().rust_mut().walls.library_json = Some(made.clone());
        }
        QString::from(made)
    }

    fn wall_online(&self, channel: &QString) -> QString {
        let w = &self.rust().walls;
        json(walls::online(
            w.channels.get(&channel.to_string()),
            &w.thumbs,
            &w.folder,
            &w.hidden,
        ))
    }

    fn wall_explore(&self) -> QString {
        json(walls::explore(&allowed(&self.rust().config)))
    }

    fn wall_owner(&self) -> QString {
        let text = &self.rust().walls.owner_json;
        QString::from(if text.is_empty() { "null" } else { text })
    }

    fn wall_colours(mut self: Pin<&mut Self>, generator: &QString, on: bool) {
        let id = generator.to_string();
        let mut list = self.rust().config.wallpaper_colours();
        list.retain(|g| g != &id);
        if on {
            list.push(id);
        }
        self.as_mut().set_value_now(
            &QString::from("wallpaper.colours"),
            &QString::from(list.join(",")),
        );
        self.as_mut().refresh_owner();
        self.wall_bump();
    }

    // The folder, what draws the wallpaper and what it shows now; then thumbnails and colours in the background.
    fn wall_scan(mut self: Pin<&mut Self>) {
        use darwan_core::wallpaper::{Env, library, online, set};
        use std::sync::atomic::Ordering;
        let folder = library::folder(&self.rust().config, &home(), &paths::config_home());
        let items = library::scan(&folder);
        let credits_dir = paths::data_dir().join("credits");
        let credits = items
            .iter()
            .filter_map(|i| online::credit(&credits_dir, &i.path).map(|c| (i.path.clone(), c)))
            .collect();
        let env = Env::system();
        let owner = set::owner(&env);
        let current = owner.as_ref().and_then(|o| set::current(&env, o, None));
        let (ticket, cancel) = {
            let mut r = self.as_mut().rust_mut();
            let w = &mut r.walls;
            w.folder = folder;
            w.ready.retain(|p, _| items.iter().any(|i| &i.path == p));
            w.items = items.clone();
            w.credits = credits;
            w.owner = owner;
            w.current = current;
            let ticket = w.scan.fetch_add(1, Ordering::Relaxed) + 1;
            (ticket, w.scan.clone())
        };
        self.as_mut().refresh_owner();
        self.as_mut().wall_bump();
        let thread = self.qt_thread();
        std::thread::spawn(move || prepare(items, ticket, cancel, thread));
    }

    // A channel starts over: {"sources": [...] (every one when empty), "text", "topic", "sort", "ratio", "first"}.
    // The first 50 results by default, each shown as it passes.
    fn wall_search(mut self: Pin<&mut Self>, channel: &QString, request: &QString) {
        use darwan_core::wallpaper::online::{Query, Sort, Source};
        let v: serde_json::Value = serde_json::from_str(&request.to_string()).unwrap_or_default();
        let text = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        let mut sources: Vec<Source> = v
            .get("sources")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().and_then(Source::parse))
                    .collect()
            })
            .unwrap_or_default();
        if sources.is_empty() {
            sources = Source::ALL.to_vec();
        }
        let query = Query {
            text: text("text"),
            sort: match text("sort").as_str() {
                "latest" => Sort::Latest,
                "random" => Sort::Random,
                _ => Sort::Popular,
            },
            ratio: Some(text("ratio")).filter(|r| !r.is_empty()),
            topic: Some(text("subject")).filter(|t| !t.is_empty()),
            ..Query::default()
        };
        let feeds = walls::feeds(&sources, &query, &text("topic"));
        let first = v
            .get("first")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(50) as usize;
        let name = channel.to_string();
        {
            let mut r = self.as_mut().rust_mut();
            let old = r.walls.channels.get(&name).map(|c| c.ticket.clone());
            let mut c = walls::Channel::new(feeds, first);
            c.stamp = r
                .walls
                .channels
                .get(&name)
                .map_or(0, |o| o.stamp.wrapping_add(1));
            if let Some(t) = old {
                // The same counter, bumped: a worker for the old query stops.
                t.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                c.ticket = t;
            }
            r.walls.channels.insert(name.clone(), c);
        }
        self.as_mut().feed_bump();
        self.fetch_channel(name);
    }

    // 25 more for a channel scrolled to its end; fetched only when what's held runs short.
    fn wall_more(mut self: Pin<&mut Self>, channel: &QString) {
        let name = channel.to_string();
        let fetch = {
            let mut r = self.as_mut().rust_mut();
            let Some(c) = r.walls.channels.get_mut(&name) else {
                return;
            };
            if c.loading || (!c.more() && c.items.len() <= c.target) {
                return;
            }
            c.target += 25;
            c.stamp = c.stamp.wrapping_add(1);
            c.items.len() < c.target && c.more()
        };
        self.as_mut().feed_bump();
        if fetch {
            self.fetch_channel(name);
        }
    }

    // Pages in the background until the channel holds its target or the source runs out; each result is shown as
    // soon as it passes, and thumbnails load four at a time.
    fn fetch_channel(mut self: Pin<&mut Self>, name: String) {
        use darwan_core::wallpaper::online::Client;
        use std::sync::atomic::Ordering;
        let allowed = allowed(&self.rust().config);
        let started = {
            let mut r = self.as_mut().rust_mut();
            r.walls.channels.get_mut(&name).map(|c| {
                c.loading = true;
                let t = c.ticket.load(Ordering::Relaxed);
                (
                    c.feeds.clone(),
                    c.target,
                    c.items.len(),
                    (t, c.ticket.clone()),
                )
            })
        };
        let (mut feeds, target, have, ticket) = match started {
            Some(s) => s,
            None => return,
        };
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let (mine, counter) = ticket;
            let live = || counter.load(Ordering::Relaxed) == mine;
            let client = std::sync::Arc::new(Client::new(paths::cache_dir().join("online")));
            let slots =
                std::sync::Arc::new((std::sync::Mutex::new(0usize), std::sync::Condvar::new()));
            let mut count = have;
            let mut problem: Result<String, String> = Ok(String::new());
            // One page from each source in turn, so every source shows early.
            // Results only ever join at the end, so nothing moves above what's on screen: a mixed round's quick
            // sources are interleaved once and appended together; Wallhaven, checking each picture, streams after them.
            let push = |batch: Vec<darwan_core::wallpaper::online::Found>| {
                let n = name.clone();
                let _ = thread.queue(move |mut qo| {
                    let mut added = false;
                    if let Some(c) = qo.as_mut().rust_mut().walls.channels.get_mut(&n)
                        && c.ticket.load(Ordering::Relaxed) == mine
                    {
                        for f in batch {
                            if !c.items.iter().any(|x| x.key() == f.key()) {
                                c.items.push(f);
                                added = true;
                            }
                        }
                        if added {
                            c.stamp = c.stamp.wrapping_add(1);
                        }
                    }
                    if added {
                        qo.as_mut().feed_bump();
                    }
                });
            };
            let mixing = feeds.len() > 1;
            'pages: while live() && count < target && feeds.iter().any(|f| f.more) {
                let mut round: Vec<darwan_core::wallpaper::online::Found> = Vec::new();
                let mut stop = false;
                for feed in feeds.iter_mut().filter(|f| f.more) {
                    if !live() || count >= target {
                        stop = true;
                        break;
                    }
                    let buffered =
                        mixing && feed.source != darwan_core::wallpaper::online::Source::Wallhaven;
                    if !buffered && !round.is_empty() {
                        push(walls::mixed(&round).into_iter().cloned().collect());
                        round.clear();
                    }
                    let mut q = feed.query.clone();
                    q.page = feed.next_page;
                    let result = client.search_each(feed.source, &q, &allowed, &mut |found| {
                        if !live() {
                            return;
                        }
                        count += 1;
                        if buffered {
                            round.push(found.clone());
                        } else {
                            push(vec![found.clone()]);
                        }
                        // At most four thumbnails at once; a cached one is only a file check.
                        let (client, slots, thread, found) =
                            (client.clone(), slots.clone(), thread.clone(), found.clone());
                        {
                            let (lock, cv) = &*slots;
                            let mut busy = lock.lock().unwrap();
                            while *busy >= 4 {
                                busy = cv.wait(busy).unwrap();
                            }
                            *busy += 1;
                        }
                        std::thread::spawn(move || {
                            if let Ok(thumb) = client.thumbnail(&found) {
                                let key = found.key();
                                let print =
                                    std::fs::read(&thumb).map(|b| walls::print(&b)).unwrap_or(0);
                                let _ = thread.queue(move |mut qo| {
                                    {
                                        let mut r = qo.as_mut().rust_mut();
                                        let w = &mut r.walls;
                                        // Two pictures with the very same thumbnail are a placeholder, not wallpapers.
                                        match w.thumb_prints.get(&print) {
                                            Some(other) if other != &key && print != 0 => {
                                                let other = other.clone();
                                                w.hidden.insert(other);
                                                w.hidden.insert(key.clone());
                                            }
                                            _ => {
                                                w.thumb_prints.insert(print, key.clone());
                                            }
                                        }
                                        for c in w.channels.values_mut() {
                                            if c.items.iter().any(|f| f.key() == key) {
                                                c.stamp = c.stamp.wrapping_add(1);
                                            }
                                        }
                                        w.thumbs.insert(key, thumb);
                                    }
                                    qo.as_mut().feed_bump();
                                });
                            }
                            let (lock, cv) = &*slots;
                            *lock.lock().unwrap() -= 1;
                            cv.notify_one();
                        });
                    });
                    feed.next_page += 1;
                    match result {
                        Ok(p) if p.refused.is_some() => {
                            problem = Ok(p.refused.unwrap_or_default());
                            feeds.iter_mut().for_each(|f| f.more = false);
                            break 'pages;
                        }
                        Ok(p) => feed.more = p.more,
                        // One source down doesn't stop the others.
                        Err(e) => {
                            feed.more = false;
                            problem = Err(e);
                        }
                    }
                }
                if !round.is_empty() {
                    push(walls::mixed(&round).into_iter().cloned().collect());
                }
                if stop {
                    break;
                }
            }
            let _ = thread.queue(move |mut qo| {
                {
                    let mut r = qo.as_mut().rust_mut();
                    let Some(c) = r.walls.channels.get_mut(&name) else {
                        return;
                    };
                    if c.ticket.load(Ordering::Relaxed) != mine {
                        return;
                    }
                    c.loading = false;
                    c.feeds = feeds;
                    c.stamp = c.stamp.wrapping_add(1);
                    match problem {
                        Ok(refused) => c.refused = refused,
                        Err(e) if c.items.is_empty() => c.error = e,
                        Err(_) => {}
                    }
                }
                qo.as_mut().feed_bump();
            });
        });
    }

    // The picture at screen size for the preview or the background, as a JPEG made once (see `library::preview`):
    // `key` is a Library path, or an online result's key, which is fetched into the cache first. A video gives its
    // largest thumbnail.
    fn wall_full(mut self: Pin<&mut Self>, key: &QString, width: i32) {
        use darwan_core::wallpaper::library;
        use darwan_core::wallpaper::online::Client;
        let key = key.to_string();
        let width = width.clamp(640, 3840) as u32;
        let slot = format!("{key}@{width}");
        if let Some(path) = self.rust().walls.full.get(&slot).cloned() {
            self.as_mut().wall_full_ready(
                QString::from(key),
                QString::from(format!("file://{}", path.display())),
            );
            return;
        }
        let online = if key.starts_with('/') {
            None
        } else {
            self.rust().walls.find(&key).cloned()
        };
        if online.is_none() && !key.starts_with('/') {
            return;
        }
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let previews = paths::cache_dir().join("previews");
            let made = match &online {
                Some(found) => Client::new(paths::cache_dir().join("online"))
                    .full(found)
                    .and_then(|file| library::preview(&previews, &file, width)),
                None => {
                    let file = std::path::PathBuf::from(&key);
                    match library::item(&file) {
                        Some(item) if item.kind == library::Kind::Video => {
                            library::thumbnail(&paths::cache_home(), &item, library::Size::XxLarge)
                        }
                        _ => library::preview(&previews, &file, width),
                    }
                }
            };
            if let Ok(path) = made {
                let _ = thread.queue(move |mut q| {
                    q.as_mut().rust_mut().walls.full.insert(slot, path.clone());
                    q.as_mut().wall_full_ready(
                        QString::from(key),
                        QString::from(format!("file://{}", path.display())),
                    );
                });
            }
        });
    }

    fn wall_stamp(&self, channel: &QString) -> i32 {
        self.rust()
            .walls
            .channels
            .get(&channel.to_string())
            .map_or(-1, |c| c.stamp as i32)
    }

    // "+": pictures and videos copied into the wallpaper folder; returns what happened, for the notice.
    fn wall_add(self: Pin<&mut Self>, files: &QString) -> QString {
        let list: Vec<String> = serde_json::from_str(&files.to_string()).unwrap_or_default();
        let folder = self.rust().walls.folder.clone();
        if let Err(e) = std::fs::create_dir_all(&folder) {
            return QString::from(format!("{}: {e}", folder.display()));
        }
        let mut added = 0;
        for f in &list {
            let src = std::path::PathBuf::from(f.strip_prefix("file://").unwrap_or(f));
            let Some(name) = src.file_name() else {
                continue;
            };
            if darwan_core::wallpaper::library::Kind::of(&src).is_none() {
                continue;
            }
            let target = folder.join(name);
            if target.exists() || std::fs::copy(&src, &target).is_ok() {
                added += 1;
            }
        }
        self.wall_scan();
        QString::from(match added {
            0 => "Nothing added: only pictures and videos can be".to_string(),
            1 => "1 wallpaper added to your Library".to_string(),
            n => format!("{n} wallpapers added to your Library"),
        })
    }

    // {"path"} or {"online": id}, with {"outputs": chosen, "all": every screen, "restart": allowed once}.
    fn wall_apply(mut self: Pin<&mut Self>, request: &QString) {
        use darwan_core::wallpaper::Env;
        use darwan_core::wallpaper::online::Client;
        use darwan_core::wallpaper::set::{self, Request, Target};
        if self.rust().wall_busy {
            return;
        }
        let raw = request.to_string();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
        let list = |k: &str| -> Vec<String> {
            v.get(k)
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        let (chosen, all) = (list("outputs"), list("all"));
        let once = v
            .get("restart")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let (online, folder, remembered, colours) = {
            let r = self.rust();
            let online = v
                .get("online")
                .and_then(|x| x.as_str())
                .and_then(|key| r.walls.find(key).cloned());
            (
                online,
                r.walls.folder.clone(),
                r.config.wallpaper_restart(),
                r.config.wallpaper_colours(),
            )
        };
        let local = v
            .get("path")
            .and_then(|x| x.as_str())
            .map(std::path::PathBuf::from);
        self.as_mut().set_wall_busy(true);
        let thread = self.qt_thread();
        let progress = move |t: &Thread, text: String| {
            let _ = t.queue(move |mut q| {
                q.as_mut()
                    .wall_progress(QString::from(text), QString::default())
            });
        };
        std::thread::spawn(move || {
            let finish = |t: &Thread, ok: bool, text: String, path: String| {
                let _ = t.queue(move |mut q| {
                    q.as_mut().set_wall_busy(false);
                    if ok {
                        q.as_mut().rust_mut().walls.current = Some(std::path::PathBuf::from(&path));
                        q.as_mut().wall_scan();
                    }
                    q.as_mut()
                        .wall_done(ok, QString::from(text), QString::from(path));
                });
            };
            let path = match (local, &online) {
                (Some(p), _) => p,
                (None, Some(found)) => {
                    progress(&thread, "Downloading…".into());
                    let client = Client::new(paths::cache_dir().join("online"));
                    match client.download(found, &folder, &paths::data_dir().join("credits")) {
                        Ok(p) => p,
                        Err(e) => {
                            return finish(
                                &thread,
                                false,
                                format!("Download failed: {e}"),
                                String::new(),
                            );
                        }
                    }
                }
                (None, None) => {
                    return finish(&thread, false, "Nothing to set".into(), String::new());
                }
            };
            let env = Env::system();
            let Some(owner) = set::owner(&env) else {
                return finish(
                    &thread,
                    false,
                    "Nothing draws a desktop wallpaper in this session".into(),
                    String::new(),
                );
            };
            let req = Request {
                path: std::fs::canonicalize(&path).unwrap_or(path),
                target: if chosen.is_empty() || chosen.len() == all.len() {
                    Target::All
                } else {
                    Target::Outputs(chosen)
                },
                outputs: all,
            };
            let plan = match set::plan(&env, &owner, &req) {
                Ok(p) => p,
                Err(e) => return finish(&thread, false, e, String::new()),
            };
            if let Some(tool) = plan.restarts
                && !once
                && !remembered.iter().any(|t| t == tool.name())
            {
                let mut again: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
                again["path"] = serde_json::Value::from(req.path.display().to_string());
                let name = tool.name().to_string();
                let _ = thread.queue(move |mut q| {
                    q.as_mut().set_wall_busy(false);
                    q.as_mut()
                        .wall_consent(QString::from(name), QString::from(again.to_string()));
                });
                return;
            }
            progress(&thread, format!("Setting it with {}…", owner.tool.name()));
            let file = req.path.display().to_string();
            if let Err(e) = set::carry_out(&env, &owner, &req, &plan, &set::System) {
                return finish(&thread, false, e, file);
            }
            let mut text = format!("Wallpaper set with {}", owner.tool.name());
            if !colours.is_empty() {
                progress(&thread, "Making colours…".into());
                for (name, result) in darwan_core::wallpaper::colours::run_after(
                    &env,
                    Some(&owner),
                    &colours,
                    &req.path,
                    &paths::cache_home(),
                ) {
                    if let Err(e) = result {
                        text = format!("{text}; {name} failed: {e}");
                    }
                }
            }
            finish(&thread, true, text, file)
        });
    }

    fn wall_folder(mut self: Pin<&mut Self>, folder: &QString) {
        let f = folder.to_string();
        let f = f.strip_prefix("file://").unwrap_or(&f).to_string();
        self.as_mut()
            .set_value_now(&QString::from("wallpaper.folder"), &QString::from(f));
        self.wall_scan();
    }

    fn wall_allow(mut self: Pin<&mut Self>, group: &QString, on: bool) {
        let group = group.to_string();
        let mut list = self
            .rust()
            .config
            .wallpaper_allow()
            .ok()
            .flatten()
            .unwrap_or_default();
        list.retain(|g| g != &group);
        if on {
            list.push(group);
        }
        self.as_mut().set_value_now(
            &QString::from("wallpaper.allow"),
            &QString::from(list.join(",")),
        );
        // Every list asks again, with the new switches.
        let names: Vec<String> = self.rust().walls.channels.keys().cloned().collect();
        for name in names {
            let restart = {
                let mut r = self.as_mut().rust_mut();
                let c = r.walls.channels.get_mut(&name).expect("listed above");
                c.ticket.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                c.items.clear();
                c.stamp = c.stamp.wrapping_add(1);
                for f in &mut c.feeds {
                    f.next_page = 1;
                    f.more = true;
                }
                c.error.clear();
                c.refused.clear();
                true
            };
            if restart {
                self.as_mut().fetch_channel(name);
            }
        }
        self.as_mut().feed_bump();
        self.wall_bump();
    }

    fn wall_allow_restart(mut self: Pin<&mut Self>, tool: &QString) {
        let mut list = self.rust().config.wallpaper_restart();
        let tool = tool.to_string();
        if !list.contains(&tool) {
            list.push(tool);
        }
        self.as_mut().set_value_now(
            &QString::from("wallpaper.restart"),
            &QString::from(list.join(",")),
        );
        self.as_mut().refresh_owner();
        self.wall_bump();
    }

    // The lock theme's background follows the desktop wallpaper from now on, written at once like an action.
    fn wall_lock_too(mut self: Pin<&mut Self>, _path: &QString) -> QString {
        let lock = self
            .rust()
            .saved
            .theme(darwan_core::config::Target::Lock)
            .ok()
            .flatten()
            .map(str::to_string);
        let Some(id) = lock else {
            return QString::from("No lock theme is chosen yet");
        };
        let name = self
            .rust()
            .catalog
            .get(&id)
            .map_or(id.clone(), |t| t.manifest.name.clone());
        let key = QString::from(format!("{id}.background"));
        self.as_mut().set_value_now(&key, &QString::from("desktop"));
        if self.rust().status_ok {
            QString::from(format!("{name} now shows your desktop wallpaper"))
        } else {
            QString::from(format!(
                "{name} can't take a background: {}",
                self.rust().status
            ))
        }
    }
}
