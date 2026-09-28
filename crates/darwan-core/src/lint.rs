use std::collections::BTreeSet;

use crate::catalog::Theme;
use crate::ini;
use crate::manifest::OptionKind;

pub fn lint(theme: &Theme) -> Vec<String> {
    let mut problems = Vec::new();
    let m = &theme.manifest;

    let mut seen = BTreeSet::new();
    for opt in &m.options {
        let key = &opt.key;
        if !seen.insert(key) {
            problems.push(format!("option {key:?} is declared twice"));
        }
        match opt.kind {
            OptionKind::Enum if opt.choices.is_empty() => {
                problems.push(format!("enum option {key:?} has no choices"))
            }
            OptionKind::Int if opt.min.is_none() || opt.max.is_none() => {
                problems.push(format!("int option {key:?} needs min and max"))
            }
            _ => {}
        }
        // theme.conf is the only source of defaults, and SDDM reads it directly.
        match theme.defaults.get(key) {
            None => problems.push(format!("option {key:?} has no default in theme.conf")),
            Some(v) => {
                if let Err(e) = opt.check(v) {
                    problems.push(format!("theme.conf default for {key:?}: {e}"));
                }
            }
        }
        for (dep, value) in &opt.enabled_when {
            match m.option(dep) {
                None => problems.push(format!("option {key:?} depends on unknown option {dep:?}")),
                Some(d) => {
                    if let Err(e) = d.check(value) {
                        problems.push(format!("option {key:?} enabled_when {dep:?}: {e}"));
                    }
                }
            }
        }
    }

    // A [supports] flag is a promise; theme.conf also says what the theme shows when it's unset.
    let qml = qml_text(&theme.dir);
    if m.supports.clock_format {
        for (key, allowed) in [
            ("clockFormat", ["12h", "24h"]),
            ("clockShowAmPm", ["true", "false"]),
        ] {
            match theme.defaults.get(key) {
                Some(v) if allowed.contains(&v.as_str()) => {}
                Some(v) => problems.push(format!(
                    "theme.conf {key}={v:?} must be {}",
                    allowed.join(" or ")
                )),
                None => problems.push(format!("supports.clock_format needs {key} in theme.conf")),
            }
        }
        if !qml.contains("config.clockFormat") {
            problems.push("supports.clock_format but no QML reads config.clockFormat".into());
        }
    }
    if m.supports.date_format && !qml.contains("config.dateFormat") {
        problems.push("supports.date_format but no QML reads config.dateFormat".into());
    }

    if m.supports.screensaver {
        // The vendored kit reads darwan.ambient itself, so only the theme's own QML counts.
        let own = qml_text_except(&theme.dir, "darwan");
        if !own.contains("Ambient {") && !own.contains("darwan.ambient") {
            problems.push(
                "supports.screensaver but the theme uses neither Ambient nor darwan.ambient".into(),
            );
        }
        // A saver runs for hours: a short Timer that always runs is JavaScript on nearly every frame, which an
        // animation isn't. One that runs only during an effect (a login windup) costs nothing while ambient.
        for ms in always_running_timers(&qml)
            .into_iter()
            .filter(|ms| *ms < 100)
        {
            problems.push(format!(
                "supports.screensaver but a Timer always runs every {ms} ms; use an animation or a slower timer"
            ));
        }
    }

    let sup = &m.supports;
    // The kit's own files don't count as the theme using it.
    let own_qml = qml_text_except(&theme.dir, "darwan");
    let uses_kit = sup.background || sup.colors || sup.motion || !sup.fonts.is_empty();
    if uses_kit {
        for file in ["Custom.qml", "Background.qml"] {
            if !theme.dir.join("darwan").join(file).is_file() {
                problems.push(format!(
                    "customisations need darwan/{file} (copy runtime/theme-kit)"
                ));
            }
        }
        if !own_qml.contains("Custom {") {
            problems.push("customisations need a `Custom { id: kit }` from the theme kit".into());
        }
    }
    for (flag, call, what) in [
        (
            sup.background,
            "Background {",
            "supports.background but no QML shows the kit's Background",
        ),
        (
            sup.colors,
            ".color(",
            "supports.colors but no QML calls color()",
        ),
        (
            sup.motion,
            ".dur(",
            "supports.motion but no QML calls dur()",
        ),
        (
            !sup.fonts.is_empty(),
            ".font(",
            "supports.fonts but no QML calls font()",
        ),
    ] {
        if flag && !own_qml.contains(call) {
            problems.push(what.into());
        }
    }
    if !sup.variants.is_empty() {
        for v in &sup.variants {
            if v != "light" && v != "dark" {
                problems.push(format!("variant {v:?} must be light or dark"));
            }
        }
        match &sup.default_variant {
            Some(d) if sup.variants.contains(d) => {
                if theme.defaults.get("colorScheme") != Some(d) {
                    problems.push(format!(
                        "supports.variants needs colorScheme={d} in theme.conf"
                    ));
                }
            }
            _ => problems
                .push("supports.variants needs a default_variant that is one of them".into()),
        }
        if !qml.contains("config.colorScheme") {
            problems.push("supports.variants but no QML reads config.colorScheme".into());
        }
    } else if sup.default_variant.is_some() {
        problems.push("default_variant without supports.variants".into());
    }
    for role in &sup.fonts {
        if role != "text" && role != "clock" {
            problems.push(format!("font role {role:?} must be text or clock"));
        }
    }
    let mut color_keys = BTreeSet::new();
    for c in &m.colors {
        if !color_keys.insert(&c.key)
            || m.option(&c.key).is_some()
            || crate::custom::setting(&c.key).is_some()
        {
            problems.push(format!("colour {:?} clashes with another key", c.key));
        }
        if !crate::custom::MATERIAL_ROLES.contains(&c.material.as_str()) {
            problems.push(format!(
                "colour {:?} follows unknown Material role {:?}",
                c.key, c.material
            ));
        }
        match theme.defaults.get(&c.key) {
            Some(v) if crate::custom::is_hex_color(v) => {}
            _ => problems.push(format!(
                "colour {:?} needs a hex default in theme.conf",
                c.key
            )),
        }
    }
    if sup.material_palette && !own_qml.contains("material_") {
        problems.push("supports.material_palette but no QML reads a material_ key".into());
    }
    if (sup.material_palette || sup.generate_by_default) && !sup.colors {
        problems.push("material_palette and generate_by_default need supports.colors".into());
    }
    // Without variants the theme's accent and text are fixed, so theme.conf states them for the form.
    if sup.colors && sup.variants.is_empty() {
        for key in ["colorAccent", "colorText"] {
            match theme.defaults.get(key) {
                Some(v) if crate::custom::is_hex_color(v) => {}
                _ => problems.push(format!("supports.colors needs a hex {key} in theme.conf")),
            }
        }
    }
    if !m.colors.is_empty() && !sup.colors {
        problems.push("[[color]] roles need supports.colors".into());
    }

    for font in &m.fonts {
        if font.file.is_empty() || font.file.contains('/') {
            problems.push(format!(
                "font file {:?} must be a plain file name",
                font.file
            ));
        }
    }

    if theme.preview.is_none() {
        problems.push("no preview.jpg or preview.png".into());
    }

    match std::fs::read_to_string(theme.dir.join("metadata.desktop")) {
        Err(e) => problems.push(format!("metadata.desktop: {e}")),
        Ok(text) => {
            let meta = metadata(&text);
            for (key, want) in [("MainScript", "Main.qml"), ("ConfigFile", "theme.conf")] {
                if meta.iter().all(|(k, v)| !(k == key && v == want)) {
                    problems.push(format!("metadata.desktop must set {key}={want}"));
                }
            }
        }
    }
    problems
}

