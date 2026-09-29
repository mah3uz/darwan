use std::path::Path;

use super::Env;
use super::set::Owner;

// Colour-scheme generators people run after changing the wallpaper. Darwan runs one only when the user switched it on
// (`wallpaper.colours`) and the wallpaper's owner doesn't make colours itself: a shell that themes from the wallpaper
// (DMS, Caelestia, Noctalia) does it when set through its own command, and a second generator would race its writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generator {
    Matugen,
    Pywal,
    Wallust,
    Hellwal,
}

impl Generator {
    pub const ALL: [Generator; 4] = [
        Generator::Matugen,
        Generator::Pywal,
        Generator::Wallust,
        Generator::Hellwal,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Generator::Matugen => "matugen",
            Generator::Pywal => "pywal",
            Generator::Wallust => "wallust",
            Generator::Hellwal => "hellwal",
        }
    }

    fn program(self) -> &'static str {
        match self {
            Generator::Pywal => "wal",
            other => other.id(),
        }
    }

    // Its own configuration, the sign the user set it up (a bare install isn't).
    fn configured(self, env: &Env) -> bool {
        let c = &env.config_home;
        match self {
            Generator::Matugen => c.join("matugen/config.toml").is_file(),
            Generator::Pywal => {
                c.join("wal/templates").is_dir()
                    || env.home.join(".cache/wal/colors.json").is_file()
            }
            Generator::Wallust => c.join("wallust/wallust.toml").is_file(),
            Generator::Hellwal => c.join("hellwal").is_dir(),
        }
    }

    // pywal sets the wallpaper itself unless told not to (-n), possibly by killing swaybg.
    pub fn argv(self, picture: &Path) -> Vec<String> {
        let p = picture.display().to_string();
        match self {
            Generator::Matugen => vec!["matugen".into(), "image".into(), p],
            Generator::Pywal => vec!["wal".into(), "-n".into(), "-i".into(), p],
            Generator::Wallust => vec!["wallust".into(), "run".into(), p],
            Generator::Hellwal => vec!["hellwal".into(), "-i".into(), p],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub generator: Generator,
    // Why it won't run, when it won't; empty when it may.
    pub reason: String,
}

// matugen with `[config.wallpaper] set = true` sets the wallpaper too, with its own command: run after Darwan set it,
// it would set it a second time.
fn matugen_sets_wallpaper(env: &Env) -> bool {
    let Ok(text) = std::fs::read_to_string(env.config_home.join("matugen/config.toml")) else {
        return false;
    };
    let Ok(v) = text.parse::<toml::Table>() else {
        return false;
    };
    v.get("config")
        .and_then(|c| c.get("wallpaper"))
        .and_then(|w| w.get("set"))
        .and_then(toml::Value::as_bool)
        == Some(true)
}

// The generators set up on this machine, each with why it won't run here, if it won't.
pub fn found(env: &Env, owner: Option<&Owner>) -> Vec<Found> {
    Generator::ALL
        .into_iter()
        .filter(|g| (env.which)(g.program()) && g.configured(env))
        .map(|generator| {
            let reason = match owner {
                Some(o) if o.caps().themes => format!(
                    "{} makes your colours from the wallpaper itself",
                    o.tool.name()
                ),
                // A theming shell beside a separate drawer still owns the colours.
                Some(Owner {
                    shell: Some(shell), ..
                }) => format!(
                    "{} makes your colours from the wallpaper itself",
                    shell.name()
                ),
                _ if generator == Generator::Matugen && matugen_sets_wallpaper(env) => {
                    "matugen is set up to change the wallpaper itself".into()
                }
                _ => String::new(),
            };
            Found { generator, reason }
        })
        .collect()
}

// What to run after `picture` became the wallpaper: the switched-on generators that may run. A video's still stands in
// for it, since the generators read pictures.
pub fn after(
    env: &Env,
    owner: Option<&Owner>,
    enabled: &[String],
    picture: &Path,
) -> Vec<Vec<String>> {
    found(env, owner)
        .into_iter()
        .filter(|f| f.reason.is_empty() && enabled.iter().any(|e| e == f.generator.id()))
        .map(|f| f.generator.argv(picture))
        .collect()
}

// Runs the switched-on generators after `file` became the wallpaper, one after another; each result names its id.
// `cache_home` holds the still a video is read from.
pub fn run_after(
    env: &Env,
    owner: Option<&Owner>,
    enabled: &[String],
    file: &Path,
    cache_home: &Path,
) -> Vec<(String, Result<(), String>)> {
    if enabled.is_empty() {
        return Vec::new();
    }
    let still;
    let picture = if super::set::is_video(file) {
        match super::library::item(file)
            .map(|i| super::library::thumbnail(cache_home, &i, super::library::Size::XxLarge))
        {
            Some(Ok(t)) => {
                still = t;
                still.as_path()
            }
            _ => {
                return vec![(
                    "colours".into(),
                    Err("no still to make colours from".into()),
                )];
            }
        }
    } else {
        file
    };
    after(env, owner, enabled, picture)
        .into_iter()
        .map(|argv| {
            let name = argv[0].clone();
            let result = std::process::Command::new(&argv[0])
                .args(&argv[1..])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output()
                .map_err(|e| format!("{name}: {e}"))
                .and_then(|o| {
                    o.status.success().then_some(()).ok_or_else(|| {
                        let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                        if err.is_empty() {
                            format!("{name} failed ({})", o.status)
                        } else {
                            err
                        }
                    })
                });
            (name, result)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::fixture::Fixture;
    use super::super::set::owner;
    use super::*;

    fn nothing(_: &str, _: &[&str]) -> Option<String> {
        None
    }

    #[test]
    fn a_generator_runs_only_when_set_up_switched_on_and_no_shell_themes() {
        let f = Fixture::new();
        f.write(
            "config/matugen/config.toml",
            "[templates.kitty]\ninput_path = 'a'\noutput_path = 'b'\n",
        );
        f.write("config/wal/templates/colors.css", "");
        f.process(1, &["hyprpaper"], Some("wayland-1"));
        let mut env = f.env(nothing);
        env.which = |p| p == "matugen" || p == "wal";
        let o = owner(&env);
        let pic = Path::new("/w/a.png");
        assert!(
            after(&env, o.as_ref(), &[], pic).is_empty(),
            "off until the user switches it on"
        );
        assert_eq!(
            after(&env, o.as_ref(), &["pywal".into()], pic),
            vec![vec!["wal", "-n", "-i", "/w/a.png"]],
            "pywal always gets -n, or it sets the wallpaper itself"
        );

        // DMS beside hyprpaper: hyprpaper draws, but DMS still owns the colours.
        f.process(2, &["/usr/bin/dms", "run", "--session"], Some("wayland-1"));
        let o = owner(&env);
        assert_eq!(
            o.as_ref().unwrap().shell,
            Some(super::super::set::Tool::Dms)
        );
        assert!(after(&env, o.as_ref(), &["pywal".into()], pic).is_empty());
    }

    #[test]
    fn a_theming_shell_or_a_self_setting_matugen_is_left_alone() {
        let f = Fixture::new();
        f.write(
            "config/matugen/config.toml",
            "[config.wallpaper]\nset = true\ncommand = 'awww'\n",
        );
        f.process(1, &["awww-daemon"], Some("wayland-1"));
        let mut env = f.env(nothing);
        env.which = |p| p == "matugen";
        let found_here = found(&env, owner(&env).as_ref());
        assert!(
            found_here[0].reason.contains("change the wallpaper itself"),
            "it would set it a second time"
        );

        let g = Fixture::new();
        g.write("config/matugen/config.toml", "");
        g.process(1, &["/usr/bin/dms", "run", "--session"], Some("wayland-1"));
        let mut env = g.env(nothing);
        env.which = |p| p == "matugen";
        let o = owner(&env);
        assert!(
            found(&env, o.as_ref())[0].reason.contains("DMS"),
            "DMS makes the colours when set through it"
        );
        assert!(after(&env, o.as_ref(), &["matugen".into()], Path::new("/w/a.png")).is_empty());
    }
}
