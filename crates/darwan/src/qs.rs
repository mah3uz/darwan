use std::path::Path;
use std::process::Command;

use darwan_core::catalog::Theme;
use darwan_core::paths::Paths;

pub fn command(paths: &Paths, shell: &str, wrapper: &[String]) -> Command {
    let mut cmd = match wrapper.split_first() {
        Some((program, args)) => {
            let mut c = Command::new(program);
            c.args(args).arg("quickshell");
            c
        }
        None => Command::new("quickshell"),
    };
    cmd.arg("--no-color")
        .arg("-p")
        .arg(paths.runtime().join(shell));
    runtime_env(&mut cmd, paths);
    cmd
}

pub fn runtime_env(cmd: &mut Command, paths: &Paths) {
    cmd.current_dir(paths.runtime())
        .env("QML_XHR_ALLOW_FILE_READ", "1")
        .env("QML2_IMPORT_PATH", paths.qml_import_path());
}

pub fn theme_env(cmd: &mut Command, theme: &Theme, theme_dir: &Path, overlay: Option<&Path>) {
    cmd.env("DARWAN_THEME_ID", &theme.id)
        .env("DARWAN_THEME_PATH", theme_dir)
        .env(
            "DARWAN_OVERLAY",
            overlay.map(Path::as_os_str).unwrap_or_default(),
        );
}

// Path-symlink races between parallel Quickshell instances are not about the theme.
pub fn problem_lines(log: &str) -> Vec<String> {
    let mut out: Vec<String> = log
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("WARN") || l.starts_with("ERROR"))
        .filter(|l| !l.contains("quickshell.paths"))
        .map(str::to_string)
        .collect();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_warnings_and_errors_count_and_path_races_are_ignored() {
        let log = "  INFO: Launching config\n  WARN scene: a.qml[1:1]: TypeError\n  WARN scene: a.qml[1:1]: TypeError\n ERROR quickshell.paths: Could not create path symlink\n ERROR qml: darwan: failed to load x\nInput #0, mov\n";
        assert_eq!(
            problem_lines(log),
            [
                "WARN scene: a.qml[1:1]: TypeError",
                "ERROR qml: darwan: failed to load x"
            ]
        );
    }
}
