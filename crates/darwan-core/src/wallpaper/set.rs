use std::path::{Path, PathBuf};

use super::{
    Env, Proc, is_caelestia, is_default_namespace, is_dms, is_noctalia, is_noctalia_legacy,
    is_omarchy_shell, is_omarchy_swaybg, processes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Dms,
    Noctalia,
    NoctaliaLegacy,
    Caelestia,
    Omarchy,
    OmarchyLegacy,
    Plasma,
    Gnome,
    Cinnamon,
    Mate,
    Xfce,
    Sway,
    Waypaper,
    Hyprpaper,
    Awww,
    Swww,
    Wpaperd,
    Mpvpaper,
    Gslapper,
    Swaybg,
    Wbg,
}

// What a tool can do with a wallpaper. `themes`: it makes the desktop's colours from the wallpaper itself, so Darwan
// never runs a colour generator on top of it. `persists`: the change survives a restart of the tool or a new login.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    pub per_output: bool,
    pub video: bool,
    pub themes: bool,
    pub persists: bool,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Dms => "DMS",
            Tool::Noctalia | Tool::NoctaliaLegacy => "Noctalia",
            Tool::Caelestia => "Caelestia",
            Tool::Omarchy | Tool::OmarchyLegacy => "Omarchy",
            Tool::Plasma => "KDE Plasma",
            Tool::Gnome => "GNOME",
            Tool::Cinnamon => "Cinnamon",
            Tool::Mate => "MATE",
            Tool::Xfce => "Xfce",
            Tool::Sway => "sway",
            Tool::Waypaper => "waypaper",
            Tool::Hyprpaper => "hyprpaper",
            Tool::Awww => "awww",
            Tool::Swww => "swww",
            Tool::Wpaperd => "wpaperd",
            Tool::Mpvpaper => "mpvpaper",
            Tool::Gslapper => "gSlapper",
            Tool::Swaybg => "swaybg",
            Tool::Wbg => "wbg",
        }
    }

    pub fn caps(self) -> Caps {
        let c = |per_output, video, themes, persists| Caps {
            per_output,
            video,
            themes,
            persists,
        };
        match self {
            Tool::Dms | Tool::Noctalia | Tool::NoctaliaLegacy => c(true, false, true, true),
            Tool::Caelestia => c(false, false, true, true),
            // Omarchy's video support is newer than its last release; images are what every version shows.
            Tool::Omarchy | Tool::OmarchyLegacy => c(false, false, false, true),
            Tool::Plasma | Tool::Gnome | Tool::Cinnamon | Tool::Mate => {
                c(false, false, false, true)
            }
            Tool::Xfce => c(true, false, false, true),
            Tool::Sway => c(true, false, false, false),
            Tool::Waypaper => c(true, false, false, true),
            Tool::Hyprpaper | Tool::Wpaperd => c(true, false, false, false),
            Tool::Awww | Tool::Swww => c(true, false, false, true),
            Tool::Mpvpaper | Tool::Gslapper => c(true, true, false, false),
            Tool::Swaybg => c(true, false, false, false),
            Tool::Wbg => c(false, false, false, false),
        }
    }
}

// Who draws the desktop wallpaper in this session. `shell`: a shell that themes from the wallpaper runs too, but a
// separate drawer is what shows it; the user decides whether the shell is told as well.
#[derive(Debug, Clone)]
pub struct Owner {
    pub tool: Tool,
    pub shell: Option<Tool>,
    proc: Option<Proc>,
}

impl Owner {
    pub fn caps(&self) -> Caps {
        self.tool.caps()
    }

    fn argv(&self) -> &[String] {
        self.proc.as_ref().map_or(&[], |p| &p.argv)
    }
}

const DRAWERS: &[(&str, Tool)] = &[
    ("hyprpaper", Tool::Hyprpaper),
    ("awww-daemon", Tool::Awww),
    ("swww-daemon", Tool::Swww),
    ("wpaperd", Tool::Wpaperd),
    ("mpvpaper", Tool::Mpvpaper),
    ("gslapper", Tool::Gslapper),
    ("swaybg", Tool::Swaybg),
    ("wbg", Tool::Wbg),
];

fn drawer(procs: &[Proc], env: &Env) -> Option<(Tool, Proc)> {
    DRAWERS.iter().find_map(|(exe, tool)| {
        procs
            .iter()
            .find(|p| {
                p.exe_name() == *exe
                    && is_default_namespace(p)
                    && !is_omarchy_swaybg(p)
                    && !(*tool == Tool::Swaybg && under_sway(p, procs, env))
            })
            .map(|p| (*tool, p.clone()))
    })
}

fn under_sway(p: &Proc, procs: &[Proc], env: &Env) -> bool {
    env.current_desktop.as_deref() == Some("sway")
        || p.parent()
            .and_then(|pid| procs.iter().find(|q| q.pid == pid))
            .is_some_and(|parent| parent.exe_name() == "sway")
}

