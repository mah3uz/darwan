use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::ini;
use crate::manifest::Manifest;

#[derive(Debug)]
pub struct Theme {
    pub id: String,
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub defaults: BTreeMap<String, String>,
    pub preview: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoadProblem {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Catalog {
    themes: Vec<Theme>,
}

impl Catalog {
    // Skips broken themes so one bad manifest can't hide the rest.
    pub fn load(root: &Path) -> io::Result<(Self, Vec<LoadProblem>)> {
        let mut dirs = Vec::new();
        find_theme_dirs(root, &mut dirs)?;
        dirs.sort();
        let mut themes = Vec::new();
        let mut problems = Vec::new();
        for dir in dirs {
            let Some(id) = theme_id(root, &dir) else {
                problems.push(LoadProblem {
                    id: dir.display().to_string(),
                    message: "path is not valid UTF-8".into(),
                });
                continue;
            };
            match load_theme(id.clone(), dir) {
                Ok(theme) => themes.push(theme),
                Err(message) => problems.push(LoadProblem { id, message }),
            }
        }
        Ok((Self { themes }, problems))
    }

    pub fn themes(&self) -> &[Theme] {
        &self.themes
    }

    pub fn get(&self, id: &str) -> Option<&Theme> {
        self.themes.iter().find(|t| t.id == id)
    }

    pub fn into_themes(self) -> Vec<Theme> {
        self.themes
    }
}

// Ids reach a root helper, so only lowercase path segments are accepted: no `..`, no absolute paths.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.split('/').all(|seg| {
            !seg.is_empty()
                && seg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                && !seg.starts_with('-')
        })
}

fn find_theme_dirs(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.join("Main.qml").is_file() {
        out.push(dir.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        if !hidden && entry.file_type()?.is_dir() {
            find_theme_dirs(&entry.path(), out)?;
        }
    }
    Ok(())
}

fn theme_id(root: &Path, dir: &Path) -> Option<String> {
    let parts: Option<Vec<&str>> = dir
        .strip_prefix(root)
        .ok()?
        .iter()
        .map(|c| c.to_str())
        .collect();
    Some(parts?.join("/"))
}

pub fn load_theme(id: String, dir: PathBuf) -> Result<Theme, String> {
    let manifest_text = std::fs::read_to_string(dir.join("darwan.toml"))
        .map_err(|e| format!("darwan.toml: {e}"))?;
    let manifest = Manifest::parse(&manifest_text).map_err(|e| format!("darwan.toml: {e}"))?;
    let conf =
        std::fs::read_to_string(dir.join("theme.conf")).map_err(|e| format!("theme.conf: {e}"))?;
    let preview = ["preview.gif", "preview.png"]
        .iter()
        .map(|f| dir.join(f))
        .find(|p| p.is_file());
    Ok(Theme {
        id,
        manifest,
        defaults: ini::parse_general(&conf),
        preview,
        dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_theme(root: &Path, id: &str, manifest: &str) {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Main.qml"), "").unwrap();
        std::fs::write(dir.join("theme.conf"), "[General]\n").unwrap();
        std::fs::write(dir.join("darwan.toml"), manifest).unwrap();
    }

    const OK: &str = "name = \"X\"\nauthor = \"a\"\nbackground = \"color\"\n";

    #[test]
    fn ids_reject_traversal_absolute_paths_and_odd_characters() {
        assert!(valid_id("clockwork/orbital"));
        assert!(valid_id("reverse-1999-1"));
        for bad in [
            "",
            "../etc",
            "a/../b",
            "/abs",
            "a//b",
            "a/",
            "Genshin",
            "a b",
            "a\nb",
            "-x",
            "a/.hidden",
        ] {
            assert!(!valid_id(bad), "{bad:?}");
        }
    }

    #[test]
    fn every_shipped_id_is_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let (cat, _) = Catalog::load(&root).unwrap();
        for t in cat.themes() {
            assert!(valid_id(&t.id), "{}", t.id);
        }
    }

    #[test]
    fn nested_theme_ids_use_forward_slashes() {
        let root = tempfile::tempdir().unwrap();
        write_theme(root.path(), "clockwork/orbital", OK);
        write_theme(root.path(), "osu", OK);
        let (cat, problems) = Catalog::load(root.path()).unwrap();
        assert!(problems.is_empty());
        let ids: Vec<&str> = cat.themes().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["clockwork/orbital", "osu"]);
    }

    #[test]
    fn a_broken_manifest_is_reported_without_hiding_other_themes() {
        let root = tempfile::tempdir().unwrap();
        write_theme(root.path(), "good", OK);
        write_theme(root.path(), "bad", "name = 1\n");
        let (cat, problems) = Catalog::load(root.path()).unwrap();
        assert!(cat.get("good").is_some());
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].id, "bad");
    }

    #[test]
    fn folders_inside_a_theme_are_not_themes() {
        let root = tempfile::tempdir().unwrap();
        write_theme(root.path(), "osu", OK);
        std::fs::create_dir_all(root.path().join("osu/font")).unwrap();
        std::fs::write(root.path().join("osu/font/Main.qml"), "").unwrap();
        let (cat, _) = Catalog::load(root.path()).unwrap();
        assert_eq!(cat.themes().len(), 1);
    }
}
