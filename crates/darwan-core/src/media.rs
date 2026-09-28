use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use crate::config::{Target, UserConfig};
use crate::paths;
use crate::settings::Key;

// A change that can alter what the lock and saver play: the lock theme, its background, or the quality.
pub fn affects(key: &Key, config: &UserConfig) -> bool {
    match key {
        Key::Theme(Target::Lock) | Key::SaverQuality => true,
        Key::Option { theme, key } => {
            key.starts_with("background")
                && config.theme(Target::Lock).ok().flatten() == Some(theme)
        }
        _ => false,
    }
}

// Eco copies are made when a theme or background is picked, never while the machine idles; `darwan` is the binary
// that makes them (the GUI passes the one next to it).
pub fn spawn_prepare(darwan: &Path) {
    let log = std::fs::create_dir_all(paths::state_dir())
        .and_then(|()| std::fs::File::create(paths::state_dir().join("media.log")));
    let mut cmd = Command::new(darwan);
    cmd.arg("prepare-media").stdin(std::process::Stdio::null());
    if let Ok(log) = log {
        if let Ok(err) = log.try_clone() {
            cmd.stderr(err);
        }
        cmd.stdout(log);
    }
    let _ = cmd.spawn();
}

// The eco copy of `source` in ~/.cache/darwan/media. The key must match MpvVideo::ecoPath in plugin/mpvvideo.cpp:
// FNV-1a 64 over "<canonical path>\0<size>\0<mtime seconds>", so an edited file gets a new copy.
pub fn eco_path(source: &Path) -> Option<PathBuf> {
    let real = std::fs::canonicalize(source).ok()?;
    let meta = std::fs::metadata(&real).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    let mut key = real.as_os_str().as_encoded_bytes().to_vec();
    key.push(0);
    key.extend(meta.len().to_string().as_bytes());
    key.push(0);
    key.extend(mtime.to_string().as_bytes());
    Some(
        paths::cache_dir()
            .join("media")
            .join(format!("{:016x}.mp4", fnv1a(&key))),
    )
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

// Animated images always get a video copy: mpv then decodes them on the GPU instead of every frame on the CPU.
pub fn is_animated_image(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "gif" | "webp" | "apng" | "png"
        )
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub codec: String,
    pub height: u32,
    pub fps: f64,
}

// `ffprobe -show_entries stream=codec_name,height,avg_frame_rate -of csv=p=0` for the first video stream.
pub fn parse_probe(line: &str) -> Option<Stream> {
    let mut parts = line.trim().split(',');
    let codec = parts.next()?.to_string();
    let height = parts.next()?.parse().ok()?;
    let fps = match parts.next()?.split_once('/') {
        Some((n, d)) => {
            let (n, d): (f64, f64) = (n.parse().ok()?, d.parse().ok()?);
            if d == 0.0 { 0.0 } else { n / d }
        }
        None => 0.0,
    };
    Some(Stream { codec, height, fps })
}

// 1080p H.264 at 30 fps or less is what every hardware decoder handles and no screen needs more for a background.
pub fn needs_eco(stream: &Stream) -> bool {
    stream.codec != "h264" || stream.height > 1080 || stream.fps > 30.5
}

pub fn probe(source: &Path) -> Option<Stream> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,height,avg_frame_rate",
            "-of",
            "csv=p=0",
        ])
        .arg(source)
        .output()
        .ok()?;
    parse_probe(&String::from_utf8_lossy(&out.stdout))
}

// The ffmpeg arguments for an eco copy: no audio, at most 1080 lines and 30 fps, H.264 High that starts playing at once.
pub fn transcode_args(source: &Path, stream: Option<&Stream>, target: &Path) -> Vec<String> {
    let fps = stream
        .map(|s| s.fps)
        .filter(|f| *f > 0.0)
        .map_or(30.0, |f| f.min(30.0));
    vec![
        "-nostdin".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-i".into(),
        source.display().to_string(),
        "-an".into(),
        "-vf".into(),
        format!("scale=-2:'min(1080,ih)':flags=lanczos,fps={fps:.3},format=yuv420p"),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "slow".into(),
        "-crf".into(),
        "20".into(),
        "-profile:v".into(),
        "high".into(),
        "-movflags".into(),
        "+faststart".into(),
        "-f".into(),
        "mp4".into(),
        target.display().to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_lock_theme_background_quality_changes_prepare_media() {
        let cfg = UserConfig::parse("[lock]\ntheme = \"forest\"\n").unwrap();
        let key = |s| Key::parse(s).unwrap();
        assert!(affects(&key("lock.theme"), &cfg));
        assert!(affects(&key("saver.quality"), &cfg));
        assert!(affects(&key("forest.background"), &cfg));
        assert!(!affects(&key("osu.background"), &cfg), "not the lock theme");
        assert!(!affects(&key("forest.accent"), &cfg));
        assert!(!affects(&key("sddm.theme"), &cfg));
    }

    #[test]
    fn fnv1a_matches_the_reference_vectors() {
        // The plugin computes the same hash in C++; both must agree with the published FNV-1a 64 values.
        assert_eq!(fnv1a(b""), 0xcbf29ce484222325);
        assert_eq!(fnv1a(b"a"), 0xaf63dc4c8601ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn eco_copies_change_when_the_file_changes() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("bg.mp4");
        std::fs::write(&f, "one").unwrap();
        let a = eco_path(&f).unwrap();
        std::fs::write(&f, "longer").unwrap();
        assert_ne!(eco_path(&f).unwrap(), a, "a new size means a new copy");
        assert!(eco_path(&dir.path().join("missing.mp4")).is_none());
    }

    #[test]
    fn only_what_some_decoder_or_screen_would_struggle_with_is_transcoded() {
        let s = |codec: &str, height, fps| Stream {
            codec: codec.into(),
            height,
            fps,
        };
        assert!(!needs_eco(&s("h264", 1080, 30.0)));
        assert!(!needs_eco(&s("h264", 720, 29.97)));
        assert!(needs_eco(&s("h264", 2160, 60.0)));
        assert!(needs_eco(&s("h264", 1080, 60.0)));
        assert!(
            needs_eco(&s("hevc", 1080, 24.0)),
            "not every decoder has HEVC"
        );
        assert_eq!(parse_probe("h264,2160,60/1\n"), Some(s("h264", 2160, 60.0)));
        assert_eq!(parse_probe("gif,480,100/3\n").unwrap().height, 480);
        assert_eq!(parse_probe(""), None);
    }

    #[test]
    fn a_transcode_never_raises_the_frame_rate() {
        let args = |fps| {
            transcode_args(
                Path::new("/a.mp4"),
                Some(&Stream {
                    codec: "h264".into(),
                    height: 2160,
                    fps,
                }),
                Path::new("/b.mp4"),
            )
            .join(" ")
        };
        assert!(args(60.0).contains("fps=30.000"));
        assert!(args(24.0).contains("fps=24.000"));
        assert!(args(60.0).contains("-an"), "backgrounds are silent");
    }
}
