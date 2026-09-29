use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::custom;
use darwan_core::environment::Environment;
use darwan_core::form::{self, FieldKind};
use darwan_core::gallery::{self, ListRow};
use darwan_core::manifest::Background;
use darwan_core::settings::Key;

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
        "still": theme.preview.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
        "fonts": m.fonts.iter().map(|f| json!({
            "file": f.file,
            "family": f.family,
            "license": f.license,
            "url": f.url,
            "installed": theme.dir.join("font").join(&f.file).is_file(),
        })).collect::<Vec<_>>(),
    })
}

// The Wall's cards; `loop` is the short animation a theme may ship, played only while the card is hovered.
fn card(theme: &Theme, lock: &str, sddm: &str) -> Value {
    let path =
        |p: Option<std::path::PathBuf>| p.map(|p| p.display().to_string()).unwrap_or_default();
    json!({
        "id": theme.id,
        "name": theme.manifest.name,
        "title": gallery::display_name(theme),
        "background": background(theme),
        "still": path(theme.preview.clone()),
        "loop": path(Some(theme.dir.join("preview.webp")).filter(|p| p.is_file())),
        "isLock": theme.id == lock,
        "isSddm": theme.id == sddm,
        "missingFonts": theme.missing_fonts(),
    })
}

// A chip above the Wall; `key` is what `wall` filters on.
struct Chip {
    key: String,
    label: String,
    keeps: Box<dyn Fn(&Theme) -> bool>,
}

fn chip(
    key: impl Into<String>,
    label: impl Into<String>,
    keeps: impl Fn(&Theme) -> bool + 'static,
) -> Chip {
    Chip {
        key: key.into(),
        label: label.into(),
        keeps: Box::new(keeps),
    }
}

// All, one per family, then kinds; their counts ignore the search.
fn chips(catalog: &Catalog, lock: &str, sddm: &str) -> Vec<Chip> {
    let mut out = vec![chip("all", "All", |_| true)];
    for row in gallery::list_rows(catalog, "") {
        if let ListRow::Section { title, .. } = row {
            let family = (title != gallery::OTHER_SECTION).then(|| title.clone());
            out.push(chip(format!("family:{title}"), title, move |t| {
                t.manifest.family == family
            }));
        }
    }
    let (lock, sddm) = (lock.to_string(), sddm.to_string());
    out.push(chip("video", "Video backgrounds", |t| {
        t.manifest.background == Background::Video
    }));
    out.push(chip("fonts", "Brings its own font", |t| {
        !t.manifest.fonts.is_empty()
    }));
    out.push(chip("inuse", "In use", move |t| {
        t.id == lock || t.id == sddm
    }));
    out
}

// Everything the Wall shows: the two gates, the chips, and the sections left after the chip and the search.
pub fn wall(catalog: &Catalog, config: &UserConfig, query: &str, filter: &str) -> Value {
    let lock = config.theme(Target::Lock).ok().flatten().unwrap_or("");
    let sddm = config.theme(Target::Sddm).ok().flatten().unwrap_or("");
    let chips = chips(catalog, lock, sddm);
    let keep = chips.iter().find(|c| c.key == filter).unwrap_or(&chips[0]);
    let themes = catalog.themes();
    let mut sections: Vec<Value> = Vec::new();
    let mut order = Vec::new();
    for row in gallery::list_rows(catalog, query) {
        match row {
            ListRow::Section { title, .. } => {
                sections.push(json!({ "title": title, "themes": [] }))
            }
            ListRow::Theme(i) if (keep.keeps)(&themes[i]) => {
                let mut c = card(&themes[i], lock, sddm);
                // Its place in the Wall's order, which arrow keys and the Stage's strip step through.
                c["index"] = order.len().into();
                order.push(json!({ "id": c["id"], "title": c["title"], "still": c["still"] }));
                let section = sections
                    .last_mut()
                    .expect("a theme row follows its section");
                section["themes"].as_array_mut().unwrap().push(c);
            }
            ListRow::Theme(_) => {}
        }
    }
    sections.retain(|s| !s["themes"].as_array().unwrap().is_empty());
    let gate = |id: &str| catalog.get(id).map_or(Value::Null, |t| card(t, lock, sddm));
    json!({
        "gates": { "lock": gate(lock), "sddm": gate(sddm) },
        "chips": chips.iter().map(|c| json!({
            "key": c.key,
            "label": c.label,
            "count": themes.iter().filter(|t| (c.keeps)(t)).count(),
            "on": c.key == keep.key,
        })).collect::<Vec<_>>(),
        "sections": sections,
        "order": order,
    })
}

