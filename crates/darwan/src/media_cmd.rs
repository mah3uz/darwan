use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use darwan_core::catalog::Catalog;
use darwan_core::config::{Target, UserConfig};
use darwan_core::hardware::{self, Tier};
use darwan_core::media;
use darwan_core::paths::{self, Paths};
use darwan_core::saver::Quality;
use rustix::fs::{FlockOperation, flock};

const VIDEO: &[&str] = &["mp4", "webm", "mkv", "mov"];

// Eco copies of what the lock theme plays, made ahead of time at the lowest priority.
pub fn run(paths: &Paths) -> Result<ExitCode, String> {
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let Some(id) = config.theme(Target::Lock).ok().flatten() else {
        return Ok(ExitCode::SUCCESS);
    };
    let (catalog, _) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let theme = catalog
        .get(id)
        .ok_or_else(|| darwan_core::catalog::unknown_theme(id))?;

    let dir = paths::cache_dir().join("media");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let lock = File::create(dir.join(".lock")).map_err(|e| e.to_string())?;
    if flock(&lock, FlockOperation::NonBlockingLockExclusive).is_err() {
        println!("Another darwan is already preparing media.");
        return Ok(ExitCode::SUCCESS);
    }

    let resolved =
        darwan_core::resolve::resolve(theme, &config, &darwan_core::system::SystemHost::new());
    let user_bg = match resolved.overlay.get("backgroundType").map(String::as_str) {
        Some("video" | "animated") => resolved.overlay.get("backgroundPath").map(PathBuf::from),
        _ => None,
    };
    let quality = config
        .saver_quality()
        .ok()
        .flatten()
        .unwrap_or(Quality::Auto);
    let eco = could_use_eco(quality);

    let mut sources: Vec<PathBuf> = Vec::new();
    if eco {
        sources.extend(theme_videos(&theme.dir));
    }
    sources.extend(user_bg);
    for source in sources {
        prepare(&source, eco);
    }
    Ok(ExitCode::SUCCESS)
}

// A laptop on mains power plays full videos but switches to eco on battery, so it needs the copies too.
fn could_use_eco(quality: Quality) -> bool {
    let mut facts = hardware::probe();
    facts.on_battery = Path::new("/sys/class/power_supply")
        .read_dir()
        .map(|d| {
            d.filter_map(Result::ok)
                .any(|e| e.file_name().to_string_lossy().starts_with("BAT"))
        })
        .unwrap_or(false);
    hardware::tier(quality, &facts).0 == Tier::Eco
}

fn theme_videos(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for path in entries.filter_map(Result::ok).map(|e| e.path()) {
        if path.is_dir() {
            out.extend(theme_videos(&path));
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| VIDEO.contains(&e))
        {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn prepare(source: &Path, eco: bool) {
    let animated = media::is_animated_image(source);
    let Some(target) = media::eco_path(source) else {
        return;
    };
    if target.exists() {
        return;
    }
    let stream = media::probe(source);
    if !animated && !(eco && stream.as_ref().is_none_or(media::needs_eco)) {
        return;
    }
    let partial = target.with_extension("partial.mp4");
    println!("{} -> {}", source.display(), target.display());
    let ok = Command::new("nice")
        .args(["-n", "19", "ffmpeg"])
        .args(media::transcode_args(source, stream.as_ref(), &partial))
        .stdin(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if ok {
        let _ = std::fs::rename(&partial, &target);
    } else {
        let _ = std::fs::remove_file(&partial);
        eprintln!("could not transcode {}", source.display());
    }
}