fn qml_text_except(dir: &std::path::Path, skip: &str) -> String {
    let mut out = String::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for path in entries.filter_map(Result::ok).map(|e| e.path()) {
        if path.is_dir() {
            if path.file_name().is_none_or(|n| n != skip) {
                out.push_str(&qml_text_except(&path, skip));
            }
        } else if path.extension().is_some_and(|x| x == "qml") {
            out.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
        }
    }
    out
}

fn qml_text(dir: &std::path::Path) -> String {
    let mut out = String::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for path in entries.filter_map(Result::ok).map(|e| e.path()) {
        if path.is_dir() {
            out.push_str(&qml_text(&path));
        } else if path.extension().is_some_and(|x| x == "qml") {
            out.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
        }
    }
    out
}

// The literal intervals of Timers declared with `running: true`.
fn always_running_timers(qml: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for (start, _) in qml.match_indices("Timer {") {
        let body = &qml[start + "Timer {".len()..];
        let mut depth = 1;
        let end = body
            .char_indices()
            .find_map(|(i, c)| {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
                (depth == 0).then_some(i)
            })
            .unwrap_or(body.len());
        let body = &body[..end];
        if !body.contains("running: true") {
            continue;
        }
        let interval = body.split_once("interval:").and_then(|(_, rest)| {
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse::<u32>().ok()
        });
        out.extend(interval);
    }
    out
}

