use std::ffi::{OsStr, OsString};
use std::process::ExitCode;

use clap::ValueEnum;
use clap_complete::engine::{CompletionCandidate, PathCompleter, ValueCompleter};
use clap_complete::env::Shells;

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::form::{self, Field, FieldKind};
use darwan_core::paths::{self, Paths};
use darwan_core::saver::Quality;
use darwan_core::settings::{DATE_PRESETS, Key};

pub const VAR: &str = "DARWAN_COMPLETE";

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

pub fn print_registration(shell: Shell) -> Result<ExitCode, String> {
    let name = match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let shells = Shells::builtins();
    let completer = shells
        .completer(name)
        .ok_or_else(|| format!("no completer for {name}"))?;
    let mut out = Vec::new();
    completer
        .write_registration(VAR, "darwan", "darwan", &exe.to_string_lossy(), &mut out)
        .map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut std::io::stdout(), &out).map_err(|e| e.to_string())?;
    Ok(ExitCode::SUCCESS)
}

fn catalog() -> Catalog {
    Catalog::load(&Paths::detect().themes())
        .map(|(c, _)| c)
        .unwrap_or_default()
}

fn candidate(value: impl Into<OsString>, help: impl Into<String>) -> CompletionCandidate {
    CompletionCandidate::new(value).help(Some(help.into().into()))
}

fn theme_help(t: &Theme, config: &UserConfig) -> String {
    let mut help = match &t.manifest.family {
        Some(f) => format!("{f} · {}", t.manifest.name),
        None => t.manifest.name.clone(),
    };
    for (target, mark) in [(Target::Lock, "lock"), (Target::Sddm, "SDDM")] {
        if config.theme(target).ok().flatten() == Some(t.id.as_str()) {
            help.push_str(&format!(" [{mark}]"));
        }
    }
    help
}

fn config() -> UserConfig {
    UserConfig::load(&paths::config_file()).unwrap_or_default()
}

fn theme_candidates(catalog: &Catalog, config: &UserConfig) -> Vec<CompletionCandidate> {
    catalog
        .themes()
        .iter()
        .map(|t| candidate(&t.id, theme_help(t, config)))
        .collect()
}

pub fn themes() -> Vec<CompletionCandidate> {
    theme_candidates(&catalog(), &config())
}

pub fn themes_needing_fonts() -> Vec<CompletionCandidate> {
    catalog()
        .themes()
        .iter()
        .filter(|t| !t.manifest.fonts.is_empty())
        .map(|t| {
            let fonts: Vec<String> = t
                .manifest
                .fonts
                .iter()
                .map(|f| format!("{} ({})", f.family, f.license))
                .collect();
            candidate(&t.id, fonts.join(", "))
        })
        .collect()
}

pub fn setting_keys() -> Vec<CompletionCandidate> {
    keys_in(&catalog())
}

// A theme id resets that theme, so ids are offered next to the settings.
pub fn unset_keys() -> Vec<CompletionCandidate> {
    let (catalog, config) = (catalog(), config());
    let mut out = keys_in(&catalog);
    out.extend(theme_candidates(&catalog, &config));
    out
}

fn keys_in(catalog: &Catalog) -> Vec<CompletionCandidate> {
    let mut out = vec![
        candidate("lock.theme", "Theme for the lockscreen"),
        candidate("sddm.theme", "Theme for the SDDM login screen"),
        candidate("clock.format", "12h or 24h, for themes that support it"),
        candidate("clock.show_ampm", "Show AM/PM with the 12-hour clock"),
        candidate("date.format", "Date format, for themes that support it"),
        candidate(
            "saver.lock_after",
            "Seconds from the screensaver to the lock, or never",
        ),
        candidate(
            "saver.return_after",
            "Seconds an untouched lock waits before the screensaver shows",
        ),
        candidate(
            "saver.quality",
            "Screensaver video quality: auto, full, eco or still",
        ),
        candidate(
            "wallpaper.folder",
            "Where the Wallpapers page reads and downloads (~/Pictures/Wallpapers)",
        ),
        candidate(
            "wallpaper.allow",
            "Online groups you allow: people, anime, games, series, war, gore, horror",
        ),
        candidate(
            "wallpaper.restart",
            "Wallpaper tools Darwan may restart to change the wallpaper",
        ),
        candidate(
            "wallpaper.colours",
            "Colour generators to run after a wallpaper change: matugen, pywal, wallust, hellwal",
        ),
        candidate(
            "gui.look",
            "The GUI's look: darwan, or system to follow your Qt theme",
        ),
    ];
    // The same fields the GUI and `darwan show` list: the theme's own options and the customisations it supports.
    for t in catalog.themes() {
        for field in theme_fields(t) {
            out.push(candidate(
                field.key.to_string(),
                format!("{}: {}", t.manifest.name, field.label),
            ));
        }
    }
    out
}

