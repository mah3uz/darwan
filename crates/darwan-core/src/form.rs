use crate::catalog::Theme;
use crate::config::UserConfig;
use crate::custom::{self, Area, Kind};
use crate::manifest::OptionKind;
use crate::settings::{self, Key};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    Bool,
    Choice(Vec<(String, String)>),
    Int { min: i64, max: i64 },
    Range { min: f64, max: f64, step: f64 },
    // `generate` offers colours generated from an image alongside a picked one.
    Color { generate: bool },
    File(Vec<String>),
    // A background: an image or video file, "desktop" or a colour; empty is the theme's own.
    Media(Vec<String>),
    // A font family, or a .ttf/.otf file.
    Font,
    Text,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub key: Key,
    pub label: String,
    pub kind: FieldKind,
    pub value: String,
    pub is_set: bool,
    pub disabled: Option<String>,
    // The heading a standard customisation is shown under; None for theme options and globals.
    pub group: Option<&'static str>,
}

// A theme's own options, then the global ones, each saying why it is disabled when it is.
pub fn fields(theme: &Theme, config: &UserConfig) -> Vec<Field> {
    let current = |key: &Key, default: &str| match settings::get(config, key) {
        Ok(Some(v)) => (v, true),
        _ => (default.to_string(), false),
    };

    let mut out: Vec<Field> = theme
        .manifest
        .options
        .iter()
        .map(|opt| {
            let key = Key::Option {
                theme: theme.id.clone(),
                key: opt.key.clone(),
            };
            let default = theme
                .defaults
                .get(&opt.key)
                .map(String::as_str)
                .unwrap_or("");
            let (mut value, mut is_set) = current(&key, default);
            if opt.check(&value).is_err() {
                (value, is_set) = (default.to_string(), false);
            }
            let kind = match opt.kind {
                OptionKind::Bool => FieldKind::Bool,
                OptionKind::Enum => FieldKind::Choice(
                    opt.choices
                        .iter()
                        .map(|c| (c.value.clone(), c.label.clone()))
                        .collect(),
                ),
                OptionKind::Int => FieldKind::Int {
                    min: opt.min.map_or(i64::MIN, |m| m as i64),
                    max: opt.max.map_or(i64::MAX, |m| m as i64),
                },
                OptionKind::Range => {
                    let (min, max) = (opt.min.unwrap_or(0.0), opt.max.unwrap_or(1.0));
                    FieldKind::Range {
                        min,
                        max,
                        step: (max - min) / 20.0,
                    }
                }
                OptionKind::Color => FieldKind::Color { generate: false },
                OptionKind::File => FieldKind::File(opt.filters.clone()),
            };
            Field {
                key,
                label: opt.label.clone(),
                kind,
                value,
                is_set,
                disabled: None,
                group: None,
            }
        })
        .collect();

    for i in 0..out.len() {
        let opt = &theme.manifest.options[i];
        for (dep, wanted) in &opt.enabled_when {
            let Some(d) = out
                .iter()
                .position(|f| matches!(&f.key, Key::Option { key, .. } if key == dep))
            else {
                continue;
            };
            if &out[d].value != wanted {
                let shown = match &out[d].kind {
                    FieldKind::Choice(c) => c
                        .iter()
                        .find(|(v, _)| v == wanted)
                        .map_or(wanted.clone(), |(_, l)| l.clone()),
                    _ => wanted.clone(),
                };
                out[i].disabled = Some(format!("only when {} is {shown}", out[d].label));
            }
        }
    }

    out.extend(standard_fields(theme, config));

    let name = &theme.manifest.name;
    let supports = &theme.manifest.supports;
    // Unset globals show what the theme itself does, declared in its theme.conf.
    let theme_default = |key: &str, fallback: &'static str| {
        theme
            .defaults
            .get(key)
            .map_or(fallback, String::as_str)
            .to_string()
    };
    let (clock, clock_set) = current(&Key::ClockFormat, &theme_default("clockFormat", "24h"));
    let (ampm, ampm_set) = current(
        &Key::ClockShowAmPm,
        &theme_default("clockShowAmPm", "false"),
    );
    let (date, date_set) = current(&Key::DateFormat, "");
    let no_clock = (!supports.clock_format).then(|| format!("{name} doesn't support clock format"));
    out.push(Field {
        key: Key::ClockFormat,
        label: "Clock".into(),
        kind: FieldKind::Choice(vec![
            ("24h".into(), "24-hour".into()),
            ("12h".into(), "12-hour".into()),
        ]),
        value: clock.clone(),
        is_set: clock_set,
        disabled: no_clock.clone(),
        group: None,
    });
    out.push(Field {
        key: Key::ClockShowAmPm,
        label: "Show AM/PM".into(),
        kind: FieldKind::Bool,
        value: ampm,
        is_set: ampm_set,
        disabled: no_clock
            .or_else(|| (clock != "12h").then(|| "only with the 12-hour clock".into())),
        group: None,
    });
    out.push(Field {
        key: Key::DateFormat,
        label: "Date format".into(),
        kind: FieldKind::Choice(
            std::iter::once((String::new(), "Theme default".to_string()))
                .chain(
                    settings::DATE_PRESETS
                        .iter()
                        .map(|p| (p.format.to_string(), p.label.to_string())),
                )
                .collect(),
        ),
        value: date,
        is_set: date_set,
        disabled: (!supports.date_format).then(|| format!("{name} doesn't support date format")),
        group: None,
    });
    out
}

