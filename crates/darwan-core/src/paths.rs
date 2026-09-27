use std::path::PathBuf;

pub const HELPER: &str = "/usr/lib/darwan/darwan-helper";

pub struct Paths {
    pub data: PathBuf,
}

impl Paths {
    pub fn detect() -> Self {
        let data = std::env::var_os("DARWAN_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/usr/share/darwan"));
        Self { data }
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
