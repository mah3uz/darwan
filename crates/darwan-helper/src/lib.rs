use std::collections::BTreeMap;
use std::io::{BufRead, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use darwan_core::catalog::{self, Theme};
use darwan_core::{ini, resolve};

pub const MAX_OVERLAY_BYTES: usize = 64 * 1024;
pub const MAX_FONT_BYTES: usize = 32 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_VIDEO_BYTES: u64 = 512 * 1024 * 1024;
// Overlay keys that name a file, and the name the helper stores that file under.
const MEDIA_KEYS: [(&str, &str); 3] = [
    ("backgroundPath", "background"),
    ("fontTextFile", "fontText"),
    ("fontClockFile", "fontClock"),
];
pub const ATTACHMENT: &str = "@attachment";
const LINK_NAME: &str = "darwan";
const CONF_NAME: &str = "zz-darwan.conf";

pub struct Roots {
    pub themes: PathBuf,
    pub sddm_themes: PathBuf,
    pub conf_d: PathBuf,
    // The user's own background and font files, copied here because SDDM can't read home directories.
    pub media: PathBuf,
}

impl Roots {
    pub fn system() -> Self {
        Self {
            themes: "/usr/share/darwan/themes".into(),
            sddm_themes: "/usr/share/sddm/themes".into(),
            conf_d: "/etc/sddm.conf.d".into(),
            media: "/var/lib/darwan/sddm/media".into(),
        }
    }
}

// Input: one JSON line {"overlay": {...}, "attachments": [{"key", "ext", "size"}, ...]}, then each
// attachment's bytes in order. File keys in the overlay hold "@attachment" and are filled in here.
pub fn apply(roots: &Roots, id: &str, input: &mut dyn BufRead) -> Result<String, String> {
    let mut line = Vec::new();
    input
        .take(MAX_OVERLAY_BYTES as u64 + 1)
        .read_until(b'\n', &mut line)
        .map_err(|e| format!("reading the request: {e}"))?;
    if line.len() > MAX_OVERLAY_BYTES {
        return Err("overlay is too large".into());
    }
    let header: serde_json::Value =
        serde_json::from_slice(&line).map_err(|e| format!("the request is not JSON: {e}"))?;
    let mut overlay: BTreeMap<String, String> =
        serde_json::from_value(header.get("overlay").cloned().unwrap_or_default())
            .map_err(|e| format!("overlay is not a JSON object of strings: {e}"))?;
    let attachments = header
        .get("attachments")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let (dir, theme) = installed_theme(roots, id)?;
    let planned = plan_attachments(roots, &attachments)?;
    for (key, _) in MEDIA_KEYS {
        match (
            overlay.get(key).map(String::as_str),
            planned.iter().find(|p| p.key == key),
        ) {
            (Some(ATTACHMENT), Some(p)) => {
                overlay.insert(key.to_string(), p.path.display().to_string());
            }
            (None, None) => {}
            _ => return Err(format!("{key} must be sent as an attachment")),
        }
    }
    let issues = resolve::check_overlay(&theme, &overlay);
    if !issues.is_empty() {
        let list: Vec<String> = issues
            .iter()
            .map(|i| format!("{}: {}", i.key, i.message))
            .collect();
        return Err(format!("rejected overlay for {id}: {}", list.join("; ")));
    }
    let text = ini::write_general(&overlay).map_err(|e| e.to_string())?;
    store_attachments(roots, &planned, input)?;

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

fn allowed(key: &str, ext: &str) -> Option<u64> {
    let (images, videos, fonts) = (
        ["png", "jpg", "jpeg", "webp", "bmp", "gif"],
        ["mp4", "mkv", "webm", "mov"],
        ["ttf", "otf"],
    );
    match key {
        "backgroundPath" if images.contains(&ext) => Some(MAX_IMAGE_BYTES),
        "backgroundPath" if videos.contains(&ext) => Some(MAX_VIDEO_BYTES),
        "fontTextFile" | "fontClockFile" if fonts.contains(&ext) => Some(MAX_FONT_BYTES as u64),
        _ => None,
    }
}

// The first bytes must match the claimed type, so a renamed file of another kind is refused.
fn magic_matches(ext: &str, head: &[u8]) -> bool {
    match ext {
        "png" => head.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => head.starts_with(&[0xff, 0xd8, 0xff]),
        "webp" => head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP",
        "bmp" => head.starts_with(b"BM"),
        "gif" => head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a"),
        "mp4" | "mov" => head.len() >= 8 && &head[4..8] == b"ftyp",
        "mkv" | "webm" => head.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]),
        "ttf" | "otf" => [&[0, 1, 0, 0][..], b"OTTO", b"true", b"ttcf"]
            .iter()
            .any(|m| head.starts_with(m)),
        _ => false,
    }
}

