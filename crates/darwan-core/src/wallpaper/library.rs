use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::UNIX_EPOCH;

use md5::{Digest, Md5};

use crate::palette::{self, Summary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Image,
    Animated,
    Video,
}

const IMAGES: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];
const VIDEOS: &[&str] = &["mp4", "mkv", "webm", "mov", "m4v"];

impl Kind {
    pub fn of(path: &Path) -> Option<Kind> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        if ext == "gif" {
            Some(Kind::Animated)
        } else if IMAGES.contains(&ext.as_str()) {
            Some(Kind::Image)
        } else if VIDEOS.contains(&ext.as_str()) {
            Some(Kind::Video)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Item {
    pub path: PathBuf,
    pub kind: Kind,
    pub bytes: u64,
    pub modified: u64,
}

// The Pictures folder is localised on many systems (~/Bilder, ~/Imágenes), so it comes from user-dirs.dirs.
pub fn pictures_dir(home: &Path, config_home: &Path) -> PathBuf {
    let text = std::fs::read_to_string(config_home.join("user-dirs.dirs")).unwrap_or_default();
    text.lines()
        .find_map(|l| l.trim().strip_prefix("XDG_PICTURES_DIR="))
        .map(|v| v.trim().trim_matches('"'))
        .filter(|v| !v.is_empty())
        .map(|v| match v.strip_prefix("$HOME") {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(v),
        })
        .unwrap_or_else(|| home.join("Pictures"))
}

// `Wallpapers` or `wallpapers` in the Pictures folder, whichever exists; `Wallpapers` when neither does yet.
pub fn default_folder(home: &Path, config_home: &Path) -> PathBuf {
    let pictures = pictures_dir(home, config_home);
    ["Wallpapers", "wallpapers"]
        .into_iter()
        .map(|n| pictures.join(n))
        .find(|p| p.is_dir())
        .unwrap_or_else(|| pictures.join("Wallpapers"))
}

// wallpaper.folder when set, else the default.
pub fn folder(config: &crate::config::UserConfig, home: &Path, config_home: &Path) -> PathBuf {
    match config.wallpaper_folder().ok().flatten() {
        Some(v) => expand(v, home),
        None => default_folder(home, config_home),
    }
}

// `~/…` as the user writes it in config.toml.
pub fn expand(value: &str, home: &Path) -> PathBuf {
    match value.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None if value == "~" => home.to_path_buf(),
        None => PathBuf::from(value),
    }
}

fn seconds(t: std::io::Result<std::time::SystemTime>) -> u64 {
    t.ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

// One file as the scan would list it, for a file set from elsewhere.
pub fn item(path: &Path) -> Option<Item> {
    let meta = std::fs::metadata(path).ok()?;
    Some(Item {
        path: path.to_path_buf(),
        kind: Kind::of(path)?,
        bytes: meta.len(),
        modified: seconds(meta.modified()),
    })
}

// Every image and video in the folder and its subfolders (three levels, hidden ones skipped), newest first.
pub fn scan(folder: &Path) -> Vec<Item> {
    let mut out = Vec::new();
    walk(folder, 0, &mut out);
    out.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.path.cmp(&b.path))
    });
    out
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<Item>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        if e.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let path = e.path();
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.is_dir() {
            if depth < 3 {
                walk(&path, depth + 1, out);
            }
        } else if let Some(kind) = Kind::of(&path) {
            out.push(Item {
                path,
                kind,
                bytes: meta.len(),
                modified: seconds(meta.modified()),
            });
        }
    }
}

