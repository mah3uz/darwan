use std::process::ExitCode;

use darwan_core::catalog::Catalog;
use darwan_core::config::{Target, UserConfig};
use darwan_core::form::{self, FieldKind};
use darwan_core::paths::{self, Paths};
use darwan_core::settings::{self, Key};

use crate::style;

fn load(paths: &Paths) -> Result<(Catalog, UserConfig), String> {
    let (catalog, _) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let path = paths::config_file();
    let config = UserConfig::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((catalog, config))
}

// Grouped like the GUI and TUI; the id stays the first word of each theme's line, for scripts.
pub fn list(paths: &Paths) -> Result<ExitCode, String> {
    let (catalog, config) = load(paths)?;
    let lock = config.theme(Target::Lock).ok().flatten().unwrap_or("");
    let sddm = config.theme(Target::Sddm).ok().flatten().unwrap_or("");
    let themes = catalog.themes();
    let id_width = themes
        .iter()
        .map(|t| t.id.chars().count())
        .max()
        .unwrap_or(0);
    let name_width = themes
        .iter()
        .map(|t| t.manifest.name.chars().count())
        .max()
        .unwrap_or(0);
    let group_of = |t: &darwan_core::catalog::Theme| match t.manifest.family.as_deref() {
        Some(f @ ("Clockwork" | "Pixel")) => f.to_string(),
        _ => "Other".to_string(),
    };
    for family in ["Clockwork", "Pixel", "Other"] {
        let group: Vec<_> = themes.iter().filter(|t| group_of(t) == family).collect();
        if group.is_empty() {
            continue;
        }
        println!(
            "{} {}",
            style::heading(family),
            style::dim(format!("· {}", group.len()))
        );
        for t in group {
            let marks = format!(
                "{}{}",
                if t.id == lock {
                    style::lock("L")
                } else {
                    " ".into()
                },
                if t.id == sddm {
                    style::sddm("S")
                } else {
                    " ".into()
                }
            );
            let mut tags = Vec::new();
            if !t.manifest.supports.variants.is_empty() {
                tags.push(style::accent("light · dark"));
            }
            match t.missing_fonts() {
                0 => {}
                n => tags.push(style::warn(format!(
                    "{n} font{} missing",
                    if n == 1 { "" } else { "s" }
                ))),
            }
            let line = format!(
                "  {marks} {}  {}  {}",
                style::id(style::pad(&t.id, id_width)),
                style::pad(&t.manifest.name, name_width),
                tags.join("  ")
            );
            println!("{}", line.trim_end());
        }
        println!();
    }
    println!(
        "{} lock theme  {} SDDM theme  {}",
        style::lock("L"),
        style::sddm("S"),
        style::dim("· darwan show <id> for a theme's settings")
    );
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
            .map(|f| style::dim(format!("{f} · ")))
            .unwrap_or_default(),
        style::heading(&m.name)
    );
    let row =
        |label: &str, value: String| println!("  {} {value}", style::dim(style::pad(label, 11)));
    row("id", style::id(&t.id));
    row("author", m.author.clone());
    row("background", format!("{:?}", m.background));
    if !m.supports.variants.is_empty() {
        row("looks", style::accent(m.supports.variants.join(" · ")));
    }
    if let Some(p) = &t.preview {
        row("preview", style::dim(p.display().to_string()));
    }
    for f in &m.fonts {
        let state = if t.dir.join("font").join(&f.file).is_file() {
            style::ok("installed")
        } else {
            style::warn("missing")
        };
        let url = if f.url.is_empty() {
            "no public source".into()
        } else {
            f.url.clone()
        };
        row(
            "font",
            format!(
                "{} ({}) {state}{}",
                f.family,
                f.file,
                style::dim(format!("; {}; {url}", f.license))
            ),
        );
    }
    println!(
        "\n{} {}",
        style::heading("Settings"),
        style::dim("· darwan set <key> <value>")
    );
    let fields = form::fields(t, &config);
    let key_width = fields
        .iter()
        .map(|f| f.key.to_string().chars().count())
        .max()
        .unwrap_or(0);
    let mut group = None;
    for field in fields {
        let heading = match field.key {
            Key::Option { .. } => field.group,
            _ => Some("Global"),
        };
        if heading != group {
            group = heading;
            if let Some(g) = group {
                println!("  {}", style::bold(g));
            }
        }
        let (shown, origin) = match (&field.kind, field.value.as_str()) {
            (_, "") => (style::dim("(theme default)"), String::new()),
            _ => (
                style::bold(&field.value),
                if field.is_set {
                    style::value(" (set)")
                } else {
                    style::dim(" (default)")
                },
            ),
        };
        let choices = match &field.kind {
            FieldKind::Choice(c) => style::dim(format!(
                "  [{}]",
                c.iter()
                    .map(|(v, _)| v.as_str())
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>()
                    .join(" | ")
            )),
            FieldKind::Bool => style::dim("  [true | false]"),
            _ => String::new(),
        };
        let disabled = field
            .disabled
            .map(|r| style::note(format!("  {r}")))
            .unwrap_or_default();
        println!(
            "  {} {shown}{origin}{choices}{disabled}",
            style::id(style::pad(&field.key.to_string(), key_width))
        );
    }
    Ok(ExitCode::SUCCESS)
}

pub fn get(paths: &Paths, key: Option<&str>) -> Result<ExitCode, String> {
    let (_, config) = load(paths)?;
    match key {
        None => {
            println!(
                "{}",
                style::dim(format!("# {}", paths::config_file().display()))
            );
            print!("{}", style::toml(&config.to_string()));
        }
        Some(k) => match settings::get(&config, &Key::parse(k)?)? {
            Some(v) => println!("{}", style::value(v)),
            None => println!("{}", style::dim("(not set)")),
        },
    }
    Ok(ExitCode::SUCCESS)
}

pub fn set(paths: &Paths, key: &str, value: &str) -> Result<ExitCode, String> {
    let (catalog, mut config) = load(paths)?;
    let key = Key::parse(key)?;
    settings::set(&mut config, &catalog, &key, value)?;
    save(&config)?;
    println!("{} = {}", style::id(key.to_string()), style::value(value));
    Ok(ExitCode::SUCCESS)
}

pub fn unset(paths: &Paths, key: &str) -> Result<ExitCode, String> {
    let (catalog, mut config) = load(paths)?;
    // Theme ids never contain '.', so a bare id can't be mistaken for a setting.
    if catalog.get(key).is_some() {
        if config.remove_theme(key) {
            save(&config)?;
            println!("{} every setting is back to its default", style::id(key));
        } else {
            println!("{} has no settings", style::id(key));
        }
        return Ok(ExitCode::SUCCESS);
    }
    let key = Key::parse(key)?;
    if settings::unset(&mut config, &key) {
        save(&config)?;
        println!("{} is back to its default", style::id(key.to_string()));
    } else {
        println!("{} was not set", style::id(key.to_string()));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn save(config: &UserConfig) -> Result<(), String> {
    let path = paths::config_file();
    config
        .save(&path)
        .map_err(|e| format!("{}: {e}", path.display()))
}
