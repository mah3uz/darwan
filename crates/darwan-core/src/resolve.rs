use std::collections::BTreeMap;

use crate::catalog::Theme;
use crate::config::UserConfig;
use crate::custom::{self, Host};
use crate::settings;

#[derive(Debug, PartialEq, Eq)]
pub struct Issue {
    pub key: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Resolved {
    pub overlay: BTreeMap<String, String>,
    pub issues: Vec<Issue>,
}

// The theme's own options go straight into the overlay; standard settings are returned for custom::apply.
fn checked_values(
    theme: &Theme,
    config: &UserConfig,
    r: &mut Resolved,
) -> BTreeMap<String, String> {
    let issue_key = |key: &str| format!("themes.\"{}\".{key}", theme.id);
    let mut standard = BTreeMap::new();
    for (key, value) in config.theme_values(&theme.id) {
        let is_standard = custom::setting(&key).is_some() || theme.manifest.color(&key).is_some();
        let checked = value.and_then(|v| {
            if is_standard {
                custom::check(theme, &key, &v).map(|()| v)
            } else {
                match theme.manifest.option(&key) {
                    None => Err(format!("{} has no option {key:?}", theme.manifest.name)),
                    Some(opt) => opt.check(&v).map(|()| v),
                }
            }
        });
        match checked {
            Ok(v) if is_standard => {
                standard.insert(key, v);
            }
            Ok(v) => {
                r.overlay.insert(key, v);
            }
            Err(message) => r.issues.push(Issue {
                key: issue_key(&key),
                message,
            }),
        }
    }
    standard
}

// The palette the theme's image would give, whatever its colours are set to, for picking from.
pub fn image_palette(
    theme: &Theme,
    config: &UserConfig,
    host: &dyn Host,
) -> Option<custom::Palette> {
    if custom::unsupported(theme, "accent").is_some() {
        return None;
    }
    let mut r = Resolved::default();
    let mut standard = checked_values(theme, config, &mut r);
    standard.insert("accent".into(), custom::GENERATE.into());
    custom::apply(theme, &standard, host, &mut r.overlay, &mut Vec::new())
}

// Defaults are not copied: SDDM and the runtime layer the overlay over theme.conf.
pub fn resolve(theme: &Theme, config: &UserConfig, host: &dyn Host) -> Resolved {
    let mut r = Resolved::default();
    let issue_key = |key: &str| format!("themes.\"{}\".{key}", theme.id);
    let standard = checked_values(theme, config, &mut r);
    let mut issues = Vec::new();
    custom::apply(theme, &standard, host, &mut r.overlay, &mut issues);
    r.issues
        .extend(issues.into_iter().map(|(key, message)| Issue {
            key: issue_key(&key),
            message,
        }));

    let supports = &theme.manifest.supports;
    if supports.clock_format {
        global(
            &mut r,
            "clock.format",
            "clockFormat",
            config.clock_format().map(|v| v.map(str::to_string)),
        );
        global(
            &mut r,
            "clock.show_ampm",
            "clockShowAmPm",
            config.clock_show_ampm().map(|v| v.map(|b| b.to_string())),
        );
    }
    if supports.date_format {
        match config.date_format() {
            Ok(Some(f)) if !f.is_empty() => match settings::date_preset(f) {
                Some(p) => {
                    r.overlay.insert("dateFormat".into(), p.format.into());
                    r.overlay
                        .insert("dateFormatNoWeekday".into(), p.no_weekday.into());
                }
                None => r.issues.push(Issue {
                    key: "date.format".into(),
                    message: format!("{f:?} is not one of the date presets"),
                }),
            },
            Ok(_) => {}
            Err(message) => r.issues.push(Issue {
                key: "date.format".into(),
                message,
            }),
        }
    }
    r
}

// The same rules resolve() applies, for an overlay that arrives already built (the root helper).
pub fn check_overlay(theme: &Theme, overlay: &BTreeMap<String, String>) -> Vec<Issue> {
    let supports = &theme.manifest.supports;
    overlay
        .iter()
        .filter_map(|(key, value)| {
            let result = match key.as_str() {
                "clockFormat" if supports.clock_format => match value.as_str() {
                    "12h" | "24h" => Ok(()),
                    _ => Err(format!("expected 12h or 24h, got {value:?}")),
                },
                "clockShowAmPm" if supports.clock_format => match value.as_str() {
                    "true" | "false" => Ok(()),
                    _ => Err(format!("expected true or false, got {value:?}")),
                },
                "dateFormat" if supports.date_format => settings::date_preset(value)
                    .map(drop)
                    .ok_or_else(|| format!("{value:?} is not a date preset")),
                "dateFormatNoWeekday" if supports.date_format => {
                    let expected = overlay
                        .get("dateFormat")
                        .and_then(|f| settings::date_preset(f))
                        .map(|p| p.no_weekday);
                    if expected == Some(value.as_str()) {
                        Ok(())
                    } else {
                        Err(format!("{value:?} does not match dateFormat"))
                    }
                }
                _ => match custom::check_contract(theme, key, value) {
                    Some(result) => result,
                    None => match theme.manifest.option(key) {
                        Some(opt) => opt.check(value),
                        None => Err(format!("{} has no option {key:?}", theme.manifest.name)),
                    },
                },
            };
            result.err().map(|message| Issue {
                key: key.clone(),
                message,
            })
        })
        .collect()
}

fn global(
    r: &mut Resolved,
    config_key: &str,
    overlay_key: &str,
    value: Result<Option<String>, String>,
) {
    match value {
        Ok(Some(v)) => {
            r.overlay.insert(overlay_key.to_string(), v);
        }
        Ok(None) => {}
        Err(message) => r.issues.push(Issue {
            key: config_key.to_string(),
            message,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(theme: &Theme, config: &UserConfig) -> Resolved {
        super::resolve(theme, config, &custom::Offline)
    }
    use crate::manifest::Manifest;

    fn theme(supports: &str) -> Theme {
        let manifest = Manifest::parse(&format!(
            "name = \"Orbital\"\nauthor = \"a\"\nbackground = \"color\"\n[supports]\n{supports}\n\
             [[option]]\nkey = \"enableWindup\"\nlabel = \"W\"\ntype = \"bool\"\n"
        ))
        .unwrap();
        Theme {
            id: "clockwork/orbital".into(),
            dir: "/nonexistent".into(),
            manifest,
            defaults: BTreeMap::new(),
            preview: None,
        }
    }

    fn cfg(text: &str) -> UserConfig {
        UserConfig::parse(text).unwrap()
    }

    struct Image;
    impl custom::Host for Image {
        fn desktop_prefers_dark(&self) -> Option<bool> {
            None
        }
        fn desktop_wallpaper(&self, _: Option<bool>) -> Option<custom::Wallpaper> {
            None
        }
        fn palette(
            &self,
            _: &std::path::Path,
            r: &custom::PaletteRequest,
        ) -> Result<custom::Palette, String> {
            let primary = r.seed.clone().unwrap_or_else(|| "#123456".into());
            Ok(custom::Palette::from([("primary".to_string(), primary)]))
        }
    }

    // The colour picker offers the image's own colours even while the user's accent is picked.
    #[test]
    fn the_image_palette_ignores_a_picked_accent() {
        let mut t = theme("colors = true\nmaterial_palette = true");
        t.preview = Some("/nonexistent/preview.jpg".into());
        let c = cfg("[themes.\"clockwork/orbital\"]\naccent = \"#ff0000\"\n");
        let p = image_palette(&t, &c, &Image).unwrap();
        assert_eq!(p["primary"], "#123456");
        assert_eq!(
            image_palette(&theme(""), &c, &Image),
            None,
            "no colours, no palette"
        );
    }

    #[test]
    fn valid_theme_value_lands_in_the_overlay() {
        let r = resolve(
            &theme(""),
            &cfg("[themes.\"clockwork/orbital\"]\nenableWindup = false\n"),
        );
        assert_eq!(
            r.overlay.get("enableWindup").map(String::as_str),
            Some("false")
        );
        assert!(r.issues.is_empty());
    }

    #[test]
    fn invalid_or_unknown_values_are_issues_and_never_reach_the_overlay() {
        let r = resolve(
            &theme(""),
            &cfg("[themes.\"clockwork/orbital\"]\nenableWindup = \"maybe\"\nfont = \"x\"\n"),
        );
        assert!(r.overlay.is_empty());
        assert_eq!(r.issues.len(), 2);
    }

    #[test]
    fn globals_only_reach_themes_that_support_them() {
        let config = cfg("[clock]\nformat = \"12h\"\n[date]\nformat = \"dddd, MMMM d\"\n");
        assert!(resolve(&theme(""), &config).overlay.is_empty());
        let r = resolve(&theme("clock_format = true\ndate_format = true"), &config);
        assert_eq!(
            r.overlay.get("clockFormat").map(String::as_str),
            Some("12h")
        );
        assert_eq!(
            r.overlay.get("dateFormat").map(String::as_str),
            Some("dddd, MMMM d")
        );
        assert_eq!(
            r.overlay.get("dateFormatNoWeekday").map(String::as_str),
            Some("MMMM d"),
            "two-line themes need the date without the weekday they already show"
        );
    }

    #[test]
    fn a_date_format_that_is_not_a_preset_never_reaches_a_theme() {
        let r = resolve(
            &theme("date_format = true"),
            &cfg("[date]\nformat = \"'; x\"\n"),
        );
        assert!(r.overlay.is_empty());
        assert_eq!(r.issues[0].key, "date.format");
    }

    #[test]
    fn check_overlay_accepts_what_resolve_produces_and_rejects_anything_else() {
        let t = theme("clock_format = true\ndate_format = true");
        let r = resolve(
            &t,
            &cfg(
                "[clock]\nformat = \"12h\"\n[date]\nformat = \"ddd, MMM d\"\n[themes.\"clockwork/orbital\"]\nenableWindup = true\n",
            ),
        );
        assert!(check_overlay(&t, &r.overlay).is_empty());
        let bad: BTreeMap<String, String> = [
            ("background", "/etc/shadow"),
            ("dateFormat", "x"),
            ("enableWindup", "yes"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert_eq!(
            check_overlay(&t, &bad).len(),
            3,
            "unknown key, a date format that is not a preset, bad value"
        );
        let mismatched: BTreeMap<String, String> = [
            ("dateFormat", "ddd, MMM d"),
            ("dateFormatNoWeekday", "yyyy"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        assert_eq!(check_overlay(&t, &mismatched).len(), 1);
    }

    #[test]
    fn empty_date_format_means_theme_default_so_no_key_is_written() {
        let r = resolve(
            &theme("date_format = true"),
            &cfg("[date]\nformat = \"\"\n"),
        );
        assert!(!r.overlay.contains_key("dateFormat"));
    }

    #[test]
    fn a_bad_global_is_an_issue_only_for_themes_that_use_it() {
        let config = cfg("[clock]\nformat = \"13h\"\n");
        assert!(resolve(&theme(""), &config).issues.is_empty());
        assert_eq!(
            resolve(&theme("clock_format = true"), &config).issues[0].key,
            "clock.format"
        );
    }
}
