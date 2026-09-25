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

    for font in &m.fonts {
        if font.file.is_empty() || font.file.contains('/') {
            problems.push(format!(
                "font file {:?} must be a plain file name",
                font.file
            ));
        }
    }

    if theme.preview.is_none() {
        problems.push("no preview.gif or preview.png".into());
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
}
