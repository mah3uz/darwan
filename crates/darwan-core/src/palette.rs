use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use image::imageops::FilterType;
use material_colors::color::Argb;
use material_colors::dynamic_color::DynamicScheme;
use material_colors::hct::Hct;
use material_colors::image::{Image, ImageReader};
use material_colors::scheme::variant::{
    SchemeContent, SchemeExpressive, SchemeFidelity, SchemeFruitSalad, SchemeMonochrome,
    SchemeNeutral, SchemeRainbow, SchemeTonalSpot, SchemeVibrant,
};

use crate::custom::{MATERIAL_ROLES, Media, Palette, PaletteRequest};

// Bumped when the output for the same input changes, so stale cache files are ignored.
const CACHE_VERSION: u32 = 1;

// Mirrors matugen 4.2: a 112×112 Triangle-filtered thumbnail, Celebi quantisation, the top-scored colour.
fn source_color(rgba: image::RgbaImage) -> (Argb, f64) {
    let small = image::imageops::resize(&rgba, 112, 112, FilterType::Triangle);
    let luminance = small
        .pixels()
        .map(|p| (0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64) / 255.0)
        .sum::<f64>()
        / (112.0 * 112.0);
    (ImageReader::extract_color(&Image::new(small)), luminance)
}

fn scheme(name: &str, source: Argb, dark: bool, contrast: f64) -> DynamicScheme {
    let hct = Hct::new(source);
    let c = Some(contrast);
    match name {
        "scheme-vibrant" => SchemeVibrant::new(hct, dark, c).scheme,
        "scheme-expressive" => SchemeExpressive::new(hct, dark, c).scheme,
        "scheme-content" => SchemeContent::new(hct, dark, c).scheme,
        "scheme-fidelity" => SchemeFidelity::new(hct, dark, c).scheme,
        "scheme-monochrome" => SchemeMonochrome::new(hct, dark, c).scheme,
        "scheme-neutral" => SchemeNeutral::new(hct, dark, c).scheme,
        "scheme-rainbow" => SchemeRainbow::new(hct, dark, c).scheme,
        "scheme-fruit-salad" => SchemeFruitSalad::new(hct, dark, c).scheme,
        _ => SchemeTonalSpot::new(hct, dark, c).scheme,
    }
}

fn role(s: &DynamicScheme, name: &str) -> Argb {
    match name {
        "primary" => s.primary(),
        "on_primary" => s.on_primary(),
        "primary_container" => s.primary_container(),
        "on_primary_container" => s.on_primary_container(),
        "secondary" => s.secondary(),
        "on_secondary" => s.on_secondary(),
        "secondary_container" => s.secondary_container(),
        "on_secondary_container" => s.on_secondary_container(),
        "tertiary" => s.tertiary(),
        "on_tertiary" => s.on_tertiary(),
        "tertiary_container" => s.tertiary_container(),
        "on_tertiary_container" => s.on_tertiary_container(),
        "error" => s.error(),
        "on_error" => s.on_error(),
        "error_container" => s.error_container(),
        "on_error_container" => s.on_error_container(),
        "background" => s.background(),
        "on_background" => s.on_background(),
        "surface" => s.surface(),
        "on_surface" => s.on_surface(),
        "surface_variant" => s.surface_variant(),
        "on_surface_variant" => s.on_surface_variant(),
        "surface_dim" => s.surface_dim(),
        "surface_bright" => s.surface_bright(),
        "surface_container_lowest" => s.surface_container_lowest(),
        "surface_container_low" => s.surface_container_low(),
        "surface_container" => s.surface_container(),
        "surface_container_high" => s.surface_container_high(),
        "surface_container_highest" => s.surface_container_highest(),
        "outline" => s.outline(),
        "outline_variant" => s.outline_variant(),
        "inverse_surface" => s.inverse_surface(),
        "inverse_on_surface" => s.inverse_on_surface(),
        "inverse_primary" => s.inverse_primary(),
        "shadow" => s.shadow(),
        _ => s.scrim(),
    }
}

