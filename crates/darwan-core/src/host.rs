use std::path::PathBuf;

pub fn sessions_json() -> String {
    let mut out = Vec::new();
    for (dir, kind) in [
        ("/usr/share/wayland-sessions", "wayland"),
        ("/usr/share/xsessions", "x11"),
    ] {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
        files.sort();
        for path in files
            .iter()
            .filter(|p| p.extension().is_some_and(|x| x == "desktop"))
        {
            let Ok(text) = std::fs::read_to_string(path) else {
                continue;
            };
            if let Some(mut entry) = desktop_entry(&text) {
                entry["file"] = path.file_name().unwrap().to_string_lossy().into();
                entry["type"] = kind.into();
                out.push(entry);
            }
        }
    }
    serde_json::Value::Array(out).to_string()
}

fn desktop_entry(text: &str) -> Option<serde_json::Value> {
    let mut in_entry = false;
    let mut entry = serde_json::json!({ "name": "", "exec": "", "comment": "" });
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        let Some((k, v)) = line.split_once('=').filter(|_| in_entry) else {
            continue;
        };
        match k.trim() {
            "Name" => entry["name"] = v.trim().into(),
            "Exec" => entry["exec"] = v.trim().into(),
            "Comment" => entry["comment"] = v.trim().into(),
            "Hidden" | "NoDisplay" if v.trim() == "true" => return None,
            _ => {}
        }
    }
    Some(entry).filter(|e| e["name"] != "")
}

pub fn user_name() -> String {
    std::env::var("USER").unwrap_or_default()
}

pub fn host_name() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_reads_only_the_main_group_and_skips_hidden() {
        let e = desktop_entry("[Desktop Entry]\nName=Hyprland\nName[de]=X\nExec=Hyprland\n[Desktop Action a]\nName=Other\n").unwrap();
        assert_eq!(e["name"], "Hyprland");
        assert_eq!(e["exec"], "Hyprland");
        assert!(desktop_entry("[Desktop Entry]\nName=A\nNoDisplay=true\n").is_none());
    }
}