// The highest-level owner wins: a shell, the desktop environment's own background, sway, Omarchy's script, waypaper
// when its backend is what runs, then the drawer itself.
pub fn owner(env: &Env) -> Option<Owner> {
    let procs = processes(env);
    let running = |f: fn(&Proc) -> bool| procs.iter().find(|p| f(p)).cloned();
    let shell = [
        (is_dms as fn(&Proc) -> bool, Tool::Dms),
        (is_noctalia, Tool::Noctalia),
        (is_noctalia_legacy, Tool::NoctaliaLegacy),
        (is_caelestia, Tool::Caelestia),
        (is_omarchy_shell, Tool::Omarchy),
    ]
    .into_iter()
    .find_map(|(f, tool)| running(f).map(|p| (tool, p)));
    let drawn = drawer(&procs, env);

    match (shell, drawn.clone()) {
        // A theming shell with a drawer beside it: the drawer shows the wallpaper (the shell's own layer is off).
        (Some((shell, _)), Some((tool, p))) if shell.caps().themes => {
            return Some(Owner {
                tool,
                shell: Some(shell),
                proc: Some(p),
            });
        }
        (Some((tool, p)), _) => {
            return Some(Owner {
                tool,
                shell: None,
                proc: Some(p),
            });
        }
        _ => {}
    }

    let desktop = env.current_desktop.as_deref().unwrap_or("");
    let has = |exe: &str| procs.iter().find(|p| p.exe_name() == exe).cloned();
    let de = [
        ("kde", "plasmashell", Tool::Plasma),
        ("gnome", "gnome-shell", Tool::Gnome),
        ("budgie", "budgie-panel", Tool::Gnome),
        ("cinnamon", "cinnamon", Tool::Cinnamon),
        ("mate", "mate-session", Tool::Mate),
        ("xfce", "xfdesktop", Tool::Xfce),
    ]
    .into_iter()
    .find_map(|(name, exe, tool)| {
        desktop
            .split(':')
            .any(|d| d.contains(name))
            .then(|| has(exe).map(|p| (tool, p)))
            .flatten()
    });
    if let Some((tool, p)) = de {
        return Some(Owner {
            tool,
            shell: None,
            proc: Some(p),
        });
    }

    if let Some(p) = procs
        .iter()
        .find(|p| p.exe_name() == "swaybg" && under_sway(p, &procs, env))
    {
        return Some(Owner {
            tool: Tool::Sway,
            shell: None,
            proc: Some(p.clone()),
        });
    }
    if let Some(p) = running(is_omarchy_swaybg) {
        return Some(Owner {
            tool: Tool::OmarchyLegacy,
            shell: None,
            proc: Some(p),
        });
    }

    let (tool, p) = drawn?;
    if (env.which)("waypaper") && waypaper_backend(env).as_deref() == Some(backend_name(tool)) {
        return Some(Owner {
            tool: Tool::Waypaper,
            shell: None,
            proc: Some(p),
        });
    }
    Some(Owner {
        tool,
        shell: None,
        proc: Some(p),
    })
}

fn backend_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Awww => "awww",
        Tool::Swww => "swww",
        Tool::Hyprpaper => "hyprpaper",
        Tool::Mpvpaper => "mpvpaper",
        Tool::Gslapper => "gslapper",
        Tool::Swaybg => "swaybg",
        _ => "",
    }
}

fn waypaper_backend(env: &Env) -> Option<String> {
    let text = std::fs::read_to_string(env.config_home.join("waypaper/config.ini")).ok()?;
    text.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == "backend").then(|| v.trim().to_string())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    All,
    Outputs(Vec<String>),
}

