use std::fmt;

use crate::catalog::{Catalog, valid_id};
use crate::config::{Target, UserConfig};
use crate::custom::{self, Kind};
use crate::manifest::OptionKind;
use crate::saver::{self, LockAfter, Quality};

pub struct DatePreset {
    pub format: &'static str,
    pub no_weekday: &'static str,
    pub label: &'static str,
}

// `no_weekday` is for themes that already show the weekday on a line of its own.
pub const DATE_PRESETS: &[DatePreset] = &[
    DatePreset {
        format: "dddd, MMMM d",
        no_weekday: "MMMM d",
        label: "Saturday, September 26",
    },
    DatePreset {
        format: "ddd, MMM d",
        no_weekday: "MMM d",
        label: "Sat, Sep 26",
    },
    DatePreset {
        format: "d MMMM yyyy",
        no_weekday: "d MMMM yyyy",
        label: "26 September 2026",
    },
    DatePreset {
        format: "yyyy-MM-dd",
        no_weekday: "yyyy-MM-dd",
        label: "2026-09-26",
    },
    DatePreset {
        format: "dd/MM/yyyy",
        no_weekday: "dd/MM/yyyy",
        label: "26/09/2026",
    },
    DatePreset {
        format: "MM/dd/yyyy",
        no_weekday: "MM/dd/yyyy",
        label: "09/26/2026",
    },
];

pub fn date_preset(format: &str) -> Option<&'static DatePreset> {
    DATE_PRESETS.iter().find(|p| p.format == format)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Theme(Target),
    ClockFormat,
    ClockShowAmPm,
    DateFormat,
    SaverLockAfter,
    SaverReturnAfter,
    SaverQuality,
    GuiLook,
    WallpaperFolder,
    Option { theme: String, key: String },
}

impl Key {
    pub fn parse(s: &str) -> Result<Self, String> {
        Ok(match s {
            "lock.theme" => Key::Theme(Target::Lock),
            "sddm.theme" => Key::Theme(Target::Sddm),
            "clock.format" => Key::ClockFormat,
            "clock.show_ampm" => Key::ClockShowAmPm,
            "date.format" => Key::DateFormat,
            "saver.lock_after" => Key::SaverLockAfter,
            "saver.return_after" => Key::SaverReturnAfter,
            "saver.quality" => Key::SaverQuality,
            "gui.look" => Key::GuiLook,
            "wallpaper.folder" => Key::WallpaperFolder,
            _ => {
                let (theme, key) = s.rsplit_once('.').filter(|(t, k)| valid_id(t) && !k.is_empty()).ok_or_else(|| {
                    format!(
                        "unknown setting {s:?}; use lock.theme, sddm.theme, clock.format, clock.show_ampm, date.format, saver.lock_after, saver.return_after, saver.quality, gui.look, wallpaper.folder or <theme-id>.<option>"
                    )
                })?;
                Key::Option {
                    theme: theme.to_string(),
                    key: key.to_string(),
                }
            }
        })
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Theme(t) => write!(f, "{}.theme", t.table()),
            Key::ClockFormat => f.write_str("clock.format"),
            Key::ClockShowAmPm => f.write_str("clock.show_ampm"),
            Key::DateFormat => f.write_str("date.format"),
            Key::SaverLockAfter => f.write_str("saver.lock_after"),
            Key::SaverReturnAfter => f.write_str("saver.return_after"),
            Key::SaverQuality => f.write_str("saver.quality"),
            Key::GuiLook => f.write_str("gui.look"),
            Key::WallpaperFolder => f.write_str("wallpaper.folder"),
            Key::Option { theme, key } => write!(f, "{theme}.{key}"),
        }
    }
}

pub fn get(config: &UserConfig, key: &Key) -> Result<Option<String>, String> {
    let owned = |v: Option<&str>| v.map(str::to_string);
    match key {
        Key::Theme(t) => config.theme(*t).map(owned),
        Key::ClockFormat => config.clock_format().map(owned),
        Key::ClockShowAmPm => config.clock_show_ampm().map(|v| v.map(|b| b.to_string())),
        Key::DateFormat => config.date_format().map(owned),
        Key::SaverLockAfter => config.saver_lock_after().map(|v| v.map(|l| l.to_string())),
        Key::SaverReturnAfter => config
            .saver_return_after()
            .map(|v| v.map(|s| s.to_string())),
        Key::SaverQuality => config.saver_quality().map(|v| v.map(|q| q.to_string())),
        Key::GuiLook => config.gui_look().map(owned),
        Key::WallpaperFolder => config.wallpaper_folder().map(owned),
        Key::Option { theme, key } => config
            .theme_values(theme)
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .transpose(),
    }
}

