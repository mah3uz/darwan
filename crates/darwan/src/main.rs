mod check;
mod doctor;
mod font;
mod lock;
mod overlay;
mod preview;
mod qs;
mod sddm;
mod session;
mod settings_cmd;
mod tui;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "darwan",
    version,
    about = "Themes for the SDDM login screen and the Quickshell lockscreen"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the themes (L marks the lock theme, S the SDDM theme)
    List,
    /// Show a theme's details, fonts and settings
    Show { id: String },
    /// Print a setting, or the whole config without KEY
    Get { key: Option<String> },
    /// Change a setting: lock.theme, sddm.theme, clock.format, clock.show_ampm, date.format or <theme-id>.<option>
    Set { key: String, value: String },
    /// Put a setting back to its default
    Unset { key: String },
    /// Lock the session with a theme (default: [lock] theme from the config)
    Lock {
        id: Option<String>,
        /// Kill a hung darwan locker and take over, e.g. from a TTY after a crash
        #[arg(long)]
        replace: bool,
        /// Testing only (needs DARWAN_DATA_DIR): unlock by itself after SECONDS
        #[arg(long, value_name = "SECONDS")]
        unlock_after: Option<u32>,
    },
    /// Show a theme in a window without locking
    Preview {
        id: Option<String>,
        /// Show the SDDM layout (session picker, power buttons) instead of the lock layout
        #[arg(long)]
        sddm: bool,
        /// Check the password with PAM instead of the mock ("test")
        #[arg(long)]
        pam: bool,
        /// Pretend the time is HH:MM (needs libfaketime; mock login only)
        #[arg(long, value_name = "HH:MM")]
        at: Option<String>,
    },
    /// Set up the SDDM login screen
    Sddm {
        #[command(subcommand)]
        command: SddmCmd,
    },
    /// Install a licensed font a theme needs (for the installed themes)
    Font {
        #[command(subcommand)]
        command: FontCmd,
    },
    /// Check the system for problems that stop themes from working
    Doctor,
    /// Load themes offscreen; fail on any QML warning or error, or if typing the password does not unlock
    Check {
        ids: Vec<String>,
        #[arg(long)]
        all: bool,
        /// Hide each theme's font/ folder, as on a fresh clone without licensed fonts
        #[arg(long)]
        no_fonts: bool,
        /// Virtual screen size
        #[arg(long, default_value = "1920x1080", value_parser = check::parse_size)]
        size: (u32, u32),
        /// Save a screenshot of each theme into DIR
        #[arg(long, value_name = "DIR")]
        shots: Option<PathBuf>,
        #[arg(long, default_value_t = 4)]
        jobs: usize,
        /// Seconds before a theme counts as hung
        #[arg(long, default_value_t = 60)]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum SddmCmd {
    /// Use a theme for the login screen (default: [sddm] theme from the config)
    Apply { id: Option<String> },
    /// Show a theme in SDDM's own test mode, with your settings
    Preview { id: Option<String> },
    /// Show which theme SDDM uses and with which settings
    Status,
    /// Remove everything darwan set up for SDDM
    Reset,
}

#[derive(Subcommand)]
enum FontCmd {
    /// Copy FILE into the theme as the font it needs
    Import {
        id: String,
        file: PathBuf,
        /// Which of the theme's fonts FILE is, when it needs more than one
        #[arg(long = "as", value_name = "NAME")]
        target: Option<String>,
    },
}

fn main() -> ExitCode {
    // Let `darwan list | head` end quietly instead of panicking on a closed pipe.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let paths = darwan_core::paths::Paths::detect();
    let command = match Cli::parse().command {
        Some(c) => c,
        None if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            return tui::run(&paths).unwrap_or_else(|e| {
                eprintln!("darwan: {e}");
                ExitCode::FAILURE
            });
        }
        None => {
            <Cli as clap::CommandFactory>::command().print_help().ok();
            return ExitCode::FAILURE;
        }
    };
    let result = match command {
        Cmd::List => settings_cmd::list(&paths),
        Cmd::Show { id } => settings_cmd::show(&paths, &id),
        Cmd::Get { key } => settings_cmd::get(&paths, key.as_deref()),
        Cmd::Set { key, value } => settings_cmd::set(&paths, &key, &value),
        Cmd::Unset { key } => settings_cmd::unset(&paths, &key),
        Cmd::Lock {
            id,
            replace,
            unlock_after,
        } => lock::run(&paths, id.as_deref(), replace, unlock_after),
        Cmd::Preview { id, sddm, pam, at } => {
            preview::run(&paths, preview::Options { id, sddm, pam, at })
        }
        Cmd::Sddm { command } => match command {
            SddmCmd::Apply { id } => sddm::apply(&paths, id.as_deref()),
            SddmCmd::Preview { id } => sddm::preview(&paths, id.as_deref()),
            SddmCmd::Status => sddm::status(),
            SddmCmd::Reset => sddm::reset(),
        },
        Cmd::Font {
            command: FontCmd::Import { id, file, target },
        } => font::import(&paths, &id, &file, target.as_deref()),
        Cmd::Doctor => doctor::run(&paths),
        Cmd::Check {
            ids,
            all,
            no_fonts,
            size,
            shots,
            jobs,
            timeout,
        } => check::run(
            &paths,
            check::Options {
                ids,
                all,
                no_fonts,
                size,
                shots,
                jobs,
                timeout: Duration::from_secs(timeout),
            },
        ),
    };
    result.unwrap_or_else(|e| {
        eprintln!("darwan: {e}");
        ExitCode::FAILURE
    })
}
