use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct WaylandSession {
    pub runtime_dir: PathBuf,
    pub display: String,
    pub hyprland: Option<String>,
}

impl WaylandSession {
    // From a TTY none of these are set, so they are discovered from the runtime dir.
    pub fn discover() -> Result<Self, String> {
        let runtime_dir = match std::env::var_os("XDG_RUNTIME_DIR") {
            Some(d) if !d.is_empty() => PathBuf::from(d),
            _ => PathBuf::from(format!("/run/user/{}", rustix::process::getuid().as_raw())),
        };
        let display = match std::env::var("WAYLAND_DISPLAY") {
            Ok(d) if !d.is_empty() => d,
            _ => only_wayland_socket(&runtime_dir)?,
        };
        // A shell that outlived its session (tmux, uwsm restart) still carries the old signature.
        let hyprland = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(s) if hyprland_is_live(&runtime_dir, &s) => Some(s),
            _ => newest_hyprland_instance(&runtime_dir),
        };
        Ok(Self {
            runtime_dir,
            display,
            hyprland,
        })
    }

    pub fn apply(&self, cmd: &mut Command) {
        cmd.env("XDG_RUNTIME_DIR", &self.runtime_dir)
            .env("WAYLAND_DISPLAY", &self.display)
            .env("XDG_SESSION_TYPE", "wayland")
            .env("QT_QPA_PLATFORM", "wayland");
        if let Some(sig) = &self.hyprland {
            cmd.env("HYPRLAND_INSTANCE_SIGNATURE", sig);
        }
    }
}

fn only_wayland_socket(runtime_dir: &Path) -> Result<String, String> {
    let mut sockets: Vec<String> = std::fs::read_dir(runtime_dir)
        .map_err(|e| format!("reading {}: {e}", runtime_dir.display()))?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_socket()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("wayland-") && !n.ends_with(".lock"))
        .collect();
    sockets.sort();
    match sockets.len() {
        0 => Err(format!(
            "no Wayland session found in {}; darwan only supports Wayland",
            runtime_dir.display()
        )),
        1 => Ok(sockets.remove(0)),
        _ => Err(format!(
            "more than one Wayland session ({}); set WAYLAND_DISPLAY to pick one",
            sockets.join(", ")
        )),
    }
}

fn hyprland_is_live(runtime_dir: &Path, signature: &str) -> bool {
    !signature.is_empty()
        && !signature.contains('/')
        && runtime_dir
            .join("hypr")
            .join(signature)
            .join(".socket.sock")
            .exists()
}

fn newest_hyprland_instance(runtime_dir: &Path) -> Option<String> {
    std::fs::read_dir(runtime_dir.join("hypr"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| e.path().join(".socket.sock").exists())
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .and_then(|e| e.file_name().into_string().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    #[test]
    fn one_wayland_socket_is_picked_and_lock_files_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let _s = UnixListener::bind(dir.path().join("wayland-1")).unwrap();
        std::fs::write(dir.path().join("wayland-1.lock"), "").unwrap();
        assert_eq!(only_wayland_socket(dir.path()), Ok("wayland-1".into()));
    }

    #[test]
    fn two_sessions_are_listed_instead_of_guessing_which_to_lock() {
        let dir = tempfile::tempdir().unwrap();
        let _a = UnixListener::bind(dir.path().join("wayland-0")).unwrap();
        let _b = UnixListener::bind(dir.path().join("wayland-1")).unwrap();
        let err = only_wayland_socket(dir.path()).unwrap_err();
        assert!(err.contains("wayland-0, wayland-1"), "{err}");
    }

    #[test]
    fn no_wayland_socket_is_an_error_not_an_x11_fallback() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            only_wayland_socket(dir.path())
                .unwrap_err()
                .contains("only supports Wayland")
        );
    }

    #[test]
    fn a_stale_hyprland_signature_is_replaced_by_the_live_instance() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("hypr/old")).unwrap();
        std::fs::create_dir_all(dir.path().join("hypr/live")).unwrap();
        let _s = UnixListener::bind(dir.path().join("hypr/live/.socket.sock")).unwrap();
        assert!(!hyprland_is_live(dir.path(), "old"));
        assert!(hyprland_is_live(dir.path(), "live"));
        assert_eq!(newest_hyprland_instance(dir.path()), Some("live".into()));
    }
}
