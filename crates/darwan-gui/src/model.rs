use std::path::PathBuf;

use serde_json::{Value, json};

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::environment::Environment;
use darwan_core::form::{self, FieldKind};
use darwan_core::gallery::{self, ListRow};
use darwan_core::manifest::Background;

fn reason(r: &Result<(), String>) -> String {
    r.clone().err().unwrap_or_default()
}

fn background(theme: &Theme) -> &'static str {
    match theme.manifest.background {
        Background::Color => "color",
        Background::Image => "image",
        Background::Video => "video",
    }
}

pub fn rows(catalog: &Catalog, config: &UserConfig, query: &str) -> Value {
    let lock = config.theme(Target::Lock).ok().flatten().unwrap_or("");
    let sddm = config.theme(Target::Sddm).ok().flatten().unwrap_or("");
    gallery::list_rows(catalog, query)
        .into_iter()
        .map(|row| match row {
            ListRow::Section { title, count } => {
                json!({ "kind": "section", "title": title, "count": count })
            }
            ListRow::Theme(i) => {
                let t = &catalog.themes()[i];
                json!({
                    "kind": "theme",
                    "id": t.id,
                    "name": t.manifest.name,
                    "background": background(t),
                    "preview": t.preview.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
                    "isLock": t.id == lock,
                    "isSddm": t.id == sddm,
                    "missingFonts": t.missing_fonts(),
                })
            }
        })
        .collect()
}

pub fn details(theme: &Theme, config: &UserConfig) -> Value {
    let m = &theme.manifest;
    let is = |target| config.theme(target).ok().flatten() == Some(theme.id.as_str());
    json!({
        "isLock": is(Target::Lock),
        "isSddm": is(Target::Sddm),
        "id": theme.id,
        "name": gallery::display_name(theme),
        "author": m.author,
        "background": background(theme),
        "dir": theme.dir.display().to_string(),
        "fonts": m.fonts.iter().map(|f| json!({
            "file": f.file,
            "family": f.family,
            "license": f.license,
            "url": f.url,
            "installed": theme.dir.join("font").join(&f.file).is_file(),
        })).collect::<Vec<_>>(),
    })
}

pub fn fields(theme: &Theme, config: &UserConfig) -> Value {
    form::fields(theme, config)
        .into_iter()
        .map(|f| {
            let mut v = json!({
                "key": f.key.to_string(),
                "label": f.label,
                "value": f.value,
                "isSet": f.is_set,
                "disabled": f.disabled.unwrap_or_default(),
                "group": f.group.unwrap_or_default(),
            });
            let kind = match f.kind {
                FieldKind::Bool => "bool",
                FieldKind::Choice(choices) => {
                    v["choices"] = choices
                        .into_iter()
                        .map(|(value, label)| json!({ "value": value, "label": label }))
                        .collect();
                    "choice"
                }
                FieldKind::Int { min, max } => {
                    v["min"] = min.into();
                    v["max"] = max.into();
                    "int"
                }
                FieldKind::Range { min, max, step } => {
                    v["min"] = min.into();
                    v["max"] = max.into();
                    v["step"] = step.into();
                    "range"
                }
                FieldKind::Color { generate } => {
                    v["generate"] = generate.into();
                    v["swatches"] = swatches(theme).into();
                    "color"
                }
                FieldKind::File(filters) => {
                    v["filters"] = filters.into();
                    "file"
                }
                FieldKind::Media(filters) => {
                    v["filters"] = filters.into();
                    "media"
                }
                FieldKind::Font => "font",
                FieldKind::Text => "text",
            };
            v["kind"] = kind.into();
            v
        })
        .collect()
}

// The theme's own colours, offered first in its colour pickers.
fn swatches(theme: &Theme) -> Vec<String> {
    let mut out: Vec<String> = theme
        .defaults
        .values()
        .filter(|v| darwan_core::custom::is_hex_color(v))
        .map(|v| v.to_lowercase())
        .collect();
    out.sort();
    out.dedup();
    out
}

// An empty string means available; anything else is the reason shown instead.
pub fn availability(env: &Environment) -> Value {
    json!({
        "wayland": reason(&env.wayland),
        "sddm": reason(&env.sddm),
        "helper": reason(&env.helper),
        "sddmPreview": reason(&env.sddm_preview()),
    })
}

// Output that isn't in doctor's and check's line format still shows, as plain text.
pub fn report(output: &str) -> Value {
    let mut rows: Vec<(&str, String)> = Vec::new();
    for line in output.lines().map(str::trim_end) {
        let row = if let Some(t) = line.strip_prefix("ok    ") {
            ("ok", t)
        } else if let Some(t) = line.strip_prefix("warn  ") {
            ("warn", t)
        } else if let Some(t) = line.strip_prefix("FAIL  ") {
            ("fail", t)
        } else if let Some(t) = line.strip_prefix("        ") {
            ("detail", t)
        } else if line.is_empty() {
            continue;
        } else {
            ("text", line)
        };
        rows.push((row.0, row.1.trim().to_string()));
    }
    for i in 0..rows.len() {
        let next_is_status = rows
            .get(i + 1)
            .is_some_and(|(k, _)| matches!(*k, "ok" | "warn" | "fail"));
        if rows[i].0 == "text" && next_is_status {
            rows[i].0 = "heading";
        }
    }
    rows.into_iter()
        .map(|(kind, text)| json!({ "kind": kind, "text": text }))
        .collect()
}

