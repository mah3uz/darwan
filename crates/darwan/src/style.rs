use std::io::IsTerminal;
use std::sync::OnceLock;

// Colour only on a terminal, and never with NO_COLOR set (no-color.org): the GUI and TUI parse this output from a pipe.
fn wanted(is_terminal: bool) -> bool {
    is_terminal && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
}

fn stdout_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| wanted(std::io::stdout().is_terminal()))
}

fn stderr_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| wanted(std::io::stderr().is_terminal()))
}

fn paint(on: bool, sgr: &str, text: &str) -> String {
    if on {
        format!("\x1b[{sgr}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

macro_rules! styles {
    ($($name:ident, $err:ident = $sgr:literal;)*) => {
        $(
            pub fn $name(text: impl AsRef<str>) -> String {
                paint(stdout_on(), $sgr, text.as_ref())
            }
            #[allow(dead_code)]
            pub fn $err(text: impl AsRef<str>) -> String {
                paint(stderr_on(), $sgr, text.as_ref())
            }
        )*
    };
}

styles! {
    ok, ok_err = "1;32";
    warn, warn_err = "1;33";
    note, note_err = "33";
    fail, fail_err = "1;31";
    heading, heading_err = "1;35";
    bold, bold_err = "1";
    dim, dim_err = "2";
    id, id_err = "36";
    value, value_err = "32";
    lock, lock_err = "1;35";
    sddm, sddm_err = "1;34";
    accent, accent_err = "35";
}

// Width-aware padding: escape codes take no columns, so pad the plain text first.
pub fn pad(text: &str, width: usize) -> String {
    format!("{text:<width$}")
}

// `darwan get` with no key: TOML with sections, keys and comments told apart.
pub fn toml(text: &str) -> String {
    text.lines()
        .map(|line| {
            let t = line.trim_start();
            if t.starts_with('#') {
                dim(line)
            } else if t.starts_with('[') {
                heading(line)
            } else if let Some((k, v)) = line.split_once('=') {
                format!("{}={}", id(k), value(v))
            } else {
                line.to_string()
            }
        })
        .map(|l| l + "\n")
        .collect()
}

pub fn clap_styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor, Effects};
    clap::builder::Styles::styled()
        .header(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .usage(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_colour_off_a_terminal_or_with_no_color() {
        assert!(
            !wanted(false),
            "a pipe gets plain text, which the GUI parses"
        );
        assert_eq!(paint(false, "1;32", "ok"), "ok");
        assert_eq!(paint(true, "1;32", "ok"), "\x1b[1;32mok\x1b[0m");
    }
}