struct Planned {
    key: String,
    ext: String,
    size: u64,
    path: PathBuf,
}

fn plan_attachments(
    roots: &Roots,
    attachments: &[serde_json::Value],
) -> Result<Vec<Planned>, String> {
    let mut planned: Vec<Planned> = Vec::new();
    for a in attachments {
        let field = |k: &str| a.get(k).and_then(serde_json::Value::as_str).unwrap_or("");
        let (key, ext) = (field("key").to_string(), field("ext").to_ascii_lowercase());
        let size = a
            .get("size")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let role = MEDIA_KEYS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, r)| *r)
            .ok_or_else(|| format!("{key:?} does not take a file"))?;
        let cap = allowed(&key, &ext).ok_or_else(|| format!("{key} does not take .{ext} files"))?;
        if size == 0 || size > cap {
            return Err(format!(
                "{key}: a .{ext} file must be 1 byte to {} MiB",
                cap / 1024 / 1024
            ));
        }
        if planned.iter().any(|p| p.key == key) {
            return Err(format!("{key} is attached twice"));
        }
        let path = roots.media.join(format!("{role}.{ext}"));
        planned.push(Planned {
            key,
            ext,
            size,
            path,
        });
    }
    Ok(planned)
}

// Exactly `size` bytes per attachment, streamed to disk; nothing is taken from a path.
fn store_attachments(
    roots: &Roots,
    planned: &[Planned],
    input: &mut dyn BufRead,
) -> Result<(), String> {
    if !planned.is_empty() {
        ensure_media_dir(&roots.media)?;
    }
    for p in planned {
        let err = |e: std::io::Error| format!("{}: {e}", p.path.display());
        let mut tmp = tempfile::NamedTempFile::new_in(&roots.media).map_err(err)?;
        let copied = std::io::copy(&mut input.take(p.size), tmp.as_file_mut()).map_err(err)?;
        if copied != p.size {
            return Err(format!(
                "{}: expected {} bytes, got {copied}",
                p.key, p.size
            ));
        }
        let mut head = [0u8; 16];
        let n = std::fs::File::open(tmp.path())
            .and_then(|mut f| f.read(&mut head))
            .map_err(err)?;
        if !magic_matches(&p.ext, &head[..n]) {
            return Err(format!("{}: the file is not a real .{}", p.key, p.ext));
        }
        tmp.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o644))
            .map_err(err)?;
        tmp.as_file().sync_all().map_err(err)?;
        tmp.persist(&p.path).map_err(|e| err(e.error))?;
    }
    prune_media(roots, planned)
}

fn ensure_media_dir(media: &Path) -> Result<(), String> {
    std::fs::create_dir_all(media).map_err(|e| format!("{}: {e}", media.display()))?;
    let meta = media
        .symlink_metadata()
        .map_err(|e| format!("{}: {e}", media.display()))?;
    if !meta.is_dir() {
        return Err(format!("{} is not a plain directory", media.display()));
    }
    std::fs::set_permissions(media, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("{}: {e}", media.display()))
}

// Only one SDDM theme is in use, so files it no longer names are removed.
fn prune_media(roots: &Roots, keep: &[Planned]) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(&roots.media) else {
        return Ok(());
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !keep.iter().any(|k| k.path == path) {
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

pub fn reset(roots: &Roots) -> Result<String, String> {
    let mut done = Vec::new();
    if roots.media.exists() {
        std::fs::remove_dir_all(&roots.media)
            .map_err(|e| format!("{}: {e}", roots.media.display()))?;
        done.push(format!("removed {}", roots.media.display()));
    }
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