// The roles offered as "from your background" swatches: the palette's colourful ones, then its neutrals.
const IMAGE_SWATCHES: &[(&str, &str)] = &[
    ("primary", "Primary"),
    ("secondary", "Secondary"),
    ("tertiary", "Tertiary"),
    ("inverse_primary", "Primary, light"),
    ("primary_container", "Primary container"),
    ("secondary_container", "Secondary container"),
    ("tertiary_container", "Tertiary container"),
    ("on_surface", "Text"),
    ("on_surface_variant", "Soft text"),
    ("surface_variant", "Surface"),
    ("outline", "Outline"),
];

// `overlay` is what the preview gets, so a generated colour can be shown as the colour it became;
// `image` is the palette the background gives, offered for picking.
pub fn fields(
    theme: &Theme,
    config: &UserConfig,
    overlay: &BTreeMap<String, String>,
    image: Option<&custom::Palette>,
) -> Value {
    form::fields(theme, config)
        .into_iter()
        // The Screensaver window holds these, next to hypridle's settings.
        .filter(|f| {
            !matches!(
                f.key,
                Key::SaverLockAfter
                    | Key::SaverReturnAfter
                    | Key::SaverScreenOffLocked
                    | Key::SaverQuality
            )
        })
        .map(|f| {
            let mut v = json!({
                "key": f.key.to_string(),
                "label": f.label,
                "value": f.value,
                "isSet": f.is_set,
                "disabled": f.disabled.unwrap_or_default(),
                "group": f.group.unwrap_or_default(),
                "global": !matches!(f.key, Key::Option { .. }),
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
                    v["imageColours"] = image
                        .filter(|_| generate)
                        .map(|p| {
                            IMAGE_SWATCHES
                                .iter()
                                .filter_map(|(role, label)| {
                                    p.get(*role)
                                        .map(|hex| json!({ "hex": hex, "label": label }))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                        .into();
                    if let Key::Option { key, .. } = &f.key
                        && let Some(hex) = overlay.get(custom::color_contract_key(key))
                    {
                        v["shown"] = hex.as_str().into();
                    }
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

// Which control the form draws; decided here so the QML only binds.
fn control(field: &Value) -> &'static str {
    let short = |choices: &Vec<Value>| {
        choices.len() <= 3
            && choices
                .iter()
                .all(|c| c["label"].as_str().is_some_and(|l| l.chars().count() < 16))
    };
    match field["kind"].as_str().unwrap_or("") {
        "bool" => "switch",
        "choice"
            if field["key"]
                .as_str()
                .is_some_and(|k| k.ends_with(".variant")) =>
        {
            "looks"
        }
        "choice" if field["choices"].as_array().is_some_and(short) => "segmented",
        "choice" => "menu",
        "int" | "range" => "slider",
        "color" => "well",
        "media" => "media",
        "font" => "font",
        "file" => "file",
        _ => "text",
    }
}

fn names(labels: &[&str]) -> String {
    match labels {
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        [] => String::new(),
    }
}

// One line per reason, naming every setting it holds back, instead of the same reason under each.
fn notes(fields: &[Value]) -> Vec<String> {
    let mut reasons: Vec<(&str, Vec<&str>)> = Vec::new();
    for f in fields {
        let reason = f["disabled"].as_str().unwrap_or("");
        if reason.is_empty() {
            continue;
        }
        let label = f["label"].as_str().unwrap_or("");
        match reasons.iter_mut().find(|(r, _)| *r == reason) {
            Some((_, labels)) => labels.push(label),
            None => reasons.push((reason, vec![label])),
        }
    }
    reasons
        .into_iter()
        .map(|(reason, labels)| match reason.strip_prefix("only ") {
            Some(rest) => format!(
                "{} {} only {rest}.",
                names(&labels),
                if labels.len() == 1 {
                    "applies"
                } else {
                    "apply"
                }
            ),
            None => format!("{}: {}", names(&labels), reason.trim_end_matches('.')) + ".",
        })
        .collect()
}

fn with_control(mut field: Value) -> Value {
    field["control"] = control(&field).into();
    let key = field["key"].as_str().unwrap_or("");
    field["unit"] = if key.ends_with("_dim") {
        "%"
    } else if key.ends_with(".motion_speed") {
        "\u{d7}"
    } else {
        ""
    }
    .into();
    field
}

fn changed(fields: &[Value]) -> usize {
    fields.iter().filter(|f| f["isSet"] == true).count()
}

// A theme's settings as the Inspector groups them, from `fields`; the global ones are in `globals`.
pub fn form(fields: &Value) -> Value {
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    // A theme's own options carry no heading of their own; they come first, as "Options".
    for f in fields
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["global"] != true)
    {
        let group = Some(f["group"].as_str().unwrap_or(""))
            .filter(|g| !g.is_empty())
            .unwrap_or("Options");
        match groups.iter_mut().find(|(g, _)| g == group) {
            Some((_, list)) => list.push(with_control(f.clone())),
            None => groups.push((group.to_string(), vec![with_control(f.clone())])),
        }
    }
    let total = groups.iter().map(|(_, list)| changed(list)).sum::<usize>();
    json!({
        "changed": total,
        "groups": groups.iter().map(|(title, list)| json!({
            "title": title,
            "changed": changed(list),
            "notes": notes(list),
            "fields": list,
        })).collect::<Vec<_>>(),
    })
}

pub fn globals(fields: &Value) -> Value {
    let list: Vec<Value> = fields
        .as_array()
        .into_iter()
        .flatten()
        .filter(|f| f["global"] == true)
        .map(|f| with_control(f.clone()))
        .collect();
    json!({ "changed": changed(&list), "notes": notes(&list), "fields": list })
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

// What the Wall's Doctor button shows before anything runs: cheap checks only; the button runs the full doctor.
pub fn health(
    env: &Environment,
    catalog: &Catalog,
    saver_problem: &str,
    idle_shells: &[&str],
) -> Value {
    let mut problems = Vec::new();
    if let Err(e) = &env.wayland {
        problems.push(format!("previews and locking can't run: {e}"));
    }
    if let Err(e) = &env.helper {
        problems.push(format!("the login screen can't be set: {e}"));
    }
    let fonts: usize = catalog.themes().iter().map(|t| t.missing_fonts()).sum();
    if fonts > 0 {
        problems.push(format!(
            "{fonts} font{} a theme needs {} missing",
            if fonts == 1 { "" } else { "s" },
            if fonts == 1 { "is" } else { "are" }
        ));
    }
    match saver_problem {
        "" => {}
        "missing" => {
            problems.push("hypridle isn't installed, so the screensaver can't start".into())
        }
        "stopped" => problems.push("hypridle isn't running, so the screensaver can't start".into()),
        "noconf" => problems.push("the screensaver isn't set up yet".into()),
        _ => problems.push("hypridle.conf needs a fix".into()),
    }
    for name in idle_shells {
        problems.push(format!("{name} also acts when you step away"));
    }
    json!({ "ok": problems.is_empty(), "problems": problems })
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

    #[test]
    fn a_disabled_field_carries_its_reason_so_the_form_can_show_it() {
        let cat = catalog();
        let f = fields(
            cat.get("terraria").unwrap(),
            &UserConfig::default(),
            &BTreeMap::new(),
            None,
        );
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

    // A colour set to Generate shows the colour it became, not an empty swatch.
    #[test]
    fn a_colour_shows_what_the_preview_gets() {
        let cat = catalog();
        let cfg = UserConfig::parse("[themes.material-you]\naccent = \"generate\"\n").unwrap();
        let overlay = BTreeMap::from([("colorAccent".to_string(), "#87521c".to_string())]);
        let image = custom::Palette::from([("primary".to_string(), "#8e4e00".to_string())]);
        let f = fields(
            cat.get("material-you").unwrap(),
            &cfg,
            &overlay,
            Some(&image),
        );
        let row = |key: &str| {
            f.as_array()
                .unwrap()
                .iter()
                .find(|r| r["key"] == key)
                .unwrap()
                .clone()
        };
        let accent = row("material-you.accent");
        assert_eq!(
            (accent["value"].as_str(), accent["shown"].as_str()),
            (Some("generate"), Some("#87521c"))
        );
        assert!(row("material-you.text_color").get("shown").is_none());
        assert_eq!(
            accent["imageColours"][0],
            json!({ "hex": "#8e4e00", "label": "Primary" }),
            "the background's colours are offered to pick from"
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
        let rows = fields(rainy, &UserConfig::default(), &BTreeMap::new(), None);
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
    fn cards(wall: &Value) -> Vec<String> {
        wall["sections"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["themes"].as_array().unwrap().iter())
            .map(|t| t["id"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn the_wall_filters_by_chip_and_search_and_the_stage_follows_its_order() {
        let cat = catalog();
        let cfg =
            UserConfig::parse("[lock]\ntheme = \"osu\"\n[sddm]\ntheme = \"terraria\"\n").unwrap();
        let all = wall(&cat, &cfg, "", "all");
        assert_eq!(cards(&all).len(), cat.themes().len());
        assert_eq!(
            all["order"].as_array().unwrap().len(),
            cat.themes().len(),
            "the strip steps through what the Wall shows"
        );
        assert_eq!(
            all["sections"][1]["themes"][0]["index"],
            all["sections"][0]["themes"].as_array().unwrap().len(),
            "indexes run on across sections"
        );
        assert_eq!(all["gates"]["lock"]["id"], "osu");
        assert_eq!(all["gates"]["sddm"]["id"], "terraria");

        let in_use = wall(&cat, &cfg, "", "inuse");
        assert_eq!(cards(&in_use), ["osu", "terraria"]);
        let chip = |w: &Value, key: &str| {
            w["chips"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["key"] == key)
                .unwrap()
                .clone()
        };
        assert_eq!(chip(&in_use, "inuse")["on"], true);
        assert_eq!(
            chip(&in_use, "all")["count"],
            cat.themes().len(),
            "chip counts ignore the chip that is on"
        );

        let video = wall(&cat, &cfg, "", "video");
        assert!(
            cards(&video)
                .iter()
                .all(|id| cat.get(id).unwrap().manifest.background == Background::Video)
        );
        assert!(
            video["sections"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["title"] != "Clockwork"),
            "a section with nothing left in it is dropped, not shown empty"
        );

        let pixel = wall(&cat, &cfg, "", "family:Pixel");
        assert!(cards(&pixel).iter().all(|id| id.starts_with("pixel-")));
        let search = wall(&cat, &cfg, "rainy", "family:Pixel");
        assert_eq!(cards(&search), ["pixel-rainyroom"]);
        assert_eq!(
            search["order"][0]["id"], "pixel-rainyroom",
            "the strip carries what it draws: id, title and still"
        );
        assert_eq!(
            cards(&wall(&cat, &cfg, "", "bogus")).len(),
            cat.themes().len(),
            "an unknown chip shows everything"
        );
    }

    #[test]
    fn the_form_groups_a_themes_settings_picks_their_controls_and_says_once_why_some_are_off() {
        let cat = catalog();
        let f = fields(
            cat.get("clockwork/neo-orbital").unwrap(),
            &UserConfig::default(),
            &BTreeMap::new(),
            None,
        );
        let form = form(&f);
        let groups = form["groups"].as_array().unwrap();
        let titles: Vec<&str> = groups
            .iter()
            .map(|g| g["title"].as_str().unwrap())
            .collect();
        assert_eq!(
            titles,
            ["Background", "Appearance", "Colours", "Fonts", "Motion"],
            "globals stay out of a theme's tab"
        );
        let field = |key: &str| {
            groups
                .iter()
                .flat_map(|g| g["fields"].as_array().unwrap())
                .find(|f| f["key"] == key)
                .unwrap()
                .clone()
        };
        assert_eq!(field("clockwork/neo-orbital.variant")["control"], "looks");
        assert_eq!(
            field("clockwork/neo-orbital.background_fit")["control"],
            "segmented",
            "two short choices fit side by side"
        );
        assert_eq!(
            field("clockwork/neo-orbital.color_scheme")["control"],
            "menu",
            "nine choices don't"
        );
        assert_eq!(
            field("clockwork/neo-orbital.background_dim")["control"],
            "slider"
        );
        assert_eq!(field("clockwork/neo-orbital.background_dim")["unit"], "%");
        assert_eq!(
            field("clockwork/neo-orbital.motion_speed")["unit"],
            "\u{d7}"
        );
        assert_eq!(field("clockwork/neo-orbital.colorGreen")["control"], "well");
        assert_eq!(
            field("clockwork/neo-orbital.reduce_motion")["control"],
            "switch"
        );
        assert_eq!(
            groups[0]["notes"],
            json!(["Fit and Dim apply only with a custom background."])
        );
        assert_eq!(form["changed"], 0);

        let still = cat
            .get("clockwork/neo-orbital")
            .unwrap()
            .preview
            .clone()
            .unwrap();
        let cfg = UserConfig::parse(&format!(
            "[themes.\"clockwork/neo-orbital\"]\nbackground = {:?}\nmotion_speed = \"2\"\n",
            still.display().to_string()
        ))
        .unwrap();
        let form = super::form(&fields(
            cat.get("clockwork/neo-orbital").unwrap(),
            &cfg,
            &BTreeMap::new(),
            None,
        ));
        assert_eq!(
            form["changed"], 2,
            "the tab's badge counts what differs from the theme's defaults"
        );
        assert_eq!(
            form["groups"][0]["notes"],
            json!([]),
            "a custom background unlocks Fit and Dim"
        );
    }

    #[test]
    fn globals_are_the_settings_every_theme_shares() {
        let cat = catalog();
        let cfg = UserConfig::parse("[clock]\nformat = \"12h\"\n").unwrap();
        let f = fields(cat.get("osu").unwrap(), &cfg, &BTreeMap::new(), None);
        let g = globals(&f);
        let keys: Vec<&str> = g["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["key"].as_str().unwrap())
            .collect();
        assert_eq!(keys, ["clock.format", "clock.show_ampm", "date.format"]);
        assert_eq!(g["changed"], 1);
        assert_eq!(g["fields"][0]["control"], "segmented");
        let form = form(&f);
        assert_eq!(
            form["groups"][0]["title"], "Options",
            "a theme's own options aren't mistaken for globals"
        );
        assert_eq!(form["groups"][0]["fields"][0]["key"], "osu.gameMode");
    }

    #[test]
    fn a_reason_that_is_not_a_condition_reads_as_a_label_and_a_sentence() {
        let f = |label: &str, reason: &str| json!({ "label": label, "disabled": reason });
        assert_eq!(
            notes(&[
                f(
                    "Light or dark",
                    "Coffee has one look: its artwork is the design"
                ),
                f("Fit", "")
            ]),
            ["Light or dark: Coffee has one look: its artwork is the design."]
        );
        assert_eq!(
            notes(&[
                f("A", "only when X"),
                f("B", "only when X"),
                f("C", "only when X")
            ]),
            ["A, B and C apply only when X."]
        );
    }

    #[test]
    fn health_names_each_problem_the_wall_can_see_without_running_doctor() {
        let env = Environment {
            wayland: Ok(()),
            sddm: Ok(()),
            helper: Err("the helper is not installed".into()),
        };
        let h = health(&env, &catalog(), "stopped", &[]);
        assert_eq!(h["ok"], false);
        let problems: Vec<&str> = h["problems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .collect();
        assert!(problems[0].starts_with("the login screen can't be set"));
        assert!(
            problems
                .iter()
                .any(|p| p.contains("hypridle isn't running"))
        );
        let env = Environment {
            wayland: Ok(()),
            sddm: Ok(()),
            helper: Ok(()),
        };
        let fonts = catalog()
            .themes()
            .iter()
            .map(|t| t.missing_fonts())
            .sum::<usize>();
        assert_eq!(
            health(&env, &catalog(), "", &[])["ok"],
            fonts == 0,
            "nothing to look at means a green light"
        );
        assert!(
            health(&env, &catalog(), "", &["DMS"])["problems"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "DMS also acts when you step away"),
            "a shell's own idle timer is something to look at"
        );
    }
}
