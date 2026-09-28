use std::path::PathBuf;

pub const HELPER: &str = "/usr/lib/darwan/darwan-helper";

pub struct Paths {
    pub data: PathBuf,
    pub qml_modules: PathBuf,
}

impl Paths {
    // A checkout (DARWAN_DATA_DIR) uses darwan's QML plugin that `just build` puts under target/.
    pub fn detect() -> Self {
        match std::env::var_os("DARWAN_DATA_DIR") {
            Some(dir) => {
                let data = PathBuf::from(dir);
                let qml_modules = data.join("target/plugin/qml");
                Self { data, qml_modules }
            }
            None => Self {
                data: PathBuf::from("/usr/share/darwan"),
                qml_modules: PathBuf::from("/usr/lib/darwan/qml"),
            },
        }
    }

    // Themes' `import QtMultimedia` resolves to darwan's shim, which needs darwan's QML plugin next to it.
    pub fn qml_import_path(&self) -> std::ffi::OsString {
        std::env::join_paths([self.runtime().join("imports"), self.qml_modules.clone()])
            .unwrap_or_default()
    }

    pub fn runtime(&self) -> PathBuf {
        self.data.join("runtime")
    }

    pub fn themes(&self) -> PathBuf {
        self.data.join("themes")
    }
}

fn xdg(var: &str, home_fallback: &str) -> PathBuf {
    match std::env::var_os(var) {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(home_fallback),
    }
}

pub fn config_file() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("darwan/config.toml")
}

pub fn cache_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join("darwan")
}

pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join("darwan")
}