// darwan-gui ships next to darwan; a dev build finds it in the same target directory.
pub fn darwan_exe() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("darwan")))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("darwan"))
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

    fn theme_row<'a>(rows: &'a Value, id: &str) -> &'a Value {
        rows.as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
    }

    #[test]
    fn rows_keep_the_tui_sections_and_mark_the_configured_themes() {
        let cfg =
            UserConfig::parse("[lock]\ntheme = \"osu\"\n[sddm]\ntheme = \"terraria\"\n").unwrap();
        let rows = rows(&catalog(), &cfg, "");
        let sections: Vec<&str> = rows
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["kind"] == "section")
            .map(|r| r["title"].as_str().unwrap())
            .collect();
        assert_eq!(sections, ["Clockwork", "Pixel", gallery::OTHER_SECTION]);
        let osu = theme_row(&rows, "osu");
        assert_eq!(
            (osu["isLock"].as_bool(), osu["isSddm"].as_bool()),
            (Some(true), Some(false))
        );
        assert_eq!(theme_row(&rows, "terraria")["isSddm"], true);
    }

    #[test]
    fn a_disabled_field_carries_its_reason_so_the_form_can_show_it() {
        let cat = catalog();
        let f = fields(cat.get("terraria").unwrap(), &UserConfig::default());
        let fixed = f
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["label"] == "Fixed background")
            .unwrap();
        assert_eq!(fixed["disabled"], "only when Background is Fixed");
        assert_eq!(fixed["kind"], "choice");
        assert!(!fixed["choices"].as_array().unwrap().is_empty());
        assert_eq!(
            fixed["key"], "terraria.background_index",
            "keys are what settings::set parses"
        );
    }

    #[test]
    fn unavailable_actions_report_why() {
        let env = Environment {
            wayland: Ok(()),
            sddm: Err("SDDM (Qt 6) is not installed".into()),
            helper: Err("SDDM (Qt 6) is not installed".into()),
        };
        let a = availability(&env);
        assert_eq!(a["wayland"], "");
        assert_eq!(a["sddmPreview"], "SDDM (Qt 6) is not installed");
    }

    #[test]
    fn details_say_which_licensed_fonts_are_missing() {
        let cat = catalog();
        let d = details(cat.get("terraria").unwrap(), &UserConfig::default());
        let font = &d["fonts"][0];
        assert_eq!(font["file"], "Andy Bold.ttf");
        assert_eq!(
            font["installed"].as_bool(),
            Some(cat.get("terraria").unwrap().missing_fonts() == 0)
        );
    }

    #[test]
    fn doctor_output_becomes_headed_groups_with_a_status_per_line() {
        let out = "Lockscreen\nok    quickshell is installed\nwarn  osu: font Torus missing\n\nThemes\nFAIL  bad: cannot load\n        darwan.toml: oops\n\nNo problems.\n";
        let kinds: Vec<(String, String)> = report(out)
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["kind"].as_str().unwrap().into(),
                    r["text"].as_str().unwrap().into(),
                )
            })
            .collect();
        let expected = [
            ("heading", "Lockscreen"),
            ("ok", "quickshell is installed"),
            ("warn", "osu: font Torus missing"),
            ("heading", "Themes"),
            ("fail", "bad: cannot load"),
            ("detail", "darwan.toml: oops"),
            ("text", "No problems."),
        ];
        assert_eq!(
            kinds,
            expected.map(|(k, t)| (k.to_string(), t.to_string())),
            "a summary line is not a heading, so it isn't styled as one"
        );
    }

    #[test]
    fn output_without_status_lines_stays_plain_text() {
        let r = report("darwan: unknown theme \"x\"\n");
        assert_eq!(
            r[0]["kind"], "text",
            "a failed command's error is shown as-is"
        );
    }

    #[test]
    fn customisations_carry_their_heading_and_colours_offer_the_themes_own_as_swatches() {
        let cat = catalog();
        let rainy = cat.get("pixel-rainyroom").unwrap();
        let rows = fields(rainy, &UserConfig::default());
        let rows = rows.as_array().unwrap();
        let accent = rows
            .iter()
            .find(|r| r["key"] == "pixel-rainyroom.accent")
            .unwrap();
        assert_eq!(
            (accent["group"].as_str(), accent["kind"].as_str()),
            (Some("Colours"), Some("color"))
        );
        assert_eq!(accent["generate"], true);
        let swatches: Vec<&str> = accent["swatches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(
            swatches.contains(&"#e6bb5c") && swatches.contains(&"#2f9eff"),
            "{swatches:?}"
        );
        let speed = rows
            .iter()
            .find(|r| r["key"] == "pixel-rainyroom.motion_speed")
            .unwrap();
        assert_eq!(
            (speed["kind"].as_str(), speed["step"].as_f64()),
            (Some("range"), Some(0.25))
        );
        let clock = rows.iter().find(|r| r["key"] == "clock.format").unwrap();
        assert_eq!(clock["group"], "", "globals have no customisation heading");
    }
}