// As GLib's g_filename_to_uri writes it, so the thumbnail name (the MD5 of this URI) is the one file managers use.
pub fn file_uri(path: &Path) -> String {
    const KEEP: &[u8] = b"-._~/!$&'()*+,=:@";
    let mut out = String::from("file://");
    for b in path.as_os_str().as_encoded_bytes() {
        if b.is_ascii_alphanumeric() || KEEP.contains(b) {
            out.push(*b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Large,
    XLarge,
    XxLarge,
}

impl Size {
    fn dir(self) -> &'static str {
        match self {
            Size::Large => "large",
            Size::XLarge => "x-large",
            Size::XxLarge => "xx-large",
        }
    }

    fn pixels(self) -> u32 {
        match self {
            Size::Large => 256,
            Size::XLarge => 512,
            Size::XxLarge => 1024,
        }
    }
}

// The freedesktop thumbnail spec's location: $XDG_CACHE_HOME/thumbnails/<size>/<md5 of the URI>.png.
pub fn thumbnail_path(cache_home: &Path, file: &Path, size: Size) -> PathBuf {
    let digest = Md5::digest(file_uri(file).as_bytes());
    let name: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    cache_home
        .join("thumbnails")
        .join(size.dir())
        .join(format!("{name}.png"))
}

// A thumbnail counts only for this file as it is now: the spec's Thumb::URI and Thumb::MTime must match.
fn valid(thumb: &Path, uri: &str, mtime: u64) -> bool {
    let Ok(file) = std::fs::File::open(thumb) else {
        return false;
    };
    let Ok(reader) = png::Decoder::new(std::io::BufReader::new(file)).read_info() else {
        return false;
    };
    let text = &reader.info().uncompressed_latin1_text;
    let tag = |k: &str| {
        text.iter()
            .find(|t| t.keyword == k)
            .map(|t| t.text.as_str())
    };
    tag("Thumb::URI") == Some(uri) && tag("Thumb::MTime") == Some(mtime.to_string().as_str())
}

// The thumbnail for `item`, made if it's missing or stale. `file` is the absolute path the URI is made from.
pub fn thumbnail(cache_home: &Path, item: &Item, size: Size) -> Result<PathBuf, String> {
    let file =
        std::fs::canonicalize(&item.path).map_err(|e| format!("{}: {e}", item.path.display()))?;
    let uri = file_uri(&file);
    let thumb = thumbnail_path(cache_home, &file, size);
    if valid(&thumb, &uri, item.modified) {
        return Ok(thumb);
    }
    let (image, width, height) = decode(&file, item.kind, size.pixels())?;
    let small = image.thumbnail(size.pixels(), size.pixels()).into_rgba8();
    write_thumbnail(
        &thumb,
        &small,
        &[
            ("Thumb::URI", uri),
            ("Thumb::MTime", item.modified.to_string()),
            ("Thumb::Size", item.bytes.to_string()),
            ("Thumb::Image::Width", width.to_string()),
            ("Thumb::Image::Height", height.to_string()),
            ("Software", "Darwan".into()),
        ],
    )?;
    Ok(thumb)
}

// The picture and its full size; a video gives a frame one second in, as the palette code takes it.
fn decode(file: &Path, kind: Kind, pixels: u32) -> Result<(image::DynamicImage, u32, u32), String> {
    let at = |e: image::ImageError| format!("{}: {e}", file.display());
    if kind != Kind::Video {
        let (w, h) = image::image_dimensions(file).map_err(at)?;
        return Ok((image::open(file).map_err(at)?, w, h));
    }
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let frame = dir.path().join("frame.png");
    for seek in ["1", "0"] {
        let ok = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-ss", seek, "-i"])
            .arg(file)
            .args(["-frames:v", "1", "-vf", &format!("scale={pixels}:-2")])
            .arg(&frame)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("cannot run ffmpeg: {e}"))?
            .success();
        if ok && frame.is_file() {
            let (w, h) = video_size(file).unwrap_or((0, 0));
            return Ok((image::open(&frame).map_err(at)?, w, h));
        }
    }
    Err(format!(
        "ffmpeg could not read a frame from {}",
        file.display()
    ))
}

fn video_size(file: &Path) -> Option<(u32, u32)> {
    let out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
        ])
        .arg(file)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (w, h) = text.trim().split_once(',')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

// Written beside the target and renamed into place, owner-only, as the spec asks.
fn write_thumbnail(
    thumb: &Path,
    rgba: &image::RgbaImage,
    tags: &[(&str, String)],
) -> Result<(), String> {
    let dir = thumb.parent().ok_or("no thumbnail folder")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    {
        let mut enc = png::Encoder::new(
            std::io::BufWriter::new(tmp.as_file()),
            rgba.width(),
            rgba.height(),
        );
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        for (k, v) in tags {
            enc.add_text_chunk(k.to_string(), v.clone())
                .map_err(|e| e.to_string())?;
        }
        let mut w = enc.write_header().map_err(|e| e.to_string())?;
        w.write_image_data(rgba.as_raw())
            .map_err(|e| e.to_string())?;
    }
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    tmp.persist(thumb).map_err(|e| e.to_string())?;
    Ok(())
}