fn metadata(text: &str) -> Vec<(String, String)> {
    let mut sectionless = String::from("[General]\n");
    for line in text.lines().filter(|l| !l.trim_start().starts_with('[')) {
        sectionless.push_str(line);
        sectionless.push('\n');
    }
    ini::parse_general(&sectionless).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    fn theme_with(manifest_extra: &str, conf: &str) -> (tempfile::TempDir, Catalog) {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("t");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Main.qml"), "").unwrap();
        std::fs::write(dir.join("preview.png"), "").unwrap();
        std::fs::write(
            dir.join("metadata.desktop"),
            "[SddmGreeterTheme]\nMainScript=Main.qml\nConfigFile=theme.conf\n",
        )
        .unwrap();
        std::fs::write(dir.join("theme.conf"), conf).unwrap();
        std::fs::write(
            dir.join("darwan.toml"),
            format!("name = \"T\"\nauthor = \"a\"\nbackground = \"color\"\n{manifest_extra}"),
        )
        .unwrap();
        let (cat, problems) = Catalog::load(root.path()).unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        (root, cat)
    }

    const MODE: &str = "[[option]]\nkey = \"themeMode\"\nlabel = \"M\"\ntype = \"enum\"\nchoices = [{ value = \"dark\", label = \"D\" }]\n";

    #[test]
    fn a_clean_theme_has_no_problems() {
        let (_root, cat) = theme_with(MODE, "[General]\nthemeMode=dark\n");
        assert_eq!(lint(&cat.themes()[0]), Vec::<String>::new());
    }

    #[test]
    fn option_without_a_theme_conf_default_is_caught() {
        let (_root, cat) = theme_with(MODE, "[General]\n");
        assert_eq!(
            lint(&cat.themes()[0]),
            ["option \"themeMode\" has no default in theme.conf"]
        );
    }

    #[test]
    fn default_that_breaks_the_option_type_is_caught() {
        let (_root, cat) = theme_with(MODE, "[General]\nthemeMode=light\n");
        assert_eq!(lint(&cat.themes()[0]).len(), 1);
    }

    #[test]
    fn enabled_when_must_name_a_real_option_and_value() {
        let dep = "[[option]]\nkey = \"i\"\nlabel = \"I\"\ntype = \"int\"\nmin = 1\nmax = 2\nenabled_when = { themeMode = \"light\", nope = \"x\" }\n";
        let (_root, cat) = theme_with(&format!("{MODE}{dep}"), "[General]\nthemeMode=dark\ni=1\n");
        assert_eq!(lint(&cat.themes()[0]).len(), 2);
    }

    #[test]
    fn variants_need_a_default_in_theme_conf_and_qml_that_reads_it() {
        let v = "[supports]\nvariants = [\"light\", \"dark\"]\ndefault_variant = \"light\"\n";
        let (_root, cat) = theme_with(v, "[General]\n");
        let problems = lint(&cat.themes()[0]);
        assert!(
            problems.iter().any(|p| p.contains("colorScheme=light")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("no QML reads config.colorScheme")),
            "{problems:?}"
        );
        let (_root, cat) = theme_with(
            "[supports]\nvariants = [\"dark\"]\n",
            "[General]\ncolorScheme=dark\n",
        );
        assert!(
            lint(&cat.themes()[0])
                .iter()
                .any(|p| p.contains("default_variant"))
        );
    }

    #[test]
    fn colour_roles_need_support_a_hex_default_and_a_real_material_role() {
        let roles = "[supports]\ncolors = true\n\n[[color]]\nkey = \"lamp\"\nlabel = \"Lamp\"\nmaterial = \"tertiary\"\n\n[[color]]\nkey = \"rain\"\nlabel = \"Rain\"\nmaterial = \"sparkle\"\n";
        let (_root, cat) = theme_with(
            roles,
            "[General]\ncolorAccent=#e6bb5c\ncolorText=#ffffff\nlamp=#e6bb5c\nrain=blue\n",
        );
        let problems = lint(&cat.themes()[0]);
        assert!(
            problems.iter().any(|p| p.contains("\"sparkle\"")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("\"rain\" needs a hex default")),
            "{problems:?}"
        );
        assert_eq!(
            problems.iter().filter(|p| p.starts_with("colour")).count(),
            2,
            "{problems:?}"
        );
        assert!(
            problems.iter().any(|p| p.contains("darwan/Custom.qml")),
            "colours need the kit: {problems:?}"
        );
    }

    #[test]
    fn a_screensaver_theme_must_hide_on_ambient_and_keep_timers_slow() {
        let (root, cat) = theme_with("[supports]\nscreensaver = true\n", "[General]\n");
        let dir = root.path().join("t");
        assert_eq!(
            lint(&cat.themes()[0]),
            ["supports.screensaver but the theme uses neither Ambient nor darwan.ambient"]
        );
        std::fs::write(
            dir.join("Main.qml"),
            "Item { opacity: darwan.ambient ? 0 : 1\n Timer { interval: 16; running: true; repeat: true }\n Timer { interval: 16; running: root.windup }\n Timer { interval: 1000; running: true }\n Timer { interval: root.x; running: true } }",
        )
        .unwrap();
        let (cat, _) = Catalog::load(root.path()).unwrap();
        assert_eq!(
            lint(&cat.themes()[0]),
            [
                "supports.screensaver but a Timer always runs every 16 ms; use an animation or a slower timer"
            ]
        );
    }

    #[test]
    fn a_supports_flag_needs_theme_conf_defaults_and_qml_that_reads_it() {
        let (_root, cat) = theme_with(
            "[supports]\nclock_format = true\ndate_format = true\n",
            "[General]\nclockFormat=13h\n",
        );
        let problems = lint(&cat.themes()[0]);
        assert_eq!(
            problems,
            [
                "theme.conf clockFormat=\"13h\" must be 12h or 24h",
                "supports.clock_format needs clockShowAmPm in theme.conf",
                "supports.clock_format but no QML reads config.clockFormat",
                "supports.date_format but no QML reads config.dateFormat",
            ]
        );
    }
}
