use std::ffi::{OsStr, OsString};
use std::process::ExitCode;

use clap::ValueEnum;
use clap_complete::engine::CompletionCandidate;
use clap_complete::env::Shells;

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::{Target, UserConfig};
use darwan_core::manifest::OptionKind;
use darwan_core::paths::{self, Paths};
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

fn keys_in(catalog: &Catalog) -> Vec<CompletionCandidate> {
    let mut out = vec![
        candidate("lock.theme", "Theme for the lockscreen"),
        candidate("sddm.theme", "Theme for the SDDM login screen"),
        candidate("clock.format", "12h or 24h, for themes that support it"),
        candidate("clock.show_ampm", "Show AM/PM with the 12-hour clock"),
        candidate("date.format", "Date format, for themes that support it"),
    ];
    for t in catalog.themes() {
        for opt in &t.manifest.options {
            out.push(candidate(
                format!("{}.{}", t.id, opt.key),
                format!("{}: {}", t.manifest.name, opt.label),
            ));
        }
    }
    out
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
        Key::Option { theme, key } => {
            let Some(opt) = catalog.get(&theme).and_then(|t| t.manifest.option(&key)) else {
                return Vec::new();
            };
            match opt.kind {
                OptionKind::Enum => opt
                    .choices
                    .iter()
                    .map(|c| candidate(&c.value, &c.label))
                    .collect(),
                OptionKind::Bool => vec![candidate("true", "on"), candidate("false", "off")],
                _ => Vec::new(),
            }
        }
    }
}

fn starting_with(values: Vec<CompletionCandidate>, current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();
    values
        .into_iter()
        .filter(|c| c.get_value().to_string_lossy().starts_with(prefix.as_ref()))
        .collect()
}

pub fn setting_values(current: &OsStr) -> Vec<CompletionCandidate> {
    let values =
        word_after(&["set"]).map_or_else(Vec::new, |key| values_for(&catalog(), &config(), &key));
    starting_with(values, current)
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