// Colour groups for the Library's filter, as ranges of HCT hue (not HSV: pure blue is 283°, pure red 27°), with the
// boundaries halfway between measured reference colours. Low chroma is black, grey or white by tone.
pub const COLOURS: &[(&str, f64, f64)] = &[
    ("red", 15.0, 42.0),
    ("orange", 42.0, 90.0),
    ("yellow", 90.0, 125.0),
    ("green", 125.0, 175.0),
    ("teal", 175.0, 235.0),
    ("blue", 235.0, 295.0),
    ("purple", 295.0, 340.0),
    ("pink", 340.0, 15.0),
];

// material-colors' scorer falls back to Google blue when no colour in the image qualifies (a black or grey picture).
const NO_COLOUR: &str = "#4285f4";

pub fn colour_group(s: &Summary) -> &'static str {
    if s.chroma < 12.0 || s.source == NO_COLOUR {
        return if s.luminance < 0.2 {
            "black"
        } else if s.luminance > 0.8 {
            "white"
        } else {
            "grey"
        };
    }
    COLOURS
        .iter()
        .find(|(_, from, to)| {
            if from < to {
                s.hue >= *from && s.hue < *to
            } else {
                s.hue >= *from || s.hue < *to
            }
        })
        .map_or("grey", |(name, _, _)| name)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Facts {
    pub width: u32,
    pub height: u32,
    pub colour: String,
    pub summary: Summary,
}

// Bumped when Facts or the way they're made changes, so old cache files are ignored.
const FACTS_VERSION: u32 = 1;

fn facts_file(cache_dir: &Path, item: &Item) -> PathBuf {
    let mut key = format!("{FACTS_VERSION}\0").into_bytes();
    key.extend(item.path.as_os_str().as_encoded_bytes());
    key.extend(format!("\0{}\0{}", item.bytes, item.modified).into_bytes());
    cache_dir.join(format!("{:016x}.json", crate::media::fnv1a(&key)))
}