fn palette(source: Argb, dark: bool, request: &PaletteRequest) -> Palette {
    let s = scheme(&request.scheme, source, dark, request.contrast);
    MATERIAL_ROLES
        .iter()
        .map(|r| {
            (
                r.to_string(),
                role(&s, r).to_hex_with_pound().to_lowercase(),
            )
        })
        .collect()
}

pub fn from_rgba(rgba: image::RgbaImage, request: &PaletteRequest) -> Palette {
    let (source, luminance) = source_color(rgba);
    palette(source, request.dark.unwrap_or(luminance < 0.5), request)
}

// A single picked colour as the seed, as Android does; "#rrggbb", validated by the caller.
pub fn from_seed(hex: &str, request: &PaletteRequest) -> Option<Palette> {
    let n = u32::from_str_radix(hex.strip_prefix('#')?, 16).ok()?;
    let seed = Argb::new(255, (n >> 16) as u8, (n >> 8) as u8, n as u8);
    Some(palette(seed, request.dark.unwrap_or(false), request))
}

// A frame one second in (the first frames are often black), falling back to the first.
fn video_frame(video: &Path, out: &Path) -> Result<(), String> {
    for at in ["1", "0"] {
        let ok = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-ss", at, "-i"])
            .arg(video)
            .args(["-frames:v", "1", "-vf", "scale=256:-2"])
            .arg(out)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("cannot run ffmpeg: {e}"))?
            .success();
        if ok && out.is_file() {
            return Ok(());
        }
    }
    Err(format!(
        "ffmpeg could not read a frame from {}",
        video.display()
    ))
}

pub fn generate(path: &Path, request: &PaletteRequest) -> Result<Palette, String> {
    let rgba = if Media::of(path) == Some(Media::Video) {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let frame = dir.path().join("frame.png");
        video_frame(path, &frame)?;
        image::open(&frame)
    } else {
        image::open(path)
    }
    .map_err(|e| format!("{}: {e}", path.display()))?
    .into_rgba8();
    Ok(from_rgba(rgba, request))
}

fn cache_key(path: &Path, request: &PaletteRequest) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    CACHE_VERSION.hash(&mut h);
    path.canonicalize().ok()?.hash(&mut h);
    meta.len().hash(&mut h);
    meta.modified().ok()?.hash(&mut h);
    request.scheme.hash(&mut h);
    request.dark.hash(&mut h);
    request.contrast.to_bits().hash(&mut h);
    Some(format!("{:016x}", h.finish()))
}