// Settings the theme doesn't support stay out, though the form shows them disabled to say why.
fn theme_fields(theme: &Theme) -> Vec<Field> {
    form::fields(theme, &UserConfig::default())
        .into_iter()
        .filter(|f| match &f.key {
            Key::Option { key, .. } => darwan_core::custom::unsupported(theme, key).is_none(),
            _ => false,
        })
        .collect()
}

fn installed_font_families() -> Vec<String> {
    let Ok(out) = std::process::Command::new("fc-list")
        .args([":", "family"])
        .output()
    else {
        return Vec::new();
    };
    let mut families: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split(',').next())
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect();
    families.sort();
    families.dedup();
    families
}

// Numbers to offer for a range: the default and a few round steps, all inside it.
fn range_values(key: &str, min: f64, max: f64) -> Vec<CompletionCandidate> {
    let steps: &[f64] = if key.ends_with(".motion_speed") {
        &[0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0]
    } else {
        &[-1.0, -0.5, 0.0, 0.5, 1.0]
    };
    steps
        .iter()
        .filter(|v| (min..=max).contains(*v))
        .map(|v| {
            candidate(
                v.to_string(),
                if *v == 1.0 && key.ends_with(".motion_speed") {
                    "the theme's own speed"
                } else {
                    ""
                },
            )
        })
        .collect()
}

pub fn sizes() -> Vec<CompletionCandidate> {
    [
        ("1920x1080", "1080p"),
        ("2560x1440", "1440p"),
        ("3840x2160", "4K"),
        ("3440x1440", "21:9 ultrawide"),
    ]
    .into_iter()
    .map(|(v, h)| candidate(v, h))
    .collect()
}

// Candidates only see the word being completed, so the key (or theme) typed earlier on the
// same line is read back from the command line the shell handed over.
fn word_after(subcommand: &[&str]) -> Option<String> {
    let args: Vec<String> = std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let start = args.iter().position(|a| a == "--").map_or(0, |i| i + 1);
    let words = &args[start..];
    let at = words
        .windows(subcommand.len())
        .position(|w| w == subcommand)?;
    words.get(at + subcommand.len()).cloned()
}

fn values_for(catalog: &Catalog, config: &UserConfig, key: &str) -> Vec<CompletionCandidate> {
    let Ok(key) = Key::parse(key) else {
        return Vec::new();
    };
    let wanted = key.to_string();
    match key {
        Key::Theme(_) => theme_candidates(catalog, config),
        Key::ClockFormat => vec![
            candidate("24h", "24-hour clock"),
            candidate("12h", "12-hour clock"),
        ],
        Key::ClockShowAmPm => vec![
            candidate("true", "show AM/PM"),
            candidate("false", "hide AM/PM"),
        ],
        Key::DateFormat => DATE_PRESETS
            .iter()
            .map(|p| candidate(p.format, p.label))
            .collect(),
        Key::SaverLockAfter => vec![
            candidate("0", "lock as the saver appears"),
            candidate("60", "a minute of grace"),
            candidate("never", "the saver never locks"),
        ],
        Key::SaverReturnAfter => vec![
            candidate("30", "half a minute, the default"),
            candidate("300", "five minutes"),
        ],
        Key::SaverQuality => Quality::ALL
            .iter()
            .map(|q| candidate(q.as_str(), q.describe()))
            .collect(),
        Key::WallpaperFolder => Vec::new(),
        Key::WallpaperColours => darwan_core::wallpaper::colours::Generator::ALL
            .iter()
            .map(|g| candidate(g.id(), "run after each wallpaper change"))
            .collect(),
        Key::WallpaperRestart => darwan_core::wallpaper::set::RESTARTED
            .iter()
            .map(|t| candidate(t.name(), "may be restarted to change the wallpaper"))
            .collect(),
        Key::WallpaperAllow => darwan_core::wallpaper::filter::GROUPS
            .iter()
            .map(|g| candidate(g.id, g.label))
            .collect(),
        Key::GuiLook => vec![
            candidate("darwan", "Darwan's own greys and blue, see-through panels"),
            candidate("system", "your Qt theme's colours and font"),
        ],
        Key::Option { theme, .. } => {
            let Some(t) = catalog.get(&theme) else {
                return Vec::new();
            };
            let Some(field) = theme_fields(t)
                .into_iter()
                .find(|f| f.key.to_string() == wanted)
            else {
                return Vec::new();
            };
            match field.kind {
                FieldKind::Choice(choices) => choices
                    .iter()
                    // An empty choice is the theme's default, which `darwan unset` restores.
                    .filter(|(v, _)| !v.is_empty())
                    .map(|(v, label)| candidate(v, label))
                    .collect(),
                FieldKind::Bool => vec![candidate("true", "on"), candidate("false", "off")],
                FieldKind::Int { min, max } if max - min <= 100 => (0..=4)
                    .map(|i| min + (max - min) * i / 4)
                    .map(|v| candidate(v.to_string(), ""))
                    .collect(),
                FieldKind::Range { min, max, .. } => range_values(&wanted, min, max),
                FieldKind::Color { generate } => {
                    let mut own: Vec<String> = t
                        .defaults
                        .values()
                        .filter(|v| darwan_core::custom::is_hex_color(v))
                        .map(|v| v.to_lowercase())
                        .collect();
                    own.sort();
                    own.dedup();
                    let mut out = Vec::new();
                    if generate {
                        out.push(candidate(
                            "generate",
                            "picked from the background, like Material You",
                        ));
                    }
                    out.extend(
                        own.iter()
                            .map(|c| candidate(c, "one of the theme's own colours")),
                    );
                    out
                }
                FieldKind::Media(_) => vec![candidate("desktop", "your desktop wallpaper")],
                FieldKind::Font => installed_font_families()
                    .into_iter()
                    .map(|f| candidate(f, "installed font"))
                    .collect(),
                _ => Vec::new(),
            }
        }
    }
}

