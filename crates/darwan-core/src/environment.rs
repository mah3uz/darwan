pub struct Environment {
    pub wayland: Result<(), String>,
    pub sddm: Result<(), String>,
    pub helper: Result<(), String>,
}

impl Environment {
    // Callers find the Wayland session their own way: the CLI from a TTY, the GUI from its own window.
    pub fn detect(wayland: Result<(), String>) -> Self {
        let on_path = |p: &str| {
            std::env::var_os("PATH")
                .is_some_and(|path| std::env::split_paths(&path).any(|d| d.join(p).is_file()))
        };
        let sddm: Result<(), String> = if on_path("sddm-greeter-qt6") {
            Ok(())
        } else {
            Err("SDDM (Qt 6) is not installed".into())
        };
        let helper = match &sddm {
            Err(e) => Err(e.clone()),
            Ok(()) if std::path::Path::new(crate::paths::HELPER).is_file() => Ok(()),
            Ok(()) => Err("darwan-helper is not installed".into()),
        };
        Self {
            wayland,
            sddm,
            helper,
        }
    }

    pub fn sddm_preview(&self) -> Result<(), String> {
        self.wayland.clone().and(self.sddm.clone())
    }
}