// Keyed by the file's path, size and mtime and the request, so an edited image is regenerated.
pub fn cached(cache_dir: &Path, path: &Path, request: &PaletteRequest) -> Result<Palette, String> {
    let file: Option<PathBuf> =
        cache_key(path, request).map(|k| cache_dir.join(format!("{k}.json")));
    if let Some(p) = file
        .as_ref()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|t| serde_json::from_str::<Palette>(&t).ok())
    {
        return Ok(p);
    }
    let palette = generate(path, request)?;
    if let Some(f) = file {
        let _ = std::fs::create_dir_all(cache_dir);
        if let Ok(text) = serde_json::to_string(&palette) {
            let _ = std::fs::write(f, text);
        }
    }
    Ok(palette)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(scheme: &str, dark: Option<bool>) -> PaletteRequest {
        PaletteRequest {
            scheme: scheme.into(),
            dark,
            contrast: 0.0,
        }
    }

    // Two thirds warm orange, one third blue: the source colour must come from the orange.
    fn sunset() -> image::RgbaImage {
        image::RgbaImage::from_fn(300, 200, |x, _| {
            if x < 200 {
                image::Rgba([230, 120, 40, 255])
            } else {
                image::Rgba([40, 70, 160, 255])
            }
        })
    }

    fn hue(hex: &str) -> f64 {
        let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
        Hct::new(Argb::new(255, (n >> 16) as u8, (n >> 8) as u8, n as u8)).get_hue()
    }

    #[test]
    fn every_material_role_is_produced_as_lowercase_hex() {
        let p = from_rgba(sunset(), &request("scheme-tonal-spot", Some(true)));
        assert_eq!(p.len(), MATERIAL_ROLES.len());
        assert!(
            p.values()
                .all(|v| v.len() == 7 && v.starts_with('#') && v == &v.to_lowercase())
        );
    }

    #[test]
    fn the_dominant_colour_sets_the_hue_like_matugen() {
        let p = from_rgba(sunset(), &request("scheme-tonal-spot", Some(false)));
        let h = hue(&p["primary"]);
        assert!(
            (20.0..80.0).contains(&h),
            "primary hue {h} should be orange"
        );
    }

    #[test]
    fn dark_and_light_schemes_flip_surface_brightness() {
        let dark = from_rgba(sunset(), &request("scheme-tonal-spot", Some(true)));
        let light = from_rgba(sunset(), &request("scheme-tonal-spot", Some(false)));
        let lum = |hex: &str| u32::from_str_radix(&hex[1..], 16).unwrap() & 0xff;
        assert!(lum(&dark["surface"]) < lum(&light["surface"]));
        assert_ne!(dark["primary"], light["primary"]);
    }

    #[test]
    fn without_a_variant_the_images_brightness_decides() {
        let night = image::RgbaImage::from_pixel(64, 64, image::Rgba([10, 12, 30, 255]));
        let p = from_rgba(night, &request("scheme-tonal-spot", None));
        let lum = u32::from_str_radix(&p["surface"][1..], 16).unwrap() & 0xff;
        assert!(lum < 0x40, "a dark image gets a dark scheme");
    }

    #[test]
    fn schemes_differ_and_unknown_names_fall_back_to_tonal_spot() {
        let tonal = from_rgba(sunset(), &request("scheme-tonal-spot", Some(true)));
        let mono = from_rgba(sunset(), &request("scheme-monochrome", Some(true)));
        assert_ne!(tonal["primary"], mono["primary"]);
        assert_eq!(tonal, from_rgba(sunset(), &request("bogus", Some(true))));
    }

    #[test]
    fn the_cache_returns_the_same_palette_and_notices_a_changed_file() {
        let dir = tempfile::tempdir().unwrap();
        let img = dir.path().join("wall.png");
        sunset().save(&img).unwrap();
        let cache = dir.path().join("cache");
        let r = request("scheme-vibrant", Some(true));
        let first = cached(&cache, &img, &r).unwrap();
        assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 1);
        assert_eq!(cached(&cache, &img, &r).unwrap(), first);
        std::thread::sleep(std::time::Duration::from_millis(20));
        image::RgbaImage::from_pixel(50, 50, image::Rgba([30, 160, 60, 255]))
            .save(&img)
            .unwrap();
        assert_ne!(
            cached(&cache, &img, &r).unwrap(),
            first,
            "an edited wallpaper is regenerated"
        );
    }

    #[test]
    fn a_seed_colour_sets_the_palettes_hue_in_both_modes() {
        for dark in [false, true] {
            let p = from_seed("#e63946", &request("scheme-tonal-spot", Some(dark))).unwrap();
            assert_eq!(p.len(), MATERIAL_ROLES.len());
            let h = hue(&p["primary"]);
            assert!(
                !(40.0..340.0).contains(&h),
                "primary hue {h} should stay red (dark: {dark})"
            );
        }
        assert!(from_seed("red", &request("scheme-tonal-spot", None)).is_none());
    }

    #[test]
    fn a_broken_image_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.png");
        std::fs::write(&bad, b"not a png").unwrap();
        assert!(generate(&bad, &request("scheme-tonal-spot", None)).is_err());
    }
}
