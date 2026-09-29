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

use crate::{idle, model};

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
