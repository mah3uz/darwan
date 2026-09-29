use std::io::{self, Write};
use std::path::Path;

use toml_edit::{DocumentMut, Item, Table, Value};

use crate::manifest::OptionKind;
use crate::saver::{self, LockAfter, Quality};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("reading {path}: {source}")]
    Io { path: String, source: io::Error },
    #[error("parsing config: {0}")]
    Parse(#[from] toml_edit::TomlError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Lock,
    Sddm,
}

impl Target {
    pub fn table(self) -> &'static str {
        match self {
            Target::Lock => "lock",
            Target::Sddm => "sddm",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct UserConfig {
    doc: DocumentMut,
}

impl UserConfig {
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        Ok(Self { doc: text.parse()? })
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Io {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let dir = path.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(self.doc.to_string().as_bytes())?;
        tmp.as_file().sync_all()?;
        tmp.persist(path).map(drop).map_err(|e| e.error)
    }

    pub fn theme(&self, target: Target) -> Result<Option<&str>, String> {
        self.string(target.table(), "theme")
    }

    pub fn set_theme(&mut self, target: Target, id: &str) -> Result<(), String> {
        self.set_global(target.table(), "theme", toml_edit::value(id))
    }

    pub fn set_global(&mut self, table: &str, key: &str, value: Item) -> Result<(), String> {
        let section = self
            .doc
            .entry(table)
            .or_insert(Item::Table(Table::new()))
            .as_table_like_mut()
            .ok_or_else(|| format!("{table} must be a table"))?;
        section.insert(key, value);
        Ok(())
    }

    pub fn remove_global(&mut self, table: &str, key: &str) -> bool {
        self.doc
            .get_mut(table)
            .and_then(Item::as_table_like_mut)
            .is_some_and(|t| t.remove(key).is_some())
    }

    pub fn clock_format(&self) -> Result<Option<&str>, String> {
        let v = self.string("clock", "format")?;
        match v {
            None | Some("12h" | "24h") => Ok(v),
            Some(other) => Err(format!(
                "clock.format must be \"12h\" or \"24h\", got {other:?}"
            )),
        }
    }

    pub fn clock_show_ampm(&self) -> Result<Option<bool>, String> {
        match self.item("clock", "show_ampm") {
            None => Ok(None),
            Some(item) => item
                .as_bool()
                .map(Some)
                .ok_or_else(|| "clock.show_ampm must be true or false".into()),
        }
    }

    pub fn saver_lock_after(&self) -> Result<Option<LockAfter>, String> {
        match self.item("saver", "lock_after") {
            None => Ok(None),
            Some(item) => match (item.as_integer(), item.as_str()) {
                (Some(n), _) => u32::try_from(n)
                    .map(|n| Some(LockAfter::Secs(n)))
                    .map_err(|_| format!("saver.lock_after must be 0 or more seconds, got {n}")),
                (_, Some(s)) => s.parse().map(Some),
                _ => Err("saver.lock_after must be a number of seconds or \"never\"".into()),
            },
        }
    }

    pub fn saver_return_after(&self) -> Result<Option<u32>, String> {
        match self.item("saver", "return_after") {
            None => Ok(None),
            Some(item) => item
                .as_integer()
                .ok_or_else(|| "saver.return_after must be a number of seconds".to_string())
                .and_then(saver::check_return_after)
                .map(Some),
        }
    }

    pub fn saver_quality(&self) -> Result<Option<Quality>, String> {
        self.string("saver", "quality")?.map(str::parse).transpose()
    }

    // Where the Wallpapers page reads and downloads, as the user wrote it (`~/…` allowed).
    pub fn wallpaper_folder(&self) -> Result<Option<&str>, String> {
        self.string("wallpaper", "folder")
    }

    // The online tab's optional groups the user switched on; unknown names are dropped. Sexual content has no entry.
    pub fn wallpaper_allow(&self) -> Result<Option<Vec<String>>, String> {
        match self.item("wallpaper", "allow") {
            None => Ok(None),
            Some(item) => item
                .as_array()
                .map(|a| {
                    Some(
                        a.iter()
                            .filter_map(|v| v.as_str())
                            .filter(|id| {
                                crate::wallpaper::filter::GROUPS.iter().any(|g| g.id == *id)
                            })
                            .map(str::to_string)
                            .collect(),
                    )
                })
                .ok_or_else(|| "wallpaper.allow must be a list of names".into()),
        }
    }

    // Colour generators the user wants run after a wallpaper change (matugen, pywal, wallust, hellwal), by id.
    pub fn wallpaper_colours(&self) -> Vec<String> {
        self.item("wallpaper", "colours")
            .and_then(Item::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    // Wallpaper tools the user let Darwan restart (swaybg, mpvpaper, gSlapper, wbg), by name.
    pub fn wallpaper_restart(&self) -> Vec<String> {
        self.item("wallpaper", "restart")
            .and_then(Item::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    // The GUI's look: "darwan", its own, or "system" to follow the Qt theme's palette and font.
    pub fn gui_look(&self) -> Result<Option<&str>, String> {
        self.string("gui", "look")
    }

    pub fn date_format(&self) -> Result<Option<&str>, String> {
        self.string("date", "format")
    }

    pub fn theme_values(&self, id: &str) -> Vec<(String, Result<String, String>)> {
        let Some(table) = self
            .doc
            .get("themes")
            .and_then(|t| t.get(id))
            .and_then(Item::as_table_like)
        else {
            return Vec::new();
        };
        table
            .iter()
            .map(|(k, item)| {
                let v = match item.as_value() {
                    Some(Value::String(s)) => Ok(s.value().clone()),
                    Some(Value::Boolean(b)) => Ok(b.value().to_string()),
                    Some(Value::Integer(i)) => Ok(i.value().to_string()),
                    Some(Value::Float(f)) => Ok(f.value().to_string()),
                    _ => Err("expected a string, boolean or number".to_string()),
                };
                (k.to_string(), v)
            })
            .collect()
    }

    pub fn set_theme_value(
        &mut self,
        id: &str,
        key: &str,
        value: &str,
        kind: OptionKind,
    ) -> Result<(), String> {
        let themes = self
            .doc
            .entry("themes")
            .or_insert_with(|| {
                let mut t = Table::new();
                t.set_implicit(true);
                Item::Table(t)
            })
            .as_table_like_mut()
            .ok_or("themes must be a table")?;
        let theme = themes
            .entry(id)
            .or_insert(Item::Table(Table::new()))
            .as_table_like_mut()
            .ok_or_else(|| format!("themes.\"{id}\" must be a table"))?;
        let typed = match kind {
            OptionKind::Bool if value == "true" || value == "false" => {
                toml_edit::value(value == "true")
            }
            OptionKind::Int => match value.parse::<i64>() {
                Ok(n) => toml_edit::value(n),
                Err(_) => toml_edit::value(value),
            },
            OptionKind::Range => match value.parse::<f64>() {
                Ok(n) => toml_edit::value(n),
                Err(_) => toml_edit::value(value),
            },
            _ => toml_edit::value(value),
        };
        theme.insert(key, typed);
        Ok(())
    }

    pub fn remove_theme_value(&mut self, id: &str, key: &str) -> bool {
        let Some(themes) = self.doc.get_mut("themes").and_then(Item::as_table_like_mut) else {
            return false;
        };
        let Some(theme) = themes.get_mut(id).and_then(Item::as_table_like_mut) else {
            return false;
        };
        let removed = theme.remove(key).is_some();
        if theme.is_empty() {
            themes.remove(id);
        }
        removed
    }

    // Every option and customisation of one theme; which theme the lock and SDDM use stays.
    pub fn remove_theme(&mut self, id: &str) -> bool {
        self.doc
            .get_mut("themes")
            .and_then(Item::as_table_like_mut)
            .is_some_and(|themes| themes.remove(id).is_some())
    }

    fn item(&self, table: &str, key: &str) -> Option<&Item> {
        self.doc.get(table).and_then(|t| t.get(key))
    }

    fn string(&self, table: &str, key: &str) -> Result<Option<&str>, String> {
        match self.item(table, key) {
            None => Ok(None),
            Some(item) => item
                .as_str()
                .map(Some)
                .ok_or_else(|| format!("{table}.{key} must be a string")),
        }
    }
}

impl std::fmt::Display for UserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.doc.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Starting a theme over must not touch other themes or which theme is in use.
    #[test]
    fn removing_a_theme_clears_only_that_theme() {
        let mut cfg = UserConfig::parse(
            "sddm = { theme = \"material-you\" }\n[themes.material-you]\naccent = \"generate\"\n[themes.\"clockwork/orbital\"]\nvariant = \"light\"\n",
        )
        .unwrap();
        assert!(cfg.remove_theme("material-you"));
        assert!(cfg.theme_values("material-you").is_empty());
        assert_eq!(cfg.theme_values("clockwork/orbital").len(), 1);
        assert_eq!(cfg.theme(Target::Sddm), Ok(Some("material-you")));
        assert!(!cfg.remove_theme("material-you"), "nothing left to remove");
    }

    #[test]
    fn missing_file_is_an_empty_config_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = UserConfig::load(&dir.path().join("config.toml")).unwrap();
        assert_eq!(cfg.theme(Target::Lock), Ok(None));
    }

    #[test]
    fn edits_keep_the_users_comments() {
        let mut cfg =
            UserConfig::parse("# my lock\n[lock]\ntheme = \"osu\" # favourite\n").unwrap();
        cfg.set_theme(Target::Lock, "clockwork/orbital").unwrap();
        let text = cfg.to_string();
        assert!(text.contains("# my lock"));
        assert!(text.contains("theme = \"clockwork/orbital\""));
    }

    #[test]
    fn a_new_theme_choice_is_written_as_a_section_not_an_inline_table() {
        let mut cfg = UserConfig::default();
        cfg.set_theme(Target::Sddm, "osu").unwrap();
        assert_eq!(cfg.to_string(), "[sddm]\ntheme = \"osu\"\n");
        let mut bad = UserConfig::parse("sddm = 1\n").unwrap();
        assert!(bad.set_theme(Target::Sddm, "osu").is_err());
    }

    #[test]
    fn theme_ids_with_slashes_become_quoted_table_names() {
        let mut cfg = UserConfig::default();
        cfg.set_theme_value(
            "clockwork/orbital",
            "enableWindup",
            "false",
            OptionKind::Bool,
        )
        .unwrap();
        assert_eq!(
            cfg.to_string(),
            "[themes.\"clockwork/orbital\"]\nenableWindup = false\n"
        );
        assert_eq!(
            cfg.theme_values("clockwork/orbital"),
            vec![("enableWindup".into(), Ok("false".into()))]
        );
    }

    #[test]
    fn toml_types_come_back_as_the_strings_themes_compare() {
        let cfg = UserConfig::parse(
            "[themes.terraria]\nbackground_index = 3\nbackground_mode = \"static\"\nbad = [1]\n",
        )
        .unwrap();
        let values = cfg.theme_values("terraria");
        assert!(values.contains(&("background_index".into(), Ok("3".into()))));
        assert!(values.contains(&("background_mode".into(), Ok("static".into()))));
        assert!(values.iter().any(|(k, v)| k == "bad" && v.is_err()));
    }

    #[test]
    fn removing_the_last_value_removes_the_theme_section() {
        let mut cfg = UserConfig::parse("[themes.osu]\ngameMode = \"menu\"\n").unwrap();
        assert!(cfg.remove_theme_value("osu", "gameMode"));
        assert!(!cfg.to_string().contains("osu"), "{cfg}");
    }

    #[test]
    fn saver_settings_read_numbers_and_never_and_reject_the_rest() {
        let cfg =
            UserConfig::parse("[saver]\nlock_after = 60\nreturn_after = 45\nquality = \"eco\"\n")
                .unwrap();
        assert_eq!(cfg.saver_lock_after(), Ok(Some(LockAfter::Secs(60))));
        assert_eq!(cfg.saver_return_after(), Ok(Some(45)));
        assert_eq!(cfg.saver_quality(), Ok(Some(Quality::Eco)));
        let never = UserConfig::parse("[saver]\nlock_after = \"never\"\n").unwrap();
        assert_eq!(never.saver_lock_after(), Ok(Some(LockAfter::Never)));
        for bad in [
            "lock_after = -1",
            "lock_after = true",
            "return_after = 0",
            "return_after = \"never\"",
            "quality = \"ultra\"",
        ] {
            let cfg = UserConfig::parse(&format!("[saver]\n{bad}\n")).unwrap();
            assert!(
                cfg.saver_lock_after().is_err()
                    || cfg.saver_return_after().is_err()
                    || cfg.saver_quality().is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn wrong_global_types_are_reported_not_ignored() {
        let cfg = UserConfig::parse("[clock]\nformat = \"13h\"\nshow_ampm = \"yes\"\n").unwrap();
        assert!(cfg.clock_format().is_err());
        assert!(cfg.clock_show_ampm().is_err());
    }

    #[test]
    fn save_replaces_the_file_atomically_and_leaves_no_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("darwan/config.toml");
        let mut cfg = UserConfig::default();
        cfg.set_theme(Target::Sddm, "pixel-rainyroom").unwrap();
        cfg.save(&path).unwrap();
        assert_eq!(
            UserConfig::load(&path).unwrap().theme(Target::Sddm),
            Ok(Some("pixel-rainyroom"))
        );
        let entries: Vec<_> = std::fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }
}
