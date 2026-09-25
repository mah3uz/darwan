mod check;
mod lock;
mod overlay;
mod paths;
mod preview;
mod qs;
mod session;

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
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
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
        /// Pretend the time is HH:MM (needs libfaketime)
        #[arg(long, value_name = "HH:MM")]
        at: Option<String>,
    },
    /// Load themes offscreen and fail on any QML warning or error
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

fn main() -> ExitCode {
    let paths = paths::Paths::detect();
    let result = match Cli::parse().command {
        Cmd::Lock {
            id,
            replace,
            unlock_after,
        } => lock::run(&paths, id.as_deref(), replace, unlock_after),
        Cmd::Preview { id, sddm, pam, at } => {
            preview::run(&paths, preview::Options { id, sddm, pam, at })
        }
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
