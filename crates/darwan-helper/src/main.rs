use std::io::Read;
use std::process::ExitCode;

use darwan_helper::{MAX_FONT_BYTES, Roots, apply, import_font, reset};

const USAGE: &str =
    "usage: darwan-helper sddm-apply <theme-id> < request (JSON line, then attachments)
       darwan-helper sddm-reset
       darwan-helper font-import <theme-id> <file-name> < font";

fn read_stdin(limit: usize) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    std::io::stdin()
        .take(limit as u64 + 1)
        .read_to_end(&mut buf)
        .map_err(|e| format!("reading stdin: {e}"))?;
    if buf.len() > limit {
        return Err("input on stdin is too large".into());
    }
    Ok(buf)
}

fn run(args: &[String]) -> Result<String, String> {
    if !rustix::process::geteuid().is_root() {
        return Err("must run as root, through pkexec".into());
    }
    let roots = Roots::system();
    match args {
        [cmd, id] if cmd == "sddm-apply" => apply(&roots, id, &mut std::io::stdin().lock()),
        [cmd] if cmd == "sddm-reset" => reset(&roots),
        [cmd, id, file] if cmd == "font-import" => {
            import_font(&roots, id, file, &read_stdin(MAX_FONT_BYTES)?)
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("darwan-helper: {e}");
            ExitCode::FAILURE
        }
    }
}