// File extensions a key also takes a path for; the shell completes those as paths.
fn path_extensions(catalog: &Catalog, key: &str) -> Option<Vec<String>> {
    let Ok(Key::Option { theme, .. }) = Key::parse(key) else {
        return None;
    };
    let field = theme_fields(catalog.get(&theme)?)
        .into_iter()
        .find(|f| f.key.to_string() == key)?;
    match field.kind {
        FieldKind::Media(exts) | FieldKind::File(exts) => Some(exts),
        FieldKind::Font => Some(
            darwan_core::custom::FONT_EXT
                .iter()
                .map(|e| format!(".{e}"))
                .collect(),
        ),
        _ => None,
    }
}

fn paths_with(current: &OsStr, extensions: Vec<String>) -> Vec<CompletionCandidate> {
    let exts: Vec<String> = extensions
        .iter()
        .map(|e| e.trim_start_matches(['*', '.']).to_lowercase())
        .collect();
    PathCompleter::any()
        .filter(move |p| {
            p.is_dir()
                || p.extension()
                    .is_some_and(|e| exts.contains(&e.to_string_lossy().to_lowercase()))
        })
        .complete(current)
}

fn starting_with(values: Vec<CompletionCandidate>, current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    values
        .into_iter()
        .filter(|c| c.get_value().to_string_lossy().starts_with(prefix.as_ref()))
        .collect()
}

pub fn setting_values(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(key) = word_after(&["set"]) else {
        return Vec::new();
    };
    let catalog = catalog();
    let mut values = starting_with(values_for(&catalog, &config(), &key), current);
    // A path is only offered once one is being typed, so `desktop` and `generate` aren't buried under files.
    let typing_path = current.to_string_lossy().contains('/')
        || current.to_string_lossy().starts_with(['.', '~']);
    if let Some(exts) = path_extensions(&catalog, &key).filter(|_| typing_path) {
        values.extend(paths_with(current, exts));
    }
    values
}

pub fn font_names(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(id) = word_after(&["font", "import"]) else {
        return Vec::new();
    };
    let catalog = catalog();
    let Some(theme) = catalog.get(&id) else {
        return Vec::new();
    };
    let names = theme
        .manifest
        .fonts
        .iter()
        .map(|f| candidate(&f.file, &f.family))
        .collect();
    starting_with(names, current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo() -> Catalog {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        Catalog::load(&root).unwrap().0
    }

    #[test]
    fn every_offered_value_is_one_set_accepts() {
        let catalog = repo();
        let mut config = UserConfig::default();
        let mut offered = 0;
        for key in keys_in(&catalog) {
            let key = key.get_value().to_string_lossy().into_owned();
            let values = values_for(&catalog, &UserConfig::default(), &key);
            for value in values.iter().map(|c| c.get_value().to_string_lossy()) {
                let parsed = Key::parse(&key).unwrap();
                darwan_core::settings::set(&mut config, &catalog, &parsed, &value)
                    .unwrap_or_else(|e| panic!("offered `darwan set {key} {value}`: {e}"));
                offered += 1;
            }
        }
        assert!(offered > 100, "only {offered} values offered");
    }

    #[test]
    fn date_format_offers_the_presets_with_samples() {
        let d = values_for(&repo(), &UserConfig::default(), "date.format");
        assert_eq!(d.len(), DATE_PRESETS.len());
        assert!(d.iter().all(|c| c.get_help().is_some()));
    }
}