// Standard customisations: every supported one, and for an area the theme doesn't support, one row saying why.
fn standard_fields(theme: &Theme, config: &UserConfig) -> Vec<Field> {
    let saved = |key: &str| {
        let k = Key::Option {
            theme: theme.id.clone(),
            key: key.to_string(),
        };
        settings::get(config, &k)
            .ok()
            .flatten()
            .filter(|v| custom::check(theme, key, v).is_ok())
    };
    let make = |key: &str,
                label: &str,
                area: Area,
                kind: FieldKind,
                default: String,
                disabled: Option<String>| {
        let value = saved(key);
        Field {
            key: Key::Option {
                theme: theme.id.clone(),
                key: key.to_string(),
            },
            label: label.to_string(),
            kind,
            is_set: value.is_some(),
            value: value.unwrap_or(default),
            disabled,
            group: Some(area.title()),
        }
    };
    let choice = |pairs: &[(&str, &str)]| {
        FieldKind::Choice(
            pairs
                .iter()
                .map(|(v, l)| (v.to_string(), l.to_string()))
                .collect(),
        )
    };
    let sup = &theme.manifest.supports;
    let mut out = Vec::new();
    for area in [
        Area::Background,
        Area::Appearance,
        Area::Colours,
        Area::Fonts,
        Area::Motion,
    ] {
        let settings: Vec<_> = custom::SETTINGS.iter().filter(|s| s.area == area).collect();
        let lead = settings[0];
        if let Some(why) = custom::unsupported(theme, lead.key).filter(|_| {
            settings
                .iter()
                .all(|s| custom::unsupported(theme, s.key).is_some())
        }) {
            out.push(make(
                lead.key,
                lead.label,
                area,
                FieldKind::Text,
                String::new(),
                Some(why),
            ));
            continue;
        }
        for s in settings {
            if custom::unsupported(theme, s.key).is_some() {
                continue;
            }
            let (kind, default) = match s.kind {
                Kind::Background => (
                    FieldKind::Media(
                        [custom::IMAGE_EXT, custom::ANIMATED_EXT, custom::VIDEO_EXT]
                            .concat()
                            .iter()
                            .map(|e| format!("*.{e}"))
                            .collect(),
                    ),
                    String::new(),
                ),
                Kind::Fit => (
                    choice(&[("cover", "Fill the screen"), ("contain", "Show all of it")]),
                    "cover".into(),
                ),
                Kind::Percent => (FieldKind::Int { min: 0, max: 80 }, "0".into()),
                Kind::Variant => {
                    let mut pairs: Vec<(String, String)> = sup
                        .variants
                        .iter()
                        .map(|v| (v.clone(), capitalise(v)))
                        .collect();
                    pairs.push(("auto".into(), "Follow the desktop".into()));
                    (
                        FieldKind::Choice(pairs),
                        sup.default_variant.clone().unwrap_or_default(),
                    )
                }
                Kind::Color => (
                    FieldKind::Color { generate: true },
                    theme
                        .defaults
                        .get(if s.key == "accent" {
                            "colorAccent"
                        } else {
                            "colorText"
                        })
                        .cloned()
                        .unwrap_or_default(),
                ),
                Kind::ColorSource => (
                    choice(&[
                        ("background", "The background"),
                        ("desktop", "My desktop wallpaper"),
                    ]),
                    "background".into(),
                ),
                Kind::Scheme => (choice(custom::SCHEMES), custom::SCHEMES[0].0.into()),
                Kind::Contrast => (
                    FieldKind::Range {
                        min: -1.0,
                        max: 1.0,
                        step: 0.25,
                    },
                    "0".into(),
                ),
                Kind::Font => (FieldKind::Font, String::new()),
                Kind::Speed => (
                    FieldKind::Range {
                        min: 0.25,
                        max: 3.0,
                        step: 0.25,
                    },
                    "1".into(),
                ),
                Kind::Curve => (
                    FieldKind::Choice(
                        std::iter::once((String::new(), "Theme default".to_string()))
                            .chain(
                                custom::CURVES
                                    .iter()
                                    .map(|c| (c.0.to_string(), c.1.to_string())),
                            )
                            .collect(),
                    ),
                    String::new(),
                ),
                Kind::Bool => (FieldKind::Bool, "false".into()),
            };
            out.push(make(s.key, s.label, area, kind, default, None));
            if s.key == "text_color" {
                for c in &theme.manifest.colors {
                    let default = theme.defaults.get(&c.key).cloned().unwrap_or_default();
                    out.push(make(
                        &c.key,
                        &c.label,
                        area,
                        FieldKind::Color { generate: true },
                        default,
                        None,
                    ));
                }
            }
        }
    }
    // Generation settings only matter once a colour is generated.
    let generating = out.iter().any(|f| {
        matches!(f.kind, FieldKind::Color { generate: true }) && f.value == custom::GENERATE
    });
    for f in &mut out {
        if let Key::Option { key, .. } = &f.key {
            if matches!(
                key.as_str(),
                "color_source" | "color_scheme" | "color_contrast"
            ) && !generating
            {
                f.disabled = Some("only when a colour is generated".into());
            }
            if matches!(key.as_str(), "background_fit" | "background_dim")
                && saved("background").is_none()
            {
                f.disabled = Some("only with a custom background".into());
            }
        }
    }
    out
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use std::path::Path;

    fn theme(id: &str) -> Theme {
        let cat = Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes"))
            .unwrap()
            .0;
        cat.into_themes().into_iter().find(|t| t.id == id).unwrap()
    }

    fn field<'a>(fields: &'a [Field], label: &str) -> &'a Field {
        fields.iter().find(|f| f.label == label).unwrap()
    }

    #[test]
    fn values_fall_back_to_theme_conf_defaults() {
        let f = fields(&theme("terraria"), &UserConfig::default());
        let mode = field(&f, "Background");
        assert_eq!((mode.value.as_str(), mode.is_set), ("random", false));
    }

    #[test]
    fn a_fixed_background_is_disabled_until_the_mode_is_fixed() {
        let t = theme("terraria");
        let f = fields(&t, &UserConfig::default());
        assert_eq!(
            field(&f, "Fixed background").disabled.as_deref(),
            Some("only when Background is Fixed")
        );
        let cfg = UserConfig::parse("[themes.terraria]\nbackground_mode = \"static\"\n").unwrap();
        assert_eq!(field(&fields(&t, &cfg), "Fixed background").disabled, None);
    }

    #[test]
    fn globals_are_shown_but_disabled_with_a_reason_when_unsupported() {
        let f = fields(&theme("ninesols"), &UserConfig::default());
        assert_eq!(
            field(&f, "Clock").disabled.as_deref(),
            Some("Nine Sols doesn't support clock format"),
            "Nine Sols shows no clock"
        );
        assert!(field(&f, "Date format").disabled.is_some());
        let osu = fields(&theme("osu"), &UserConfig::default());
        assert_eq!(field(&osu, "Clock").disabled, None);
        assert!(
            field(&osu, "Date format").disabled.is_some(),
            "osu! shows a clock but no date"
        );
    }

    #[test]
    fn an_invalid_saved_value_shows_the_default_instead() {
        let cfg = UserConfig::parse("[themes.osu]\ngameMode = \"arcade\"\n").unwrap();
        let f = fields(&theme("osu"), &cfg);
        let g = field(&f, "Start screen");
        assert_eq!((g.value.as_str(), g.is_set), ("menu", false));
    }

    #[test]
    fn an_unset_clock_shows_the_theme_own_format() {
        let f = fields(&theme("clockwork/orbital"), &UserConfig::default());
        let clock = field(&f, "Clock");
        assert_eq!(
            (clock.value.as_str(), clock.is_set),
            ("12h", false),
            "Orbital is a 12-hour design"
        );
        assert_eq!(field(&f, "Show AM/PM").disabled, None);
        let cfg = UserConfig::parse("[clock]\nformat = \"24h\"\n").unwrap();
        let all = fields(&theme("clockwork/orbital"), &cfg);
        let clock = field(&all, "Clock");
        assert_eq!(
            (clock.label.as_str(), clock.value.as_str(), clock.is_set),
            ("Clock", "24h", true)
        );
    }

    #[test]
    fn date_format_is_a_choice_of_presets_led_by_the_theme_default() {
        let f = fields(&theme("osu"), &UserConfig::default());
        let FieldKind::Choice(choices) = &field(&f, "Date format").kind else {
            panic!("date format should be a choice");
        };
        assert_eq!(choices[0], (String::new(), "Theme default".to_string()));
        assert_eq!(choices.len(), 1 + settings::DATE_PRESETS.len());
    }
}
