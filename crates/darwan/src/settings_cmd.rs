use std::process::ExitCode;

use darwan_core::catalog::Catalog;
use darwan_core::config::{Target, UserConfig};
use darwan_core::form::{self, FieldKind};
use darwan_core::paths::{self, Paths};
use darwan_core::settings::{self, Key};

fn load(paths: &Paths) -> Result<(Catalog, UserConfig), String> {
    let (catalog, _) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let path = paths::config_file();
    let config = UserConfig::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((catalog, config))
}

pub fn list(paths: &Paths) -> Result<ExitCode, String> {
    let (catalog, config) = load(paths)?;
    let lock = config.theme(Target::Lock).ok().flatten().unwrap_or("");
    let sddm = config.theme(Target::Sddm).ok().flatten().unwrap_or("");
    for t in catalog.themes() {
        let marks = format!(
            "{}{}",
            if t.id == lock { "L" } else { " " },
            if t.id == sddm { "S" } else { " " }
        );
        let name = match &t.manifest.family {
            Some(f) => format!("{f} · {}", t.manifest.name),
            None => t.manifest.name.clone(),
        };
        let fonts = match t.missing_fonts() {
            0 => String::new(),
            n => format!("  ({n} font{} missing)", if n == 1 { "" } else { "s" }),
        };
        println!("{marks} {:<24} {name}{fonts}", t.id);
    }
    println!("\nL = lock theme, S = SDDM theme");
    Ok(ExitCode::SUCCESS)
}

pub fn show(paths: &Paths, id: &str) -> Result<ExitCode, String> {
    let (catalog, config) = load(paths)?;
    let t = catalog
        .get(id)
        .ok_or_else(|| darwan_core::catalog::unknown_theme(id))?;
    let m = &t.manifest;
    println!(
        "{}{}",
        m.family
            .as_ref()
            .map(|f| format!("{f} · "))
            .unwrap_or_default(),
        m.name
    );
    println!("  id          {}", t.id);
    println!("  author      {}", m.author);
    println!("  background  {:?}", m.background);
    if let Some(p) = &t.preview {
        println!("  preview     {}", p.display());
    }
    for f in &m.fonts {
        let state = if t.dir.join("font").join(&f.file).is_file() {
            "installed"
        } else {
            "MISSING"
        };
        let url = if f.url.is_empty() {
            "no public source".into()
        } else {
            f.url.clone()
        };
        println!(
            "  font        {} ({}) {state}; {}; {url}",
            f.family, f.file, f.license
        );
    }
    println!("\nSettings (set with `darwan set <key> <value>`):");
    for field in form::fields(t, &config) {
        let (shown, origin) = match (&field.kind, field.value.as_str()) {
            (FieldKind::Text, "") => ("(theme default)".to_string(), ""),
            _ => (
                field.value.clone(),
                if field.is_set { " (set)" } else { " (default)" },
            ),
        };
        let choices = match &field.kind {
            FieldKind::Choice(c) => format!(
                "  [{}]",
                c.iter()
                    .map(|(v, _)| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            FieldKind::Bool => "  [true | false]".into(),
            _ => String::new(),
        };
        let disabled = field
            .disabled
            .map(|r| format!("  disabled: {r}"))
            .unwrap_or_default();
        println!(
            "  {:<32} {shown}{origin}{choices}{disabled}",
            field.key.to_string()
        );
    }
    Ok(ExitCode::SUCCESS)
}

pub fn get(paths: &Paths, key: Option<&str>) -> Result<ExitCode, String> {
    let (_, config) = load(paths)?;
    match key {
        None => {
            println!("# {}", paths::config_file().display());
            print!("{config}");
        }
        Some(k) => match settings::get(&config, &Key::parse(k)?)? {
            Some(v) => println!("{v}"),
            None => println!("(not set)"),
        },
    }
    Ok(ExitCode::SUCCESS)
}

pub fn set(paths: &Paths, key: &str, value: &str) -> Result<ExitCode, String> {
    let (catalog, mut config) = load(paths)?;
    let key = Key::parse(key)?;
    settings::set(&mut config, &catalog, &key, value)?;
    save(&config)?;
    println!("{key} = {value}");
    Ok(ExitCode::SUCCESS)
}

pub fn unset(paths: &Paths, key: &str) -> Result<ExitCode, String> {
    let (_, mut config) = load(paths)?;
    let key = Key::parse(key)?;
    if settings::unset(&mut config, &key) {
        save(&config)?;
        println!("{key} is back to its default");
    } else {
        println!("{key} was not set");
    }
    Ok(ExitCode::SUCCESS)
}

pub fn save(config: &UserConfig) -> Result<(), String> {
    let path = paths::config_file();
    config
        .save(&path)
        .map_err(|e| format!("{}: {e}", path.display()))
}
