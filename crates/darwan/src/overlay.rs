use std::io::Write;
use std::path::PathBuf;

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::paths::{self, Paths};
use darwan_core::{ini, resolve};

pub struct Prepared {
    pub theme: Theme,
    pub overlay: PathBuf,
}

// Lenient on purpose: a bad setting is dropped with a warning so the lock still starts.
pub fn prepare(paths: &Paths, id: Option<&str>, file_name: &str) -> Result<Prepared, String> {
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let id = match id {
        Some(id) => id.to_string(),
        None => config
            .theme(Target::Lock)
            .map_err(|e| format!("{}: {e}", config_path.display()))?
            .ok_or("no lock theme set: pass a theme id, or set [lock] theme in the config")?
            .to_string(),
    };

    let (catalog, problems) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let Some(pos) = catalog.themes().iter().position(|t| t.id == id) else {
        return Err(match problems.iter().find(|p| p.id == id) {
            Some(p) => format!("theme {id:?} is broken: {}", p.message),
            None => format!("unknown theme {id:?}"),
        });
    };
    let theme = catalog.into_themes().swap_remove(pos);

    let resolved = resolve::resolve(&theme, &config, &darwan_core::system::SystemHost::new());
    for issue in &resolved.issues {
        eprintln!("warning: ignoring {}: {}", issue.key, issue.message);
    }
    let text = ini::write_general(&resolved.overlay).map_err(|e| e.to_string())?;

    let dir = paths::state_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let overlay = dir.join(file_name);
    let mut tmp = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    tmp.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    tmp.persist(&overlay).map_err(|e| e.to_string())?;
    Ok(Prepared { theme, overlay })
}