// Validated here so the config file never holds a value the resolver would later drop.
pub fn set(
    config: &mut UserConfig,
    catalog: &Catalog,
    key: &Key,
    value: &str,
) -> Result<(), String> {
    match key {
        Key::Theme(t) => {
            catalog
                .get(value)
                .ok_or_else(|| crate::catalog::unknown_theme(value))?;
            config.set_theme(*t, value)
        }
        Key::ClockFormat => match value {
            "12h" | "24h" => config.set_global("clock", "format", toml_edit::value(value)),
            _ => Err(format!("clock.format is 12h or 24h, not {value:?}")),
        },
        Key::ClockShowAmPm => match value {
            "true" | "false" => {
                config.set_global("clock", "show_ampm", toml_edit::value(value == "true"))
            }
            _ => Err(format!("clock.show_ampm is true or false, not {value:?}")),
        },
        Key::DateFormat if value.is_empty() => {
            config.remove_global("date", "format");
            Ok(())
        }
        Key::DateFormat => match date_preset(value) {
            Some(_) => config.set_global("date", "format", toml_edit::value(value)),
            None => Err(format!(
                "date.format is one of {}, or \"\" for the theme's own, not {value:?}",
                DATE_PRESETS
                    .iter()
                    .map(|p| format!("{:?}", p.format))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        },
        Key::SaverLockAfter => match value.parse()? {
            LockAfter::Never => config.set_global("saver", "lock_after", toml_edit::value("never")),
            LockAfter::Secs(n) => {
                config.set_global("saver", "lock_after", toml_edit::value(i64::from(n)))
            }
        },
        Key::SaverReturnAfter => {
            let secs = value
                .parse()
                .map_err(|_| format!("saver.return_after is a number of seconds, not {value:?}"))?;
            let secs = saver::check_return_after(secs)?;
            config.set_global("saver", "return_after", toml_edit::value(i64::from(secs)))
        }
        Key::SaverQuality => {
            let q: Quality = value.parse()?;
            config.set_global("saver", "quality", toml_edit::value(q.as_str()))
        }
        Key::GuiLook => match value {
            "darwan" | "system" => config.set_global("gui", "look", toml_edit::value(value)),
            _ => Err(format!("gui.look is darwan or system, not {value:?}")),
        },
        // Absolute or under the home folder: Darwan runs from different working folders.
        Key::WallpaperFolder => {
            if value.starts_with('/') || value.starts_with("~/") {
                config.set_global("wallpaper", "folder", toml_edit::value(value))
            } else {
                Err(format!(
                    "wallpaper.folder is an absolute path or starts with ~/, not {value:?}"
                ))
            }
        }
        Key::Option { theme, key } => {
            let t = catalog
                .get(theme)
                .ok_or_else(|| crate::catalog::unknown_theme(theme))?;
            if let Some(kind) = custom_kind(t, key) {
                if value.is_empty() {
                    config.remove_theme_value(theme, key);
                    return Ok(());
                }
                custom::check(t, key, value).map_err(|e| format!("{theme}.{key}: {e}"))?;
                return config.set_theme_value(theme, key, value, kind);
            }
            let opt = t.manifest.option(key).ok_or_else(|| {
                let keys: Vec<&str> = t.manifest.options.iter().map(|o| o.key.as_str()).collect();
                let standard: Vec<&str> = custom::SETTINGS.iter().map(|s| s.key).collect();
                if keys.is_empty() {
                    format!(
                        "{theme} has no option {key:?}; the standard settings are {}",
                        standard.join(", ")
                    )
                } else {
                    format!(
                        "{theme} has no option {key:?}; it has {}, and the standard settings {}",
                        keys.join(", "),
                        standard.join(", ")
                    )
                }
            })?;
            opt.check(value)
                .map_err(|e| format!("{theme}.{key}: {e}"))?;
            config.set_theme_value(theme, key, value, opt.kind)
        }
    }
}

// Standard settings and a theme's colour roles, with the TOML type they are stored as.
fn custom_kind(theme: &crate::catalog::Theme, key: &str) -> Option<OptionKind> {
    match custom::setting(key).map(|s| s.kind) {
        Some(Kind::Percent) => Some(OptionKind::Int),
        Some(Kind::Speed | Kind::Contrast) => Some(OptionKind::Range),
        Some(Kind::Bool) => Some(OptionKind::Bool),
        Some(_) => Some(OptionKind::Enum),
        None => theme.manifest.color(key).map(|_| OptionKind::Color),
    }
}

pub fn unset(config: &mut UserConfig, key: &Key) -> bool {
    match key {
        Key::Theme(t) => config.remove_global(t.table(), "theme"),
        Key::ClockFormat => config.remove_global("clock", "format"),
        Key::ClockShowAmPm => config.remove_global("clock", "show_ampm"),
        Key::DateFormat => config.remove_global("date", "format"),
        Key::SaverLockAfter => config.remove_global("saver", "lock_after"),
        Key::SaverReturnAfter => config.remove_global("saver", "return_after"),
        Key::SaverQuality => config.remove_global("saver", "quality"),
        Key::GuiLook => config.remove_global("gui", "look"),
        Key::WallpaperFolder => config.remove_global("wallpaper", "folder"),
        Key::Option { theme, key } => config.remove_theme_value(theme, key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn catalog() -> Catalog {
        Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes"))
            .unwrap()
            .0
    }

    #[test]
    fn keys_round_trip_and_theme_ids_keep_their_slashes() {
        for s in [
            "lock.theme",
            "sddm.theme",
            "clock.format",
            "clock.show_ampm",
            "date.format",
            "saver.lock_after",
            "saver.return_after",
            "saver.quality",
            "gui.look",
            "wallpaper.folder",
            "clockwork/orbital.themeMode",
        ] {
            assert_eq!(Key::parse(s).unwrap().to_string(), s);
        }
        assert_eq!(
            Key::parse("clockwork/orbital.themeMode").unwrap(),
            Key::Option {
                theme: "clockwork/orbital".into(),
                key: "themeMode".into()
            }
        );
        assert!(Key::parse("themeMode").is_err());
        assert!(Key::parse("../x.y").is_err());
    }

    #[test]
    fn set_rejects_what_the_resolver_would_drop() {
        let cat = catalog();
        let mut cfg = UserConfig::default();
        for (k, v) in [
            ("lock.theme", "no-such-theme"),
            ("clock.format", "13h"),
            ("clock.show_ampm", "yes"),
            ("osu.gameMode", "arcade"),
            ("osu.noSuchKey", "x"),
            ("date.format", "a\nb"),
            ("date.format", "ddd d"),
            ("saver.lock_after", "-1"),
            ("saver.return_after", "0"),
            ("saver.return_after", "never"),
            ("saver.quality", "ultra"),
            ("gui.look", "windows"),
            ("wallpaper.folder", "relative/folder"),
            ("wallpaper.folder", ""),
        ] {
            assert!(
                set(&mut cfg, &cat, &Key::parse(k).unwrap(), v).is_err(),
                "{k}={v}"
            );
        }
        assert_eq!(cfg.to_string(), "", "nothing invalid may reach the file");
    }

    #[test]
    fn set_get_unset_round_trip() {
        let cat = catalog();
        let mut cfg = UserConfig::default();
        let key = Key::parse("terraria.background_index").unwrap();
        set(&mut cfg, &cat, &key, "3").unwrap();
        assert_eq!(get(&cfg, &key), Ok(Some("3".into())));
        assert!(
            cfg.to_string().contains("background_index = \"3\""),
            "an enum choice is saved as the string the manifest declares"
        );
        assert!(unset(&mut cfg, &key));
        assert_eq!(get(&cfg, &key), Ok(None));

        let look = Key::parse("gui.look").unwrap();
        set(&mut cfg, &cat, &look, "system").unwrap();
        assert_eq!(get(&cfg, &look), Ok(Some("system".into())));
        assert!(
            cfg.to_string().contains("[gui]\nlook = \"system\""),
            "the GUI's look lives in the one config file"
        );

        let folder = Key::parse("wallpaper.folder").unwrap();
        set(&mut cfg, &cat, &folder, "~/Pictures/Walls").unwrap();
        assert_eq!(
            get(&cfg, &folder),
            Ok(Some("~/Pictures/Walls".into())),
            "kept as written, expanded where used"
        );

        let back = Key::parse("saver.return_after").unwrap();
        set(&mut cfg, &cat, &back, "45").unwrap();
        assert_eq!(get(&cfg, &back), Ok(Some("45".into())));
        assert!(
            cfg.to_string().contains("return_after = 45\n"),
            "saved as a number, as the README shows it"
        );
    }

    #[test]
    fn date_format_takes_a_preset_and_empty_means_the_theme_default() {
        let cat = catalog();
        let mut cfg = UserConfig::default();
        set(&mut cfg, &cat, &Key::DateFormat, "yyyy-MM-dd").unwrap();
        assert_eq!(get(&cfg, &Key::DateFormat), Ok(Some("yyyy-MM-dd".into())));
        set(&mut cfg, &cat, &Key::DateFormat, "").unwrap();
        assert_eq!(
            get(&cfg, &Key::DateFormat),
            Ok(None),
            "\"\" unsets instead of storing an empty format"
        );
    }

    #[test]
    fn no_weekday_variants_really_drop_the_weekday() {
        for p in DATE_PRESETS {
            assert!(!p.no_weekday.contains("ddd"), "{}", p.format);
        }
    }
}
