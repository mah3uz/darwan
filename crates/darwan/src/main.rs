mod check;
mod completion;
mod doctor;
mod font;
mod lock;
mod media_cmd;
mod overlay;
mod preview;
mod qs;
mod saver;
mod sddm;
mod session;
mod settings_cmd;
mod style;
mod supervise;
mod tui;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter};

#[derive(Parser)]
#[command(
    name = "darwan",
    version,
    about = "Themes for the SDDM login screen and the Quickshell lockscreen",
    styles = style::clap_styles()
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
    Show {
        #[arg(add = ArgValueCandidates::new(completion::themes))]
        id: String,
    },
    /// Print a setting, or the whole config without KEY
    Get {
        #[arg(add = ArgValueCandidates::new(completion::setting_keys))]
        key: Option<String>,
    },
    /// Change a setting: lock.theme, sddm.theme, clock.format, clock.show_ampm, date.format, saver.lock_after, saver.quality, or <theme-id>.<option> for a theme's options and customisations (background, accent, variant, motion_speed, …)
    Set {
        #[arg(add = ArgValueCandidates::new(completion::setting_keys))]
        key: String,
        #[arg(add = ArgValueCompleter::new(completion::setting_values))]
        value: String,
    },
    /// Put a setting back to its default, or every setting of a theme when KEY is its id
    Unset {
        #[arg(add = ArgValueCandidates::new(completion::unset_keys))]
        key: String,
    },
    /// Lock the session with a theme (default: [lock] theme from the config)
    Lock {
        #[arg(add = ArgValueCandidates::new(completion::themes))]
        id: Option<String>,
        /// Kill a hung darwan locker and take over, e.g. from a TTY after a crash
        #[arg(long)]
        replace: bool,
        /// Testing only (needs DARWAN_DATA_DIR): unlock by itself after SECONDS
        #[arg(long, value_name = "SECONDS")]
        unlock_after: Option<u32>,
        /// Lock with a black screen and load the theme after wake (for before_sleep_cmd)
        #[arg(long)]
        for_sleep: bool,
    },
    /// Start the screensaver: the lock theme's background, then the lock (for hypridle's on-timeout)
    Saver,
    /// Record a wake from sleep, so the saver doesn't return before any input (for hypridle's after_sleep_cmd)
    Resumed,
    /// Internal: make the eco copies of the lock theme's videos; started when the lock theme or background changes
    #[command(name = "prepare-media", hide = true)]
    PrepareMedia,
    /// Internal: keeps the lock alive; started by `darwan lock`
    #[command(name = "lock-supervisor", hide = true)]
    LockSupervisor,
    /// Show a theme full screen without locking
    Preview {
        #[arg(add = ArgValueCandidates::new(completion::themes))]
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
        /// Save a 1280x720 PNG of the theme once it has settled, then close
        #[arg(long, value_name = "FILE")]
        shot: Option<PathBuf>,
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
        #[arg(add = ArgValueCandidates::new(completion::themes))]
        ids: Vec<String>,
        #[arg(long)]
        all: bool,
        /// Hide each theme's font/ folder, as on a fresh clone without licensed fonts
        #[arg(long)]
        no_fonts: bool,
        /// Virtual screen size
        #[arg(long, default_value = "1920x1080", value_parser = check::parse_size,
              add = ArgValueCandidates::new(completion::sizes))]
        size: (u32, u32),
        /// Save a screenshot of each theme into DIR
        #[arg(long, value_name = "DIR")]
        shots: Option<PathBuf>,
        #[arg(long, default_value_t = 4)]
        jobs: usize,
        /// Seconds before a theme counts as hung
        #[arg(long, default_value_t = 60)]
        timeout: u64,
        /// Apply your settings from config.toml, as the lock does, instead of each theme's defaults
        #[arg(long)]
        with_config: bool,
    },
    /// Print the script that sets up tab completion, e.g. `source <(darwan completion zsh)`
    Completion { shell: completion::Shell },
}

#[derive(Subcommand)]
enum SddmCmd {
    /// Use a theme for the login screen (default: [sddm] theme from the config)
    Apply {
        #[arg(add = ArgValueCandidates::new(completion::themes))]
        id: Option<String>,
    },
    /// Show a theme in SDDM's own test mode, with your settings
    Preview {
        #[arg(add = ArgValueCandidates::new(completion::themes))]
        id: Option<String>,
    },
    /// Show which theme SDDM uses and with which settings
    Status,
    /// Remove everything darwan set up for SDDM
    Reset,
}

#[derive(Subcommand)]
enum FontCmd {
    /// Copy FILE into the theme as the font it needs
    Import {
        #[arg(add = ArgValueCandidates::new(completion::themes_needing_fonts))]
        id: String,
        file: PathBuf,
        /// Which of the theme's fonts FILE is, when it needs more than one
        #[arg(long = "as", value_name = "NAME",
              add = ArgValueCompleter::new(completion::font_names))]
        target: Option<String>,
    },
}

fn main() -> ExitCode {
    // Let `darwan list | head` end quietly instead of panicking on a closed pipe.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    clap_complete::CompleteEnv::with_factory(<Cli as clap::CommandFactory>::command)
        .var(completion::VAR)
        .complete();
    let paths = darwan_core::paths::Paths::detect();
    let command = match Cli::parse().command {
        Some(c) => c,
        None if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            return tui::run(&paths).unwrap_or_else(|e| {
                eprintln!("{} {e}", style::fail_err("darwan:"));
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
            for_sleep,
        } => lock::run(&paths, id.as_deref(), replace, unlock_after, for_sleep),
        Cmd::Saver => saver::run(&paths),
        Cmd::Resumed => saver::resumed(),
        Cmd::PrepareMedia => media_cmd::run(&paths),
        Cmd::LockSupervisor => return supervise::run(),
        Cmd::Preview {
            id,
            sddm,
            pam,
            at,
            shot,
        } => preview::run(
            &paths,
            preview::Options {
                id,
                sddm,
                pam,
                at,
                shot,
            },
        ),
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
        Cmd::Completion { shell } => completion::print_registration(shell),
        Cmd::Check {
            ids,
            all,
            no_fonts,
            size,
            shots,
            jobs,
            timeout,
            with_config,
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
                with_config,
            },
        ),
    };
    result.unwrap_or_else(|e| {
        eprintln!("{} {e}", style::fail_err("darwan:"));
        ExitCode::FAILURE
    })
}
