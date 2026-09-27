use std::path::Path;
use std::process::ExitCode;

use darwan_core::catalog::Catalog;
use darwan_core::paths::Paths;

use crate::sddm;

// The font is read here, as the user, and piped to the root helper, which never opens a user path.
pub fn import(
    paths: &Paths,
    id: &str,
    file: &Path,
    target: Option<&str>,
) -> Result<ExitCode, String> {
    let (catalog, _) = Catalog::load(&paths.themes()).map_err(|e| e.to_string())?;
    let theme = catalog
        .get(id)
        .ok_or_else(|| darwan_core::catalog::unknown_theme(id))?;
    let needed: Vec<&str> = theme
        .manifest
        .fonts
        .iter()
        .map(|f| f.file.as_str())
        .collect();
    let name = match (target, needed.as_slice()) {
        (_, []) => return Err(format!("{id} bundles its fonts; nothing to import")),
        (Some(t), _) if needed.contains(&t) => t,
        (Some(t), _) => {
            return Err(format!(
                "{id} does not need {t:?}; it needs {}",
                needed.join(", ")
            ));
        }
        (None, [only]) => only,
        (None, _) => {
            return Err(format!(
                "{id} needs several fonts; pass --as with one of: {}",
                needed.join(", ")
            ));
        }
    };
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    sddm::run_helper(&["font-import", id, name], &bytes)?;
    Ok(ExitCode::SUCCESS)
}
