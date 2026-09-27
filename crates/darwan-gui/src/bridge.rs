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
        #[qproperty(QString, runtime_dir, READ, CONSTANT)]
        #[qproperty(QString, overlay_path, READ, CONSTANT)]
        #[qproperty(QString, user_name, READ, CONSTANT)]
        #[qproperty(QString, host_name, READ, CONSTANT)]
        #[qproperty(QString, sessions, READ, CONSTANT)]
        type Backend = super::BackendRust;

        #[qinvokable]
        fn rows(self: &Backend, query: &QString) -> QString;

        #[qinvokable]
        fn details(self: &Backend, id: &QString) -> QString;

        #[qinvokable]
        fn fields(self: &Backend, id: &QString) -> QString;

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
        fn set_value_now(self: Pin<&mut Backend>, key: &QString, value: &QString);

        #[qinvokable]
        fn save_changes(self: Pin<&mut Backend>) -> bool;

        #[qinvokable]
        fn discard_changes(self: Pin<&mut Backend>);

        #[qinvokable]
        fn write_overlay(self: Pin<&mut Backend>, id: &QString) -> bool;

        #[qinvokable]
        fn run(self: Pin<&mut Backend>, args: &QString);

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

use crate::model;

pub struct BackendRust {
    paths: Paths,
    catalog: Catalog,
    // The draft the preview shows; `saved` is what is on disk, which the lock, SDDM and every command read.
    config: UserConfig,
    saved: UserConfig,
    env: Environment,
    revision: i32,
    busy: bool,
    dirty: bool,
    status: QString,
    runtime_dir: QString,
    overlay_path: QString,
    user_name: QString,
    host_name: QString,
    sessions: QString,
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
            paths,
            catalog: Catalog::default(),
            config: UserConfig::default(),
            saved: UserConfig::default(),
            env: environment(),
            revision: 0,
            busy: false,
            dirty: false,
            status: QString::default(),
        };
        if let Err(e) = this.load() {
            this.status = QString::from(e);
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
        Ok(())
    }
}

fn write(config: &UserConfig) -> Result<(), String> {
    let path = paths::config_file();
    config
        .save(&path)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn json(v: serde_json::Value) -> QString {
    QString::from(v.to_string())
}

impl qobject::Backend {
    fn rows(&self, query: &QString) -> QString {
        let r = self.rust();
        json(model::rows(&r.catalog, &r.config, &query.to_string()))
    }

    fn details(&self, id: &QString) -> QString {
        let r = self.rust();
        match r.catalog.get(&id.to_string()) {
            Some(t) => json(model::details(t, &r.config)),
            None => QString::from("null"),
        }
    }

    fn fields(&self, id: &QString) -> QString {
        let r = self.rust();
        match r.catalog.get(&id.to_string()) {
            Some(t) => json(model::fields(t, &r.config)),
            None => QString::from("[]"),
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
        let status = result.unwrap_or_else(|e| e);
        self.as_mut().set_status(QString::from(status));
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
        let status = result.unwrap_or_else(|e| e);
        self.as_mut().set_status(QString::from(status));
        self.as_mut().sync_dirty();
        self.bump();
    }

    fn save_changes(mut self: Pin<&mut Self>) -> bool {
        let result = {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            write(&r.config).map(|()| r.saved = r.config.clone())
        };
        let ok = result.is_ok();
        let status = result.map_or_else(|e| e, |()| "saved".to_string());
        self.as_mut().set_status(QString::from(status));
        self.as_mut().sync_dirty();
        self.bump();
        ok
    }

    fn discard_changes(mut self: Pin<&mut Self>) {
        {
            let mut r = self.as_mut().rust_mut();
            let r = &mut *r;
            r.config = r.saved.clone();
        }
        self.as_mut()
            .set_status(QString::from("unsaved changes discarded"));
        self.as_mut().sync_dirty();
        self.bump();
    }

    fn sync_dirty(mut self: Pin<&mut Self>) {
        let dirty = {
            let r = self.rust();
            r.config.to_string() != r.saved.to_string()
        };
        if dirty != self.rust().dirty {
            self.as_mut().rust_mut().dirty = dirty;
            self.dirty_changed();
        }
    }

    fn bump(mut self: Pin<&mut Self>) {
        let next = self.rust().revision.wrapping_add(1);
        self.as_mut().rust_mut().revision = next;
        self.revision_changed();
    }

    fn set_status(mut self: Pin<&mut Self>, status: QString) {
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
            self.as_mut().set_status(QString::from(p));
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
                    .set_status(QString::from(format!("bad command: {e}")));
                return;
            }
        };
        let command = format!("darwan {}", args.join(" "));
        self.as_mut().set_busy(true);
        self.as_mut()
            .set_status(QString::from(format!("running {command}")));
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
                qobject.as_mut().set_status(QString::from(status));
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
