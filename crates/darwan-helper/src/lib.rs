use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use darwan_core::catalog::{self, Theme};
use darwan_core::{ini, resolve};

pub const MAX_OVERLAY_BYTES: usize = 64 * 1024;
pub const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const LINK_NAME: &str = "darwan";
const CONF_NAME: &str = "zz-darwan.conf";

pub struct Roots {
    pub themes: PathBuf,
    pub sddm_themes: PathBuf,
    pub conf_d: PathBuf,
}

impl Roots {
    pub fn system() -> Self {
        Self {
            themes: "/usr/share/darwan/themes".into(),
            sddm_themes: "/usr/share/sddm/themes".into(),
            conf_d: "/etc/sddm.conf.d".into(),
        }
    }
}

pub fn apply(roots: &Roots, id: &str, overlay_json: &[u8]) -> Result<String, String> {
    if overlay_json.len() > MAX_OVERLAY_BYTES {
        return Err("overlay is too large".into());
    }
    let overlay: BTreeMap<String, String> = serde_json::from_slice(overlay_json)
        .map_err(|e| format!("overlay is not a JSON object of strings: {e}"))?;
    let (dir, theme) = installed_theme(roots, id)?;
    let issues = resolve::check_overlay(&theme, &overlay);
    if !issues.is_empty() {
        let list: Vec<String> = issues
            .iter()
            .map(|i| format!("{}: {}", i.key, i.message))
            .collect();
        return Err(format!("rejected overlay for {id}: {}", list.join("; ")));
    }
    let text = ini::write_general(&overlay).map_err(|e| e.to_string())?;

    let link = roots.sddm_themes.join(LINK_NAME);
    if let Ok(meta) = link.symlink_metadata()
        && !meta.file_type().is_symlink()
    {
        return Err(format!(
            "{} exists and is not darwan's symlink; not touching it",
            link.display()
        ));
    }

    write_atomic(&dir.join("theme.conf.user"), text.as_bytes())?;
    let staged = roots.sddm_themes.join(".darwan.new");
    let _ = std::fs::remove_file(&staged);
    std::os::unix::fs::symlink(&dir, &staged).map_err(|e| format!("{}: {e}", staged.display()))?;
    std::fs::rename(&staged, &link).map_err(|e| format!("{}: {e}", link.display()))?;
    std::fs::create_dir_all(&roots.conf_d)
        .map_err(|e| format!("{}: {e}", roots.conf_d.display()))?;
    write_atomic(&roots.conf_d.join(CONF_NAME), b"[Theme]\nCurrent=darwan\n")?;
    Ok(format!("SDDM now uses {id}"))
}

pub fn reset(roots: &Roots) -> Result<String, String> {
    let mut done = Vec::new();
    let conf = roots.conf_d.join(CONF_NAME);
    if conf.symlink_metadata().is_ok() {
        std::fs::remove_file(&conf).map_err(|e| format!("{}: {e}", conf.display()))?;
        done.push(format!("removed {}", conf.display()));
    }
    let link = roots.sddm_themes.join(LINK_NAME);
    match link.symlink_metadata() {
        Ok(m) if m.file_type().is_symlink() => {
            std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))?;
            done.push(format!("removed {}", link.display()));
        }
        Ok(_) => done.push(format!(
            "left {} alone: it is not darwan's symlink",
            link.display()
        )),
        Err(_) => {}
    }
    let mut overlays = Vec::new();
    find_overlays(&roots.themes, &mut overlays);
    for path in overlays {
        std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        done.push(format!("removed {}", path.display()));
    }
    if done.is_empty() {
        done.push("nothing to reset".into());
    }
    Ok(done.join("\n"))
}

pub fn import_font(
    roots: &Roots,
    id: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<String, String> {
    let (dir, theme) = installed_theme(roots, id)?;
    if !theme.manifest.fonts.iter().any(|f| f.file == file_name) {
        let wanted: Vec<&str> = theme
            .manifest
            .fonts
            .iter()
            .map(|f| f.file.as_str())
            .collect();
        return Err(format!(
            "{id} does not need {file_name:?} (it needs: {})",
            wanted.join(", ")
        ));
    }
    if bytes.is_empty() || bytes.len() > MAX_FONT_BYTES {
        return Err(format!(
            "font must be 1 byte to {} MiB",
            MAX_FONT_BYTES / 1024 / 1024
        ));
    }
    let magic = &bytes[..bytes.len().min(4)];
    if ![&[0, 1, 0, 0][..], b"OTTO", b"true", b"ttcf"].contains(&magic) {
        return Err("not a TrueType or OpenType font".into());
    }
    let font_dir = dir.join("font");
    match font_dir.symlink_metadata() {
        Ok(m) if !m.is_dir() => {
            return Err(format!("{} is not a plain directory", font_dir.display()));
        }
        Ok(_) => {}
        Err(_) => {
            std::fs::create_dir(&font_dir).map_err(|e| format!("{}: {e}", font_dir.display()))?;
            std::fs::set_permissions(&font_dir, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("{}: {e}", font_dir.display()))?;
        }
    }
    write_atomic(&font_dir.join(file_name), bytes)?;
    Ok(format!("installed {file_name} for {id}"))
}

// The id is checked, then the resolved path must equal the literal one: no symlink anywhere below the root.
fn installed_theme(roots: &Roots, id: &str) -> Result<(PathBuf, Theme), String> {
    if !catalog::valid_id(id) {
        return Err(format!("invalid theme id {id:?}"));
    }
    let root = roots
        .themes
        .canonicalize()
        .map_err(|e| format!("{}: {e}", roots.themes.display()))?;
    let dir = root.join(id);
    let real = dir
        .canonicalize()
        .map_err(|_| format!("theme {id:?} is not installed"))?;
    if real != dir {
        return Err(format!(
            "theme {id:?} resolves outside {} through a symlink",
            root.display()
        ));
    }
    let main = dir.join("Main.qml").symlink_metadata();
    if !main.is_ok_and(|m| m.is_file()) {
        return Err(format!("{id:?} is not a theme"));
    }
    let theme = catalog::load_theme(id.to_string(), dir.clone())?;
    Ok((dir, theme))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent directory")?;
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(err)?;
    tmp.write_all(bytes).map_err(err)?;
    tmp.as_file()
        .set_permissions(std::fs::Permissions::from_mode(0o644))
        .map_err(err)?;
    tmp.as_file().sync_all().map_err(err)?;
    tmp.persist(path).map_err(|e| err(e.error))?;
    Ok(())
}

fn find_overlays(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            find_overlays(&entry.path(), out);
        } else if kind.is_file() && entry.file_name() == "theme.conf.user" {
            out.push(entry.path());
        }
    }
}

#[cfg(test)]
mod tests;