// Size, colours and colour group, cached per file (path, size and mtime), made from its thumbnail: the source colour
// comes from a 112-pixel copy anyway, so decoding the full picture again would change nothing.
pub fn facts(cache_dir: &Path, item: &Item, thumb: &Path) -> Result<Facts, String> {
    let file = facts_file(cache_dir, item);
    if let Some(f) = std::fs::read_to_string(&file)
        .ok()
        .and_then(|t| serde_json::from_str::<Facts>(&t).ok())
    {
        return Ok(f);
    }
    let decoder = png::Decoder::new(std::io::BufReader::new(
        std::fs::File::open(thumb).map_err(|e| format!("{}: {e}", thumb.display()))?,
    ));
    let reader = decoder.read_info().map_err(|e| e.to_string())?;
    let size = |k: &str| {
        reader
            .info()
            .uncompressed_latin1_text
            .iter()
            .find(|t| t.keyword == k)
            .and_then(|t| t.text.parse().ok())
            .unwrap_or(0)
    };
    let (width, height) = (size("Thumb::Image::Width"), size("Thumb::Image::Height"));
    let rgba = image::open(thumb).map_err(|e| e.to_string())?.into_rgba8();
    let summary = palette::summary(rgba);
    let facts = Facts {
        width,
        height,
        colour: colour_group(&summary).to_string(),
        summary,
    };
    let _ = std::fs::create_dir_all(cache_dir);
    if let Ok(text) = serde_json::to_string(&facts) {
        let _ = std::fs::write(file, text);
    }
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uri_is_escaped_exactly_as_glib_does_so_file_managers_share_the_thumbnail() {
        // GLib.filename_to_uri on this machine (Python gi), for the characters that differ between URI encoders.
        assert_eq!(
            file_uri(Path::new("/tmp/a b(c)&é,#%?[x]~!$'*+;=:@.png")),
            "file:///tmp/a%20b(c)&%C3%A9,%23%25%3F%5Bx%5D~!$'*+%3B=:@.png"
        );
        // A GNOME-made thumbnail's name on this machine is the MD5 of its URI.
        assert!(
            thumbnail_path(
                Path::new("/c"),
                Path::new("/home/mahfuz/MacTahoe-gtk-theme/screenshot01.jpeg"),
                Size::Large
            )
            .ends_with("thumbnails/large/17e85018ba45cef360ec3b2235638c75.png")
        );
    }

    #[test]
    fn the_folder_follows_the_localised_pictures_folder_and_either_spelling() {
        let d = tempfile::tempdir().unwrap();
        let (home, config) = (d.path().join("home"), d.path().join("config"));
        std::fs::create_dir_all(&config).unwrap();
        assert_eq!(
            default_folder(&home, &config),
            home.join("Pictures/Wallpapers")
        );
        std::fs::write(
            config.join("user-dirs.dirs"),
            "XDG_PICTURES_DIR=\"$HOME/Bilder\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(home.join("Bilder/wallpapers")).unwrap();
        assert_eq!(
            default_folder(&home, &config),
            home.join("Bilder/wallpapers")
        );
        assert_eq!(expand("~/Walls", &home), home.join("Walls"));
    }

    #[test]
    fn the_scan_finds_pictures_and_videos_in_subfolders_but_not_hidden_or_other_files() {
        let d = tempfile::tempdir().unwrap();
        let f = |p: &str| {
            let path = d.path().join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"x").unwrap();
        };
        for p in [
            "a.JPG",
            "b.png",
            "loop.mp4",
            "spin.gif",
            "LICENSE",
            "README.md",
            "nature/c.webp",
            ".trash/d.png",
            "a/b/c/d/deep.png",
        ] {
            f(p);
        }
        let mut found: Vec<String> = scan(d.path())
            .iter()
            .map(|i| i.path.strip_prefix(d.path()).unwrap().display().to_string())
            .collect();
        found.sort();
        assert_eq!(
            found,
            ["a.JPG", "b.png", "loop.mp4", "nature/c.webp", "spin.gif"]
        );
        assert_eq!(Kind::of(Path::new("x.gif")), Some(Kind::Animated));
    }

    fn picture(dir: &Path, name: &str, rgb: [u8; 3]) -> Item {
        let path = dir.join(name);
        image::RgbImage::from_pixel(1920, 1080, image::Rgb(rgb))
            .save(&path)
            .unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        Item {
            path,
            kind: Kind::Image,
            bytes: meta.len(),
            modified: seconds(meta.modified()),
        }
    }

    #[test]
    fn a_thumbnail_follows_the_spec_and_is_remade_only_when_the_file_changes() {
        let d = tempfile::tempdir().unwrap();
        let cache = d.path().join("cache");
        let item = picture(d.path(), "blue.png", [20, 60, 200]);
        let thumb = thumbnail(&cache, &item, Size::XLarge).unwrap();
        let (w, h) = image::image_dimensions(&thumb).unwrap();
        assert_eq!(
            (w, h),
            (512, 288),
            "fits 512 on its long side, keeping the aspect"
        );
        let uri = file_uri(&std::fs::canonicalize(&item.path).unwrap());
        assert!(valid(&thumb, &uri, item.modified));
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&thumb).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let made = std::fs::metadata(&thumb).unwrap().modified().unwrap();
        thumbnail(&cache, &item, Size::XLarge).unwrap();
        assert_eq!(
            std::fs::metadata(&thumb).unwrap().modified().unwrap(),
            made,
            "a valid one is reused"
        );
        assert!(
            !valid(&thumb, &uri, item.modified + 1),
            "a changed file makes it stale"
        );
    }

    #[test]
    fn facts_give_the_size_and_a_colour_group_and_are_cached() {
        let d = tempfile::tempdir().unwrap();
        let cache = d.path().join("cache");
        for (name, rgb, group) in [
            ("b.png", [20, 60, 200], "blue"),
            ("n.png", [10, 10, 10], "black"),
            ("g.png", [40, 160, 60], "green"),
            ("o.png", [255, 128, 0], "orange"),
            ("p.png", [255, 96, 176], "pink"),
            ("t.png", [0, 160, 160], "teal"),
            ("w.png", [245, 245, 245], "white"),
            ("m.png", [128, 128, 128], "grey"),
        ] {
            let item = picture(d.path(), name, rgb);
            let thumb = thumbnail(&cache, &item, Size::Large).unwrap();
            let f = facts(&cache.join("facts"), &item, &thumb).unwrap();
            assert_eq!(
                (f.width, f.height),
                (1920, 1080),
                "the original's size, from the thumbnail's tags"
            );
            assert_eq!(f.colour, group, "{name}: {:?}", f.summary);
            assert_eq!(f.summary.swatches.len(), 4);
            std::fs::remove_file(&thumb).unwrap();
            let again = facts(&cache.join("facts"), &item, &thumb)
                .expect("read from the cache, not the deleted thumbnail");
            assert_eq!(
                (again.colour, again.summary.swatches),
                (f.colour, f.summary.swatches)
            );
        }
    }
}
