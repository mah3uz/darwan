use crate::catalog::Theme;
use crate::config::UserConfig;
use crate::manifest::OptionKind;
use crate::settings::{self, Key};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Bool,
    Choice(Vec<(String, String)>),
    Int { min: i64, max: i64 },
    Color,
    File(Vec<String>),
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
                    min: opt.min.unwrap_or(i64::MIN),
                    max: opt.max.unwrap_or(i64::MAX),
                },
                OptionKind::Color => FieldKind::Color,
                OptionKind::File => FieldKind::File(opt.filters.clone()),
            };
            Field {
                key,
                label: opt.label.clone(),
                kind,
                value,
                is_set,
                disabled: None,
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
    });
    out.push(Field {
        key: Key::ClockShowAmPm,
        label: "Show AM/PM".into(),
        kind: FieldKind::Bool,
        value: ampm,
        is_set: ampm_set,
        disabled: no_clock
            .or_else(|| (clock != "12h").then(|| "only with the 12-hour clock".into())),
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
    });
    out
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
        let clock = &fields(&theme("clockwork/orbital"), &cfg)[2];
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