// `outputs`: every connected output by connector name (Qt's QScreen::name), for tools that need one call per output.
#[derive(Debug, Clone)]
pub struct Request {
    pub path: PathBuf,
    pub target: Target,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    Exit,
    // DMS answers `SUCCESS: …` or `ERROR: …`; its exit status isn't a promise.
    Starts(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Respawn {
    pub old_pid: u32,
    pub old_start: Option<u64>,
    pub argv: Vec<String>,
    pub unit: String,
    // Darwan's own earlier drawer: stopped through its unit.
    pub old_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Run {
        argv: Vec<String>,
        reply: Reply,
    },
    Line {
        socket: PathBuf,
        line: String,
        reply: &'static str,
    },
    Respawn(Respawn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub tool: Tool,
    pub steps: Vec<Step>,
    // Tried when the first steps run but the read-back disagrees (hyprpaper ≤0.7's commands).
    pub fallback: Vec<Step>,
    // Restarting a drawer Darwan didn't start needs the user's consent, once per tool.
    pub restarts: Option<Tool>,
    pub notes: Vec<String>,
}

const VIDEO: &[&str] = &["mp4", "mkv", "webm", "mov", "avi", "m4v"];

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO.contains(&e.to_ascii_lowercase().as_str()))
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

fn run(parts: &[&str]) -> Step {
    Step::Run {
        argv: argv(parts),
        reply: Reply::Exit,
    }
}

fn targets(req: &Request) -> Vec<String> {
    match &req.target {
        Target::All => req.outputs.clone(),
        Target::Outputs(o) => o.clone(),
    }
}

fn file_uri(path: &Path) -> String {
    let mut out = String::from("file://");
    for b in path.as_os_str().as_encoded_bytes() {
        match *b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

// Checks the request against what the owner can show, then lists exactly what would run. Pure apart from reading the
// owner's own state (DMS's per-monitor mode, Xfce's property list), so every route is testable on fixtures.
pub fn plan(env: &Env, owner: &Owner, req: &Request) -> Result<Plan, String> {
    let caps = owner.caps();
    let name = owner.tool.name();
    if !req.path.is_absolute() {
        return Err(format!("{} is not an absolute path", req.path.display()));
    }
    if is_video(&req.path) && !caps.video {
        return Err(format!(
            "{name} can't show videos; mpvpaper or gSlapper can"
        ));
    }
    if matches!(req.target, Target::Outputs(_)) && !caps.per_output {
        return Err(format!("{name} shows one wallpaper on every screen"));
    }
    let path = req.path.to_str().ok_or("the path is not valid UTF-8")?;
    if path.chars().any(char::is_control) {
        return Err("the path contains control characters".into());
    }
    let mut p = Plan {
        tool: owner.tool,
        steps: Vec::new(),
        fallback: Vec::new(),
        restarts: None,
        notes: Vec::new(),
    };
    let each = |f: &dyn Fn(&str) -> Step| targets(req).iter().map(|o| f(o)).collect::<Vec<_>>();

    match owner.tool {
        Tool::Dms => {
            let dms = |args: &[&str]| Step::Run {
                argv: [&["dms", "ipc", "call", "wallpaper"][..], args]
                    .concat()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                reply: Reply::Starts("SUCCESS"),
            };
            // `set` refuses while DMS is in per-monitor mode, and `setFor` switches that mode on.
            p.steps = if req.target == Target::All && !dms_per_monitor(env) {
                vec![dms(&["set", path])]
            } else {
                each(&|o| dms(&["setFor", o, path]))
            };
            if dms_per_mode(env) {
                p.notes.push(
                    "DMS keeps light and dark wallpapers; this sets the current mode's".into(),
                );
            }
        }
        Tool::Noctalia => {
            p.steps = match req.target {
                Target::All => vec![run(&["noctalia", "msg", "wallpaper-set", path])],
                Target::Outputs(_) => {
                    each(&|o| run(&["noctalia", "msg", "wallpaper-set", o, path]))
                }
            };
        }
        Tool::NoctaliaLegacy => {
            let legacy = |screen: &str| {
                run(&[
                    "qs",
                    "-c",
                    "noctalia-shell",
                    "ipc",
                    "call",
                    "wallpaper",
                    "set",
                    path,
                    screen,
                ])
            };
            p.steps = match req.target {
                Target::All => vec![legacy("all")],
                Target::Outputs(_) => each(&|o| legacy(o)),
            };
        }
        Tool::Caelestia => p.steps = vec![run(&["caelestia", "shell", "wallpaper", "set", path])],
        Tool::Omarchy | Tool::OmarchyLegacy => p.steps = vec![run(&["omarchy-theme-bg-set", path])],
        Tool::Plasma => {
            // Plasma numbers its screens itself; for every screen the image plugin gets the same file.
            let uri = file_uri(&req.path);
            p.steps = (0..req.outputs.len().max(1))
                .map(|n| {
                    run(&[
                        "busctl",
                        "--user",
                        "call",
                        "org.kde.plasmashell",
                        "/PlasmaShell",
                        "org.kde.PlasmaShell",
                        "setWallpaper",
                        "sa{sv}u",
                        "org.kde.image",
                        "1",
                        "Image",
                        "s",
                        &uri,
                        &n.to_string(),
                    ])
                })
                .collect();
        }
        Tool::Gnome => {
            let uri = file_uri(&req.path);
            p.steps = vec![
                run(&[
                    "gsettings",
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri",
                    &uri,
                ]),
                run(&[
                    "gsettings",
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    &uri,
                ]),
            ];
        }
        Tool::Cinnamon => {
            let uri = file_uri(&req.path);
            p.steps = vec![run(&[
                "gsettings",
                "set",
                "org.cinnamon.desktop.background",
                "picture-uri",
                &uri,
            ])];
        }
        Tool::Mate => {
            p.steps = vec![run(&[
                "gsettings",
                "set",
                "org.mate.background",
                "picture-filename",
                path,
            ])]
        }
        Tool::Xfce => {
            let keys = xfce_image_keys(env, &req.target);
            if keys.is_empty() {
                return Err(
                    "Xfce has no desktop image for that screen yet; set one once in its settings"
                        .into(),
                );
            }
            p.steps = keys
                .iter()
                .map(|k| run(&["xfconf-query", "-c", "xfce4-desktop", "-p", k, "-s", path]))
                .collect();
        }
        Tool::Sway => {
            let mode = flag_after(owner.argv(), &["-m", "--mode"]).unwrap_or_else(|| "fill".into());
            // sway joins and re-parses the command, so the path is quoted for it.
            let quoted = format!("\"{}\"", path.replace('\\', "\\\\").replace('"', "\\\""));
            let set = |out: &str| run(&["swaymsg", &format!("output {out} bg {quoted} {mode}")]);
            p.steps = match req.target {
                Target::All => vec![set("*")],
                Target::Outputs(_) => each(&|o| set(o)),
            };
        }
        Tool::Waypaper => {
            p.steps = match req.target {
                Target::All => vec![run(&["waypaper", "--wallpaper", path])],
                Target::Outputs(_) => {
                    each(&|o| run(&["waypaper", "--wallpaper", path, "--monitor", o]))
                }
            };
        }
        Tool::Hyprpaper => {
            // hyprctl splits its argument on commas.
            if path.contains(',') {
                return Err("hyprpaper can't take a path with a comma in it".into());
            }
            if targets(req).is_empty() {
                return Err("no screens to set".into());
            }
            p.steps =
                each(&|o| run(&["hyprctl", "hyprpaper", "wallpaper", &format!("{o},{path}")]));
            p.fallback = std::iter::once(run(&["hyprctl", "hyprpaper", "preload", path]))
                .chain(each(&|o| {
                    run(&["hyprctl", "hyprpaper", "wallpaper", &format!("{o},{path}")])
                }))
                .collect();
            p.notes.push(
                "hyprpaper forgets this when it restarts unless hyprpaper.conf says so too".into(),
            );
        }
        Tool::Awww | Tool::Swww => {
            let program = if owner.tool == Tool::Awww {
                "awww"
            } else {
                "swww"
            };
            let mut a = argv(&[program, "img"]);
            if let Target::Outputs(o) = &req.target {
                a.extend(["-o".to_string(), o.join(",")]);
            }
            a.push(path.to_string());
            p.steps = vec![Step::Run {
                argv: a,
                reply: Reply::Exit,
            }];
        }
        Tool::Wpaperd => {
            p.steps = match req.target {
                Target::All => vec![run(&["wpaperctl", "set", path])],
                Target::Outputs(_) => each(&|o| run(&["wpaperctl", "set", path, o])),
            };
        }
        Tool::Mpvpaper => {
            if let Some(socket) = mpv_socket(owner.argv()) {
                let line = serde_json::json!({ "command": ["loadfile", path] }).to_string();
                p.steps = vec![Step::Line {
                    socket,
                    line,
                    reply: "\"error\":\"success\"",
                }];
            } else {
                respawn(owner, req, &mut p, |argv| replace_last_file(argv, path))?;
            }
        }
        Tool::Gslapper => {
            if let Some(socket) = flag_after(owner.argv(), &["-I", "--ipc-socket"]) {
                p.steps = vec![Step::Line {
                    socket: PathBuf::from(socket),
                    line: format!("change {path}"),
                    reply: "OK",
                }];
            } else {
                respawn(owner, req, &mut p, |argv| replace_last_file(argv, path))?;
            }
        }
        Tool::Swaybg => respawn(owner, req, &mut p, |argv| {
            swaybg_argv(argv, &req.target, path)
        })?,
        Tool::Wbg => respawn(owner, req, &mut p, |_| Some(argv(&["wbg", path])))?,
    }
    Ok(p)
}

fn respawn(
    owner: &Owner,
    req: &Request,
    p: &mut Plan,
    build: impl Fn(&[String]) -> Option<Vec<String>>,
) -> Result<(), String> {
    let proc = owner.proc.as_ref().ok_or("no running process to replace")?;
    let name = owner.tool.name();
    let service = proc.service();
    let ours = service
        .as_deref()
        .is_some_and(|u| u.starts_with(UNIT_PREFIX));
    if let (Some(unit), false) = (&service, ours) {
        return Err(format!(
            "{name} runs as {unit}, which would start it again with the old file; change the file in that unit"
        ));
    }
    let argv =
        build(&proc.argv).ok_or_else(|| format!("{name} can't show that on the chosen screen"))?;
    if matches!(owner.tool, Tool::Mpvpaper | Tool::Gslapper)
        && matches!(req.target, Target::Outputs(_))
    {
        return Err(format!("{name} is changed as a whole; choose all screens"));
    }
    p.steps = vec![Step::Respawn(Respawn {
        old_pid: proc.pid,
        old_start: proc.start_time(),
        argv,
        unit: format!("{UNIT_PREFIX}{}", owner.tool.name().to_lowercase()),
        old_unit: service.filter(|_| ours),
    })];
    if !ours {
        p.restarts = Some(owner.tool);
    }
    p.notes.push(format!(
        "{name} is restarted with the new file; it forgets it after logging out"
    ));
    Ok(())
}

pub const UNIT_PREFIX: &str = "darwan-wallpaper-";

// The tools Darwan may have to restart to change the wallpaper; the user allows each once (`wallpaper.restart`).
pub const RESTARTED: &[Tool] = &[Tool::Swaybg, Tool::Mpvpaper, Tool::Gslapper, Tool::Wbg];

fn flag_after(argv: &[String], flags: &[&str]) -> Option<String> {
    argv.windows(2)
        .find(|w| flags.contains(&w[0].as_str()))
        .map(|w| w[1].clone())
        .or_else(|| {
            argv.iter().find_map(|a| {
                flags
                    .iter()
                    .filter(|f| f.starts_with("--"))
                    .find_map(|f| a.strip_prefix(&format!("{f}=")).map(str::to_string))
            })
        })
}

fn mpv_socket(argv: &[String]) -> Option<PathBuf> {
    argv.iter().find_map(|a| {
        a.split_whitespace()
            .find_map(|o| {
                o.strip_prefix("input-ipc-server=")
                    .or_else(|| o.strip_prefix("--input-ipc-server="))
            })
            .map(PathBuf::from)
    })
}

// mpvpaper and gSlapper take the file last.
fn replace_last_file(argv: &[String], path: &str) -> Option<Vec<String>> {
    let mut out = argv.to_vec();
    let last = out.len().checked_sub(1).filter(|i| *i > 0)?;
    out[last] = path.to_string();
    Some(out)
}

// swaybg: `-o OUT` starts a group, `-i` sets its image. Every `-i` for all screens, or only the chosen groups; a screen
// without its own group gets one, keeping the mode the rest use.
fn swaybg_argv(argv: &[String], target: &Target, path: &str) -> Option<Vec<String>> {
    let mut out = argv.to_vec();
    let mut group: Option<String> = None;
    let mut done: Vec<String> = Vec::new();
    let mut i = 1;
    while i < out.len() {
        let flag = out[i].clone();
        match flag.as_str() {
            "-o" | "--output" if i + 1 < out.len() => {
                group = Some(out[i + 1].clone());
                i += 2;
            }
            "-i" | "--image" if i + 1 < out.len() => {
                let wanted = match target {
                    Target::All => true,
                    Target::Outputs(o) => group.as_ref().is_some_and(|g| o.contains(g)),
                };
                if wanted {
                    out[i + 1] = path.to_string();
                    if let Some(g) = &group {
                        done.push(g.clone());
                    }
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    if let Target::Outputs(o) = target {
        let mode = flag_after(argv, &["-m", "--mode"]).unwrap_or_else(|| "fill".into());
        for name in o.iter().filter(|n| !done.contains(n)) {
            out.extend(["-o", name, "-i", path, "-m", &mode].map(str::to_string));
        }
    }
    Some(out)
}

fn dms_session(env: &Env) -> Option<serde_json::Value> {
    let text =
        std::fs::read_to_string(env.state_home.join("DankMaterialShell/session.json")).ok()?;
    serde_json::from_str(&text).ok()
}

fn dms_per_monitor(env: &Env) -> bool {
    dms_session(env).and_then(|v| v.get("perMonitorWallpaper")?.as_bool()) == Some(true)
}

fn dms_per_mode(env: &Env) -> bool {
    dms_session(env).and_then(|v| v.get("perModeWallpaper")?.as_bool()) == Some(true)
}

// Xfce keeps an image per monitor and workspace; only the ones it already has are set.
fn xfce_image_keys(env: &Env, target: &Target) -> Vec<String> {
    let listed = (env.run)("xfconf-query", &["-c", "xfce4-desktop", "-l"]).unwrap_or_default();
    listed
        .lines()
        .map(str::trim)
        .filter(|k| k.starts_with("/backdrop/") && k.ends_with("/last-image"))
        .filter(|k| match target {
            Target::All => true,
            Target::Outputs(o) => o.iter().any(|name| k.contains(&format!("/monitor{name}/"))),
        })
        .map(str::to_string)
        .collect()
}

pub trait Runner {
    // Stdout on success; the tool's own error text otherwise.
    fn run(&self, argv: &[String]) -> Result<String, String>;
    fn line(&self, socket: &Path, line: &str) -> Result<String, String>;
    fn respawn(&self, r: &Respawn) -> Result<(), String>;
}

pub fn apply(steps: &[Step], runner: &dyn Runner) -> Result<(), String> {
    for step in steps {
        match step {
            Step::Run { argv, reply } => {
                let out = runner.run(argv)?;
                if let Reply::Starts(ok) = reply
                    && !out.trim_start().starts_with(ok)
                {
                    return Err(out.trim().to_string());
                }
            }
            Step::Line {
                socket,
                line,
                reply,
            } => {
                let out = runner.line(socket, line)?;
                if !out.contains(reply) {
                    return Err(out.trim().to_string());
                }
            }
            Step::Respawn(r) => runner.respawn(r)?,
        }
    }
    Ok(())
}

// The real runner: argv only, never a shell.
pub struct System;

impl Runner for System {
    fn run(&self, argv: &[String]) -> Result<String, String> {
        let (program, args) = argv.split_first().ok_or("nothing to run")?;
        let out = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| format!("{program}: {e}"))?;
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        if out.status.success() {
            return Ok(stdout);
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        let said = [stderr.trim(), stdout.trim()]
            .into_iter()
            .find(|t| !t.is_empty());
        Err(said.map_or_else(
            || format!("{program} failed ({})", out.status),
            str::to_string,
        ))
    }

    fn line(&self, socket: &Path, line: &str) -> Result<String, String> {
        use std::io::{BufRead, Write};
        let at = |e: std::io::Error| format!("{}: {e}", socket.display());
        let stream = std::os::unix::net::UnixStream::connect(socket).map_err(at)?;
        let limit = Some(std::time::Duration::from_secs(3));
        stream.set_read_timeout(limit).map_err(at)?;
        stream.set_write_timeout(limit).map_err(at)?;
        (&stream)
            .write_all(format!("{line}\n").as_bytes())
            .map_err(at)?;
        let mut reply = String::new();
        std::io::BufReader::new(&stream)
            .read_line(&mut reply)
            .map_err(at)?;
        Ok(reply)
    }

    // The new drawer starts first, detached from Darwan so it outlives the GUI; the old one stops once the new one
    // runs, so the screen is never bare. Only the exact process that was planned is stopped.
    fn respawn(&self, r: &Respawn) -> Result<(), String> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        let unit = format!("{}-{stamp}", r.unit);
        let mut cmd = if super::on_path("systemd-run") {
            let mut c = std::process::Command::new("systemd-run");
            c.args(["--user", "--collect", "--quiet", &format!("--unit={unit}")]);
            for var in [
                "WAYLAND_DISPLAY",
                "XDG_RUNTIME_DIR",
                "HYPRLAND_INSTANCE_SIGNATURE",
                "SWAYSOCK",
                "NIRI_SOCKET",
            ] {
                if let Ok(v) = std::env::var(var) {
                    c.arg(format!("--setenv={var}={v}"));
                }
            }
            c.arg("--").args(&r.argv);
            c
        } else {
            let mut c = std::process::Command::new("setsid");
            c.arg("-f").args(&r.argv);
            c
        };
        let started = cmd
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("cannot start {}: {e}", r.argv[0]))?;
        if !started.success() {
            return Err(format!("{} did not start", r.argv[0]));
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Some(old) = &r.old_unit {
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "stop", old])
                .status();
            return Ok(());
        }
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", r.old_pid)).unwrap_or_default();
        let start = stat
            .rsplit_once(')')
            .and_then(|(_, rest)| rest.split_whitespace().nth(19)?.parse::<u64>().ok());
        if start.is_some()
            && start == r.old_start
            && let Some(pid) = rustix::process::Pid::from_raw(r.old_pid as i32)
        {
            let _ = rustix::process::kill_process(pid, rustix::process::Signal::TERM);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    // The owner reports the new file.
    Shown,
    // It took the command but can't be asked what it shows.
    Sent,
}

// Runs the plan, then waits up to two seconds for the owner to report the file (tools apply a change a moment after
// they answer); a disagreeing read-back tries the fallback (hyprpaper ≤0.7) once.
pub fn carry_out(
    env: &Env,
    owner: &Owner,
    req: &Request,
    plan: &Plan,
    runner: &dyn Runner,
) -> Result<Outcome, String> {
    apply(&plan.steps, runner)?;
    let mut shown = wait_for(env, owner, req);
    if shown == Some(false) && !plan.fallback.is_empty() {
        apply(&plan.fallback, runner)?;
        shown = wait_for(env, owner, req);
    }
    match shown {
        Some(true) => Ok(Outcome::Shown),
        None => Ok(Outcome::Sent),
        Some(false) => Err(format!(
            "{} took the command but doesn't show {}",
            owner.tool.name(),
            req.path.display()
        )),
    }
}

fn wait_for(env: &Env, owner: &Owner, req: &Request) -> Option<bool> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let shown = shows(env, owner, req);
        if shown != Some(false) || std::time::Instant::now() > deadline {
            return shown;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}

// What the owner shows now, read back through its own getter; None when it can't be asked.
pub fn current(env: &Env, owner: &Owner, output: Option<&str>) -> Option<PathBuf> {
    let ask = |program: &str, args: &[&str]| (env.run)(program, args);
    let path_in = |text: String| -> Option<PathBuf> {
        let t = text.trim().trim_matches('\'').trim();
        let t = t.strip_prefix("file://").unwrap_or(t);
        (!t.is_empty()).then(|| PathBuf::from(percent_decode(t)))
    };
    match owner.tool {
        Tool::Dms => match output {
            Some(o) => ask("dms", &["ipc", "call", "wallpaper", "getFor", o]).and_then(path_in),
            None => ask("dms", &["ipc", "call", "wallpaper", "get"]).and_then(path_in),
        },
        Tool::Noctalia => match output {
            Some(o) => ask("noctalia", &["msg", "wallpaper-get", o]).and_then(path_in),
            None => ask("noctalia", &["msg", "wallpaper-get"]).and_then(path_in),
        },
        Tool::NoctaliaLegacy => ask(
            "qs",
            &[
                "-c",
                "noctalia-shell",
                "ipc",
                "call",
                "wallpaper",
                "get",
                output.unwrap_or("all"),
            ],
        )
        .and_then(path_in),
        Tool::Caelestia => super::caelestia(env),
        Tool::Omarchy | Tool::OmarchyLegacy => super::omarchy(env),
        Tool::Gnome => ask(
            "gsettings",
            &["get", "org.gnome.desktop.background", "picture-uri"],
        )
        .and_then(path_in),
        Tool::Cinnamon => ask(
            "gsettings",
            &["get", "org.cinnamon.desktop.background", "picture-uri"],
        )
        .and_then(path_in),
        Tool::Mate => ask(
            "gsettings",
            &["get", "org.mate.background", "picture-filename"],
        )
        .and_then(path_in),
        Tool::Hyprpaper => {
            let listed = ask("hyprctl", &["hyprpaper", "listactive"])?;
            listed.lines().find_map(|l| {
                let (mon, file) = l.split_once(" = ").or_else(|| l.split_once(": "))?;
                output
                    .is_none_or(|o| mon.trim() == o)
                    .then(|| PathBuf::from(file.trim()))
            })
        }
        Tool::Awww | Tool::Swww => {
            let program = if owner.tool == Tool::Awww {
                "awww"
            } else {
                "swww"
            };
            let listed = ask(program, &["query"])?;
            listed.lines().find_map(|l| {
                if output.is_some_and(|o| !l.contains(&format!("{o}:"))) {
                    return None;
                }
                l.split_once("image: ")
                    .map(|(_, f)| PathBuf::from(f.trim()))
            })
        }
        // A restarted drawer shows what its new command line says.
        Tool::Swaybg | Tool::Mpvpaper | Tool::Gslapper | Tool::Wbg | Tool::Sway => {
            let exe = match owner.tool {
                Tool::Mpvpaper => "mpvpaper",
                Tool::Gslapper => "gslapper",
                Tool::Wbg => "wbg",
                _ => "swaybg",
            };
            processes(env)
                .into_iter()
                .filter(|p| p.exe_name() == exe)
                .find_map(|p| super::last_file_argument(&p).or_else(|| super::swaybg(&p)))
        }
        Tool::Plasma | Tool::Xfce | Tool::Waypaper | Tool::Wpaperd => None,
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// Whether the owner shows `path` now, comparing canonical paths (Omarchy and hyprctl resolve symlinks).
pub fn shows(env: &Env, owner: &Owner, req: &Request) -> Option<bool> {
    let want = std::fs::canonicalize(&req.path).unwrap_or_else(|_| req.path.clone());
    let same = |p: PathBuf| std::fs::canonicalize(&p).unwrap_or(p) == want;
    match &req.target {
        Target::Outputs(o) if owner.caps().per_output => {
            let mut all = true;
            for out in o {
                all &= same(current(env, owner, Some(out))?);
            }
            Some(all)
        }
        _ => current(env, owner, None).map(same),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::Fixture;
    use super::*;
    use std::cell::RefCell;

    fn nothing(_: &str, _: &[&str]) -> Option<String> {
        None
    }

    fn found(f: &Fixture) -> Owner {
        owner(&f.env(nothing)).expect("an owner")
    }

    fn all(path: &Path, outputs: &[&str]) -> Request {
        Request {
            path: path.to_path_buf(),
            target: Target::All,
            outputs: outputs.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn only(path: &Path, outputs: &[&str], chosen: &[&str]) -> Request {
        Request {
            target: Target::Outputs(chosen.iter().map(|s| s.to_string()).collect()),
            ..all(path, outputs)
        }
    }

    fn argvs(p: &Plan) -> Vec<Vec<String>> {
        p.steps
            .iter()
            .filter_map(|s| match s {
                Step::Run { argv, .. } => Some(argv.clone()),
                _ => None,
            })
            .collect()
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    const DMS: &[&str] = &["/usr/bin/dms", "run", "--session"];

    #[test]
    fn a_shell_owns_the_wallpaper_unless_a_drawer_runs_beside_it() {
        let f = Fixture::new();
        f.process(1, DMS, Some("wayland-1"));
        let o = found(&f);
        assert_eq!((o.tool, o.shell), (Tool::Dms, None));

        // DMS with its layer off and awww drawing: awww shows it, and DMS may be told too so its colours follow.
        f.process(2, &["awww-daemon"], Some("wayland-1"));
        let o = found(&f);
        assert_eq!((o.tool, o.shell), (Tool::Awww, Some(Tool::Dms)));
    }

    #[test]
    fn a_namespaced_awww_daemon_is_a_backdrop_not_the_wallpaper() {
        let f = Fixture::new();
        f.process(1, &["awww-daemon", "-n", "backdrop"], Some("wayland-1"));
        f.process(2, &["swaybg", "-i", "/w.png"], Some("wayland-1"));
        assert_eq!(found(&f).tool, Tool::Swaybg);
    }

    #[test]
    fn omarchy_and_sway_own_the_swaybg_they_started() {
        let f = Fixture::new();
        f.process(
            1,
            &[
                "swaybg",
                "-i",
                "/h/.config/omarchy/current/background",
                "-m",
                "fill",
            ],
            Some("wayland-1"),
        );
        assert_eq!(found(&f).tool, Tool::OmarchyLegacy);

        let g = Fixture::new();
        g.process(5, &["sway"], Some("wayland-1"));
        g.process(
            6,
            &["swaybg", "-o", "*", "-i", "/w.png", "-m", "fit"],
            Some("wayland-1"),
        );
        g.stat(6, 5, 100);
        let o = found(&g);
        assert_eq!(o.tool, Tool::Sway);
        let w = g.wall("new \"one\".png");
        let p = plan(&g.env(nothing), &o, &all(&w, &["DP-1"])).unwrap();
        assert_eq!(
            argvs(&p),
            vec![strs(&[
                "swaymsg",
                &format!(
                    "output * bg \"{}\" fit",
                    w.display().to_string().replace('"', "\\\"")
                )
            ])],
            "sway re-parses the command, so the path is quoted and the user's mode is kept"
        );
    }

    #[test]
    fn a_desktop_environment_sets_its_own_background() {
        let f = Fixture::new();
        f.process(1, &["gnome-shell"], Some("wayland-1"));
        let mut env = f.env(nothing);
        env.current_desktop = Some("ubuntu:gnome".into());
        let o = owner(&env).unwrap();
        assert_eq!(o.tool, Tool::Gnome);
        let w = f.wall("a b.png");
        let p = plan(&env, &o, &all(&w, &["eDP-1"])).unwrap();
        let uri = format!("file://{}", w.display()).replace(' ', "%20");
        assert_eq!(
            argvs(&p),
            vec![
                strs(&[
                    "gsettings",
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri",
                    &uri
                ]),
                strs(&[
                    "gsettings",
                    "set",
                    "org.gnome.desktop.background",
                    "picture-uri-dark",
                    &uri
                ]),
            ],
            "GNOME 42+ shows picture-uri-dark in dark style, so both are set"
        );
        assert!(
            plan(&env, &o, &only(&w, &["eDP-1", "HDMI-A-1"], &["HDMI-A-1"])).is_err(),
            "GNOME has one background for every screen"
        );
    }

    #[test]
    fn waypaper_is_used_only_when_its_backend_is_what_draws() {
        let f = Fixture::new();
        f.process(1, &["awww-daemon"], Some("wayland-1"));
        f.write(
            "config/waypaper/config.ini",
            "[Settings]\nbackend = awww\npost_command = matugen image $wallpaper\n",
        );
        let mut env = f.env(nothing);
        env.which = |p| p == "waypaper";
        assert_eq!(
            owner(&env).unwrap().tool,
            Tool::Waypaper,
            "its post_command hooks keep working"
        );

        f.write(
            "config/waypaper/config.ini",
            "[Settings]\nbackend = swaybg\n",
        );
        assert_eq!(
            owner(&env).unwrap().tool,
            Tool::Awww,
            "waypaper would start swaybg and kill awww"
        );
    }

    #[test]
    fn dms_uses_set_for_every_screen_but_set_for_in_per_monitor_mode() {
        let f = Fixture::new();
        f.process(1, DMS, Some("wayland-1"));
        let env = f.env(nothing);
        let o = found(&f);
        let w = f.wall("w.png");
        let p = plan(&env, &o, &all(&w, &["DP-1", "DP-2"])).unwrap();
        assert_eq!(
            argvs(&p),
            vec![strs(&[
                "dms",
                "ipc",
                "call",
                "wallpaper",
                "set",
                w.to_str().unwrap()
            ])]
        );

        f.write(
            "state/DankMaterialShell/session.json",
            r#"{"perMonitorWallpaper":true,"perModeWallpaper":true}"#,
        );
        let p = plan(&env, &o, &all(&w, &["DP-1", "DP-2"])).unwrap();
        assert_eq!(
            argvs(&p),
            ["DP-1", "DP-2"].map(|o| strs(&[
                "dms",
                "ipc",
                "call",
                "wallpaper",
                "setFor",
                o,
                w.to_str().unwrap()
            ])),
            "DMS refuses `set` in per-monitor mode"
        );
        assert!(p.notes.iter().any(|n| n.contains("current mode")));

        let clip = f.wall("clip.mp4");
        assert!(
            plan(&env, &o, &all(&clip, &["DP-1"]))
                .unwrap_err()
                .contains("videos")
        );
    }

    #[test]
    fn one_screen_is_refused_by_owners_that_show_one_wallpaper_everywhere() {
        let f = Fixture::new();
        f.process(1, &["qs", "-c", "caelestia"], Some("wayland-1"));
        let w = f.wall("w.png");
        let err = plan(
            &f.env(nothing),
            &found(&f),
            &only(&w, &["DP-1", "DP-2"], &["DP-2"]),
        )
        .unwrap_err();
        assert!(err.contains("one wallpaper on every screen"), "{err}");
    }

    #[test]
    fn hyprpaper_gets_one_call_per_screen_and_no_commas() {
        let f = Fixture::new();
        f.process(1, &["hyprpaper"], Some("wayland-1"));
        let (env, o) = (f.env(nothing), found(&f));
        let w = f.wall("w.png");
        let p = plan(&env, &o, &all(&w, &["DP-1", "HDMI-A-1"])).unwrap();
        let path = w.to_str().unwrap();
        assert_eq!(
            argvs(&p),
            ["DP-1", "HDMI-A-1"].map(|m| strs(&[
                "hyprctl",
                "hyprpaper",
                "wallpaper",
                &format!("{m},{path}")
            ])),
            "an empty monitor is only hyprpaper's fallback, so every screen is named"
        );
        assert_eq!(p.fallback.len(), 3, "hyprpaper ≤0.7 needs preload first");
        let comma = f.wall("a,b.png");
        assert!(
            plan(&env, &o, &all(&comma, &["DP-1"]))
                .unwrap_err()
                .contains("comma")
        );
    }

    #[test]
    fn mpvpaper_with_a_socket_changes_file_without_a_restart() {
        let f = Fixture::new();
        f.process(
            1,
            &[
                "mpvpaper",
                "-o",
                "no-audio loop input-ipc-server=/tmp/mpv-DP-1",
                "DP-1",
                "/old.mp4",
            ],
            Some("wayland-1"),
        );
        let w = f.wall("say \"hi\".mp4");
        let p = plan(&f.env(nothing), &found(&f), &all(&w, &["DP-1"])).unwrap();
        let Step::Line { socket, line, .. } = &p.steps[0] else {
            panic!("{p:?}")
        };
        assert_eq!(socket, &PathBuf::from("/tmp/mpv-DP-1"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(line).unwrap()["command"][1],
            w.to_str().unwrap(),
            "the JSON protocol carries any path, quotes included"
        );
        assert_eq!(p.restarts, None);
    }

    #[test]
    fn swaybg_is_restarted_with_only_the_chosen_screens_changed_and_consent_is_asked() {
        let f = Fixture::new();
        f.process(
            7,
            &[
                "swaybg", "-o", "DP-1", "-i", "/a.png", "-m", "fit", "-o", "DP-2", "-i", "/b.png",
                "-m", "fill",
            ],
            Some("wayland-1"),
        );
        f.stat(7, 1, 4242);
        let (env, o) = (f.env(nothing), found(&f));
        let w = f.wall("new.png");
        let new = w.to_str().unwrap();
        let p = plan(
            &env,
            &o,
            &only(&w, &["DP-1", "DP-2", "HDMI-A-1"], &["DP-2", "HDMI-A-1"]),
        )
        .unwrap();
        let Step::Respawn(r) = &p.steps[0] else {
            panic!("{p:?}")
        };
        assert_eq!(
            r.argv,
            strs(&[
                "swaybg", "-o", "DP-1", "-i", "/a.png", "-m", "fit", "-o", "DP-2", "-i", new, "-m",
                "fill", "-o", "HDMI-A-1", "-i", new, "-m", "fit"
            ]),
            "the user's other screens and modes survive; a screen without a group gets one"
        );
        assert_eq!(
            (r.old_pid, r.old_start),
            (7, Some(4242)),
            "a reused pid is never killed"
        );
        assert_eq!(
            p.restarts,
            Some(Tool::Swaybg),
            "Darwan didn't start this swaybg"
        );
    }

    #[test]
    fn a_drawer_under_systemd_is_not_restarted_behind_its_back_but_darwans_own_is() {
        let f = Fixture::new();
        f.process(7, &["swaybg", "-i", "/a.png"], Some("wayland-1"));
        f.cgroup(7, "swaybg.service");
        let w = f.wall("w.png");
        let err = plan(&f.env(nothing), &found(&f), &all(&w, &["DP-1"])).unwrap_err();
        assert!(
            err.contains("swaybg.service"),
            "systemd would start the old one again: {err}"
        );

        f.cgroup(7, "darwan-wallpaper-swaybg.service");
        let p = plan(&f.env(nothing), &found(&f), &all(&w, &["DP-1"])).unwrap();
        assert_eq!(
            p.restarts, None,
            "Darwan started it, so no consent is needed"
        );
        let Step::Respawn(r) = &p.steps[0] else {
            panic!()
        };
        assert_eq!(
            r.old_unit.as_deref(),
            Some("darwan-wallpaper-swaybg.service")
        );
    }

    #[test]
    fn xfce_sets_the_images_it_already_keeps_for_the_chosen_screens() {
        fn xfconf(program: &str, _: &[&str]) -> Option<String> {
            (program == "xfconf-query").then(|| {
                "/backdrop/screen0/monitorDP-1/workspace0/last-image\n/backdrop/screen0/monitorDP-1/workspace0/image-style\n/backdrop/screen0/monitorHDMI-A-1/workspace0/last-image\n".into()
            })
        }
        let f = Fixture::new();
        f.process(1, &["xfdesktop"], Some("wayland-1"));
        let mut env = f.env(xfconf);
        env.current_desktop = Some("xfce".into());
        let o = owner(&env).unwrap();
        let w = f.wall("w.png");
        let p = plan(&env, &o, &only(&w, &["DP-1", "HDMI-A-1"], &["HDMI-A-1"])).unwrap();
        assert_eq!(
            argvs(&p),
            vec![strs(&[
                "xfconf-query",
                "-c",
                "xfce4-desktop",
                "-p",
                "/backdrop/screen0/monitorHDMI-A-1/workspace0/last-image",
                "-s",
                w.to_str().unwrap()
            ])]
        );
    }

    struct Fake {
        replies: RefCell<Vec<Result<String, String>>>,
        ran: RefCell<Vec<Vec<String>>>,
    }

    impl Runner for Fake {
        fn run(&self, argv: &[String]) -> Result<String, String> {
            self.ran.borrow_mut().push(argv.to_vec());
            self.replies.borrow_mut().remove(0)
        }
        fn line(&self, _: &Path, _: &str) -> Result<String, String> {
            self.replies.borrow_mut().remove(0)
        }
        fn respawn(&self, _: &Respawn) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn apply_stops_at_the_first_refusal_and_reports_the_tools_own_words() {
        let steps = vec![
            Step::Run {
                argv: strs(&["dms", "a"]),
                reply: Reply::Starts("SUCCESS"),
            },
            Step::Run {
                argv: strs(&["dms", "b"]),
                reply: Reply::Starts("SUCCESS"),
            },
            Step::Run {
                argv: strs(&["dms", "c"]),
                reply: Reply::Starts("SUCCESS"),
            },
        ];
        let fake = Fake {
            replies: RefCell::new(vec![
                Ok("SUCCESS: set".into()),
                Ok("ERROR: Wallpaper file not found\n".into()),
                Ok("SUCCESS".into()),
            ]),
            ran: RefCell::new(Vec::new()),
        };
        assert_eq!(
            apply(&steps, &fake),
            Err("ERROR: Wallpaper file not found".into())
        );
        assert_eq!(fake.ran.borrow().len(), 2);
    }

    #[test]
    fn the_read_back_is_per_screen_where_the_owner_knows_screens() {
        fn hyprctl(_: &str, _: &[&str]) -> Option<String> {
            Some("DP-1 = /etc/hostname\nHDMI-A-1 = /etc/passwd\n".into())
        }
        let f = Fixture::new();
        f.process(1, &["hyprpaper"], Some("wayland-1"));
        let env = f.env(hyprctl);
        let o = owner(&env).unwrap();
        assert_eq!(
            current(&env, &o, Some("HDMI-A-1")),
            Some(PathBuf::from("/etc/passwd"))
        );
        let req = only(Path::new("/etc/hostname"), &["DP-1", "HDMI-A-1"], &["DP-1"]);
        assert_eq!(shows(&env, &o, &req), Some(true));
        let req = only(
            Path::new("/etc/hostname"),
            &["DP-1", "HDMI-A-1"],
            &["DP-1", "HDMI-A-1"],
        );
        assert_eq!(shows(&env, &o, &req), Some(false));
    }
}
