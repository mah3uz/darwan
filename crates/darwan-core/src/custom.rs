use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::catalog::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Background,
    Appearance,
    Colours,
    Fonts,
    Motion,
}

impl Area {
    pub fn title(self) -> &'static str {
        match self {
            Area::Background => "Background",
            Area::Appearance => "Appearance",
            Area::Colours => "Colours",
            Area::Fonts => "Fonts",
            Area::Motion => "Motion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Background,
    Fit,
    Percent,
    Variant,
    Color,
    ColorSource,
    Scheme,
    Contrast,
    Font,
    Speed,
    Curve,
    Bool,
}

#[derive(Debug)]
pub struct Setting {
    pub key: &'static str,
    pub label: &'static str,
    pub area: Area,
    pub kind: Kind,
}

const fn s(key: &'static str, label: &'static str, area: Area, kind: Kind) -> Setting {
    Setting {
        key,
        label,
        area,
        kind,
    }
}

// The same keys and checks for every theme; each theme opts in through [supports] in darwan.toml.
pub const SETTINGS: &[Setting] = &[
    s(
        "background",
        "Custom background",
        Area::Background,
        Kind::Background,
    ),
    s("background_fit", "Fit", Area::Background, Kind::Fit),
    s("background_dim", "Dim", Area::Background, Kind::Percent),
    s("variant", "Light or dark", Area::Appearance, Kind::Variant),
    s("accent", "Accent colour", Area::Colours, Kind::Color),
    s("text_color", "Text colour", Area::Colours, Kind::Color),
    s(
        "color_source",
        "Generate colours from",
        Area::Colours,
        Kind::ColorSource,
    ),
    s("color_scheme", "Colour scheme", Area::Colours, Kind::Scheme),
    s("color_contrast", "Contrast", Area::Colours, Kind::Contrast),
    s("font_text", "Text font", Area::Fonts, Kind::Font),
    s("font_clock", "Clock font", Area::Fonts, Kind::Font),
    s("motion_speed", "Animation speed", Area::Motion, Kind::Speed),
    s("motion_curve", "Animation curve", Area::Motion, Kind::Curve),
    s("reduce_motion", "Reduce motion", Area::Motion, Kind::Bool),
];

pub fn setting(key: &str) -> Option<&'static Setting> {
    SETTINGS.iter().find(|s| s.key == key)
}

// matugen's scheme names, so a desktop themed with matugen and a lock screen generated here match.
pub const SCHEMES: &[(&str, &str)] = &[
    ("scheme-tonal-spot", "Tonal spot"),
    ("scheme-vibrant", "Vibrant"),
    ("scheme-expressive", "Expressive"),
    ("scheme-content", "Content"),
    ("scheme-fidelity", "Fidelity"),
    ("scheme-monochrome", "Monochrome"),
    ("scheme-neutral", "Neutral"),
    ("scheme-rainbow", "Rainbow"),
    ("scheme-fruit-salad", "Fruit salad"),
];

// value, label, Qt Easing name
pub const CURVES: &[(&str, &str, &str)] = &[
    ("smooth", "Smooth", "InOutCubic"),
    ("snappy", "Snappy", "OutExpo"),
    ("bouncy", "Bouncy", "OutBack"),
    ("linear", "Linear", "Linear"),
];

pub const IMAGE_EXT: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];
pub const ANIMATED_EXT: &[&str] = &["gif"];
pub const VIDEO_EXT: &[&str] = &["mp4", "mkv", "webm", "mov"];
pub const FONT_EXT: &[&str] = &["ttf", "otf"];

// The Material colour roles a generated palette provides (Material Color Utilities' dynamic scheme).
pub const MATERIAL_ROLES: &[&str] = &[
    "primary",
    "on_primary",
    "primary_container",
    "on_primary_container",
    "secondary",
    "on_secondary",
    "secondary_container",
    "on_secondary_container",
    "tertiary",
    "on_tertiary",
    "tertiary_container",
    "on_tertiary_container",
    "error",
    "on_error",
    "error_container",
    "on_error_container",
    "background",
    "on_background",
    "surface",
    "on_surface",
    "surface_variant",
    "on_surface_variant",
    "surface_dim",
    "surface_bright",
    "surface_container_lowest",
    "surface_container_low",
    "surface_container",
    "surface_container_high",
    "surface_container_highest",
    "outline",
    "outline_variant",
    "inverse_surface",
    "inverse_on_surface",
    "inverse_primary",
    "shadow",
    "scrim",
];

pub const GENERATE: &str = "generate";
pub const DESKTOP: &str = "desktop";

// The theme contract keys this module writes, and only these.
pub const CONTRACT_KEYS: &[&str] = &[
    "backgroundType",
    "backgroundPath",
    "backgroundColor",
    "backgroundFit",
    "backgroundDim",
    "colorScheme",
    "colorAccent",
    "colorText",
    "fontText",
    "fontTextFile",
    "fontClock",
    "fontClockFile",
    "animSpeed",
    "animEasing",
    "reduceMotion",
];

// Why a theme can't use a setting, or None when it can.
pub fn unsupported(theme: &Theme, key: &str) -> Option<String> {
    let m = &theme.manifest;
    let sup = &m.supports;
    let name = &m.name;
    let why = |yes: bool, msg: String| (!yes).then_some(msg);
    let Some(setting) = setting(key) else {
        return why(
            m.color(key).is_some(),
            format!("{name} has no colour {key:?}"),
        );
    };
    match setting.area {
        Area::Background => why(
            sup.background,
            format!("{name}'s background is part of its design"),
        ),
        Area::Appearance => why(
            !sup.variants.is_empty(),
            format!("{name} has one look: its artwork is the design"),
        ),
        Area::Colours => why(
            sup.colors,
            format!("{name} doesn't support changing its colours"),
        ),
        Area::Fonts => {
            let role = key.trim_start_matches("font_");
            why(
                sup.fonts.iter().any(|f| f == role),
                format!("{name} doesn't support changing its {role} font"),
            )
        }
        Area::Motion => why(
            sup.motion,
            format!("{name} doesn't support changing its animations"),
        ),
    }
}

pub fn expand(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(path),
    }
}

fn ext(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Media {
    Image,
    Animated,
    Video,
}

impl Media {
    pub fn of(path: &Path) -> Option<Self> {
        let e = ext(path);
        if IMAGE_EXT.contains(&e.as_str()) {
            Some(Media::Image)
        } else if ANIMATED_EXT.contains(&e.as_str()) {
            Some(Media::Animated)
        } else if VIDEO_EXT.contains(&e.as_str()) {
            Some(Media::Video)
        } else {
            None
        }
    }

    fn name(self) -> &'static str {
        match self {
            Media::Image => "image",
            Media::Animated => "animated",
            Media::Video => "video",
        }
    }
}

pub fn is_hex_color(v: &str) -> bool {
    v.strip_prefix('#')
        .is_some_and(|h| matches!(h.len(), 3 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit()))
}

fn existing_file(path: &str, what: &str, exts: &[&str]) -> Result<(), String> {
    let p = expand(path);
    if !exts.contains(&ext(&p).as_str()) {
        return Err(format!(
            "{what} must end in .{}, got {path:?}",
            exts.join(", .")
        ));
    }
    if !p.is_file() {
        return Err(format!("{} does not exist", p.display()));
    }
    Ok(())
}

fn number(v: &str, min: f64, max: f64) -> Result<f64, String> {
    let n: f64 = v
        .parse()
        .map_err(|_| format!("expected a number, got {v:?}"))?;
    if (min..=max).contains(&n) {
        Ok(n)
    } else {
        Err(format!("expected {min}..={max}, got {n}"))
    }
}

// Checks a config value for a standard setting or one of the theme's colour roles.
pub fn check(theme: &Theme, key: &str, value: &str) -> Result<(), String> {
    if let Some(why) = unsupported(theme, key) {
        return Err(why);
    }
    let kind = setting(key).map_or(Kind::Color, |s| s.kind);
    match kind {
        Kind::Background => match value {
            DESKTOP => Ok(()),
            v if v.starts_with('#') => {
                if is_hex_color(v) {
                    Ok(())
                } else {
                    Err(format!("expected #RGB, #RRGGBB or #AARRGGBB, got {v:?}"))
                }
            }
            v => {
                let exts: Vec<&str> = [IMAGE_EXT, ANIMATED_EXT, VIDEO_EXT].concat();
                existing_file(v, "a background", &exts)
            }
        },
        Kind::Fit => one_of(value, &["cover", "contain"]),
        Kind::Percent => value
            .parse::<u8>()
            .ok()
            .filter(|n| *n <= 80)
            .map(drop)
            .ok_or_else(|| format!("expected 0..=80, got {value:?}")),
        Kind::Variant => {
            if value == "auto" || theme.manifest.supports.variants.iter().any(|v| v == value) {
                Ok(())
            } else {
                Err(format!(
                    "expected auto or {}, got {value:?}",
                    theme.manifest.supports.variants.join(", ")
                ))
            }
        }
        Kind::Color => {
            if value == GENERATE || is_hex_color(value) {
                Ok(())
            } else {
                Err(format!(
                    "expected a colour like #e6bb5c, or generate, got {value:?}"
                ))
            }
        }
        Kind::ColorSource => one_of(value, &["background", DESKTOP]),
        Kind::Scheme => one_of(value, &SCHEMES.iter().map(|s| s.0).collect::<Vec<_>>()),
        Kind::Contrast => number(value, -1.0, 1.0).map(drop),
        Kind::Font => {
            if FONT_EXT.contains(&ext(Path::new(value)).as_str()) {
                existing_file(value, "a font file", FONT_EXT)
            } else if value.trim().is_empty()
                || value.len() > 100
                || value.chars().any(char::is_control)
            {
                Err(format!(
                    "expected a font family or a .ttf/.otf file, got {value:?}"
                ))
            } else {
                Ok(())
            }
        }
        Kind::Speed => number(value, 0.25, 3.0).map(drop),
        Kind::Curve => one_of(value, &CURVES.iter().map(|c| c.0).collect::<Vec<_>>()),
        Kind::Bool => one_of(value, &["true", "false"]),
    }
}

fn one_of(value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "expected one of {}, got {value:?}",
            allowed.join(", ")
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub path: PathBuf,
    // Who draws it, e.g. "DMS", shown by doctor and the GUI.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteRequest {
    pub scheme: String,
    // None: decided from the image's brightness.
    pub dark: Option<bool>,
    pub contrast: f64,
}

// Material role name (primary, on_surface, …) → "#rrggbb".
pub type Palette = BTreeMap<String, String>;

// What resolving needs from the running system; tests use Offline.
pub trait Host {
    fn desktop_prefers_dark(&self) -> Option<bool>;
    fn desktop_wallpaper(&self, dark: Option<bool>) -> Option<Wallpaper>;
    fn palette(&self, image: &Path, request: &PaletteRequest) -> Result<Palette, String>;
}

pub struct Offline;

impl Host for Offline {
    fn desktop_prefers_dark(&self) -> Option<bool> {
        None
    }
    fn desktop_wallpaper(&self, _: Option<bool>) -> Option<Wallpaper> {
        None
    }
    fn palette(&self, _: &Path, _: &PaletteRequest) -> Result<Palette, String> {
        Err("colour generation is not available here".into())
    }
}

// Turns checked config values into theme contract keys; values arrive validated by check().
pub fn apply(
    theme: &Theme,
    values: &BTreeMap<String, String>,
    host: &dyn Host,
    overlay: &mut BTreeMap<String, String>,
    issues: &mut Vec<(String, String)>,
) {
    let get = |k: &str| values.get(k).map(String::as_str);
    let sup = &theme.manifest.supports;

    let dark = match get("variant") {
        Some("auto") => host
            .desktop_prefers_dark()
            .map(|d| if d { "dark" } else { "light" })
            .filter(|v| sup.variants.iter().any(|x| x == v)),
        Some(v) => Some(v),
        None => None,
    };
    if let Some(v) = dark {
        overlay.insert("colorScheme".into(), v.into());
    }
    let variant = dark.or(sup.default_variant.as_deref());
    let is_dark = variant.map(|v| v == "dark");

    let mut background_file: Option<PathBuf> = None;
    match get("background") {
        None => {}
        Some(DESKTOP) => match host.desktop_wallpaper(is_dark) {
            Some(w) if is_hex_color(&w.path.display().to_string()) => {
                overlay.insert("backgroundType".into(), "color".into());
                overlay.insert("backgroundColor".into(), w.path.display().to_string());
            }
            Some(w) => match Media::of(&w.path) {
                Some(kind) => {
                    overlay.insert("backgroundType".into(), kind.name().into());
                    overlay.insert("backgroundPath".into(), w.path.display().to_string());
                    background_file = Some(w.path);
                }
                None => issues.push((
                    "background".into(),
                    format!(
                        "the desktop wallpaper {} is not an image or video",
                        w.path.display()
                    ),
                )),
            },
            None => issues.push((
                "background".into(),
                "no desktop wallpaper found; the theme keeps its own background".into(),
            )),
        },
        Some(c) if c.starts_with('#') => {
            overlay.insert("backgroundType".into(), "color".into());
            overlay.insert("backgroundColor".into(), c.into());
        }
        Some(path) => {
            let p = expand(path);
            if let Some(kind) = Media::of(&p) {
                overlay.insert("backgroundType".into(), kind.name().into());
                overlay.insert("backgroundPath".into(), p.display().to_string());
                background_file = Some(p);
            }
        }
    }
    if overlay.contains_key("backgroundType") {
        if let Some(v) = get("background_fit") {
            overlay.insert("backgroundFit".into(), v.into());
        }
        if let Some(v) = get("background_dim") {
            overlay.insert("backgroundDim".into(), v.into());
        }
    }

    // Colours: accent and text, then the theme's own roles, each a hex value or generated.
    let mut roles: Vec<(&str, &str, String)> = vec![
        ("accent", "colorAccent", "primary".to_string()),
        ("text_color", "colorText", "on_surface".to_string()),
    ];
    for c in &theme.manifest.colors {
        roles.push((c.key.as_str(), c.key.as_str(), c.material.clone()));
    }
    let user_colours = roles.iter().any(|(k, _, _)| get(k).is_some());
    let by_default = sup.generate_by_default && !user_colours;
    let wants_palette = by_default || roles.iter().any(|(k, _, _)| get(k) == Some(GENERATE));
    let palette = if wants_palette {
        let source = match get("color_source").unwrap_or("background") {
            DESKTOP => host.desktop_wallpaper(is_dark).map(|w| w.path),
            _ => background_file
                .clone()
                .or_else(|| {
                    let m = &theme.manifest;
                    let dark_file = m
                        .background_file_dark
                        .as_ref()
                        .filter(|_| is_dark == Some(true));
                    dark_file
                        .or(m.background_file.as_ref())
                        .map(|f| theme.dir.join(f))
                })
                .or_else(|| theme.preview.clone()),
        };
        let request = PaletteRequest {
            scheme: get("color_scheme").unwrap_or(SCHEMES[0].0).to_string(),
            dark: is_dark,
            contrast: get("color_contrast")
                .and_then(|c| c.parse().ok())
                .unwrap_or(0.0),
        };
        match source {
            Some(image) => host
                .palette(&image, &request)
                .map_err(|e| {
                    issues.push((
                        "color_source".into(),
                        format!("cannot generate colours: {e}"),
                    ));
                })
                .ok(),
            None => {
                issues.push((
                    "color_source".into(),
                    "no image to generate colours from".into(),
                ));
                None
            }
        }
    } else {
        None
    };
    if sup.material_palette
        && let Some(p) = &palette
    {
        for (role, hex) in p {
            overlay.insert(format!("material_{role}"), hex.clone());
        }
    }
    for (key, contract, material) in roles {
        match get(key) {
            Some(GENERATE) => {
                if let Some(hex) = palette.as_ref().and_then(|p| p.get(&material)) {
                    overlay.insert(contract.into(), hex.clone());
                }
            }
            Some(hex) => {
                overlay.insert(contract.into(), hex.into());
            }
            None => {}
        }
    }

    for (key, family, file) in [
        ("font_text", "fontText", "fontTextFile"),
        ("font_clock", "fontClock", "fontClockFile"),
    ] {
        match get(key) {
            Some(v) if FONT_EXT.contains(&ext(Path::new(v)).as_str()) => {
                overlay.insert(file.into(), expand(v).display().to_string());
            }
            Some(v) => {
                overlay.insert(family.into(), v.into());
            }
            None => {}
        }
    }

    if let Some(v) = get("motion_speed") {
        overlay.insert("animSpeed".into(), v.into());
    }
    if let Some(qt) = get("motion_curve").and_then(|v| CURVES.iter().find(|c| c.0 == v)) {
        overlay.insert("animEasing".into(), qt.2.into());
    }
    if get("reduce_motion") == Some("true") {
        overlay.insert("reduceMotion".into(), "true".into());
    }
}

// The same rules for an overlay that arrives already built (the root helper). None: not a contract key.
pub fn check_contract(theme: &Theme, key: &str, value: &str) -> Option<Result<(), String>> {
    let sup = &theme.manifest.supports;
    let absolute = |v: &str| {
        if Path::new(v).is_absolute() && !v.contains('\n') {
            Ok(())
        } else {
            Err(format!("expected an absolute path, got {v:?}"))
        }
    };
    let color = |v: &str| {
        if is_hex_color(v) {
            Ok(())
        } else {
            Err(format!("expected a hex colour, got {v:?}"))
        }
    };
    let gate = |yes: bool, r: Result<(), String>| {
        if yes {
            r
        } else {
            Err(format!("{} does not support {key}", theme.manifest.name))
        }
    };
    let fonts = |role: &str| sup.fonts.iter().any(|f| f == role);
    Some(match key {
        "backgroundType" => gate(
            sup.background,
            one_of(value, &["image", "animated", "video", "color"]),
        ),
        "backgroundPath" => gate(sup.background, absolute(value)),
        "backgroundColor" => gate(sup.background, color(value)),
        "backgroundFit" => gate(sup.background, one_of(value, &["cover", "contain"])),
        "backgroundDim" => gate(
            sup.background,
            value
                .parse::<u8>()
                .ok()
                .filter(|n| *n <= 80)
                .map(drop)
                .ok_or_else(|| format!("expected 0..=80, got {value:?}")),
        ),
        "colorScheme" => gate(sup.variants.iter().any(|v| v == value), Ok(())),
        "colorAccent" | "colorText" => gate(sup.colors, color(value)),
        "fontText" => gate(fonts("text"), Ok(())),
        "fontTextFile" => gate(fonts("text"), absolute(value)),
        "fontClock" => gate(fonts("clock"), Ok(())),
        "fontClockFile" => gate(fonts("clock"), absolute(value)),
        "animSpeed" => gate(sup.motion, number(value, 0.25, 3.0).map(drop)),
        "animEasing" => gate(
            sup.motion,
            one_of(value, &CURVES.iter().map(|c| c.2).collect::<Vec<_>>()),
        ),
        "reduceMotion" => gate(sup.motion, one_of(value, &["true", "false"])),
        k if theme.manifest.color(k).is_some() => color(value),
        k if k
            .strip_prefix("material_")
            .is_some_and(|r| MATERIAL_ROLES.contains(&r)) =>
        {
            gate(sup.material_palette, color(value))
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Theme;
    use crate::manifest::Manifest;

    fn theme(extra: &str) -> Theme {
        let manifest = Manifest::parse(&format!(
            "name = \"Rainy Room\"\nauthor = \"a\"\nbackground = \"video\"\n{extra}"
        ))
        .unwrap();
        Theme {
            id: "pixel-rainyroom".into(),
            dir: PathBuf::from("/nonexistent"),
            manifest,
            defaults: BTreeMap::new(),
            preview: None,
        }
    }

    const ALL: &str = "[supports]\nbackground = true\ncolors = true\nfonts = [\"text\", \"clock\"]\nmotion = true\nvariants = [\"light\", \"dark\"]\ndefault_variant = \"dark\"\n\n[[color]]\nkey = \"colorLamp\"\nlabel = \"Lamp\"\nmaterial = \"tertiary\"\n";

    struct Fake;
    impl Host for Fake {
        fn desktop_prefers_dark(&self) -> Option<bool> {
            Some(false)
        }
        fn desktop_wallpaper(&self, dark: Option<bool>) -> Option<Wallpaper> {
            let name = if dark == Some(true) {
                "night.jpg"
            } else {
                "day.jpg"
            };
            Some(Wallpaper {
                path: PathBuf::from("/walls").join(name),
                source: "test".into(),
            })
        }
        fn palette(&self, image: &Path, r: &PaletteRequest) -> Result<Palette, String> {
            let tag = format!("{}-{}", image.display(), r.dark == Some(true));
            let mut p = Palette::new();
            p.insert("primary".into(), "#112233".into());
            p.insert("on_surface".into(), "#eeeeee".into());
            p.insert(
                "tertiary".into(),
                if tag.contains("night") {
                    "#000001".into()
                } else {
                    "#000002".into()
                },
            );
            Ok(p)
        }
    }

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn a_theme_without_support_explains_why_instead_of_accepting_a_value() {
        let plain = theme("");
        for key in [
            "background",
            "variant",
            "accent",
            "font_text",
            "motion_speed",
        ] {
            assert!(check(&plain, key, "x").is_err(), "{key}");
            assert!(unsupported(&plain, key).is_some(), "{key}");
        }
        assert!(
            unsupported(&plain, "variant")
                .unwrap()
                .contains("artwork is the design")
        );
    }

    #[test]
    fn values_are_checked_by_kind() {
        let t = theme(ALL);
        assert!(check(&t, "accent", "#e6bb5c").is_ok());
        assert!(check(&t, "accent", GENERATE).is_ok());
        assert!(check(&t, "accent", "red").is_err());
        assert!(
            check(&t, "colorLamp", "#123").is_ok(),
            "a theme's own colour role"
        );
        assert!(check(&t, "variant", "dark").is_ok());
        assert!(check(&t, "variant", "sepia").is_err());
        assert!(check(&t, "motion_speed", "0.5").is_ok());
        assert!(check(&t, "motion_speed", "5").is_err());
        assert!(check(&t, "motion_curve", "bouncy").is_ok());
        assert!(check(&t, "background_dim", "81").is_err());
        assert!(check(&t, "background", DESKTOP).is_ok());
        assert!(check(&t, "background", "#000").is_ok());
        assert!(check(&t, "background", "/nope/missing.mp4").is_err());
        assert!(
            check(&t, "background", "/etc/hostname").is_err(),
            "not an image or video"
        );
        assert!(check(&t, "font_text", "JetBrains Mono").is_ok());
        assert!(check(&t, "font_text", "/nope/x.ttf").is_err());
        assert!(check(&t, "color_scheme", "scheme-vibrant").is_ok());
        assert!(check(&t, "color_contrast", "2").is_err());
    }

    #[test]
    fn a_user_file_becomes_its_media_type_and_path() {
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("rain.MP4");
        std::fs::write(&clip, b"x").unwrap();
        let t = theme(ALL);
        let path = clip.display().to_string();
        assert!(check(&t, "background", &path).is_ok());
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        apply(
            &t,
            &values(&[("background", &path), ("background_dim", "30")]),
            &Offline,
            &mut overlay,
            &mut issues,
        );
        assert_eq!(
            overlay.get("backgroundType").map(String::as_str),
            Some("video")
        );
        assert_eq!(overlay.get("backgroundPath"), Some(&path));
        assert_eq!(overlay.get("backgroundDim").map(String::as_str), Some("30"));
        assert!(issues.is_empty());
    }

    #[test]
    fn auto_follows_the_desktop_and_the_wallpaper_follows_the_variant() {
        let t = theme(ALL);
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        apply(
            &t,
            &values(&[("variant", "auto"), ("background", DESKTOP)]),
            &Fake,
            &mut overlay,
            &mut issues,
        );
        assert_eq!(
            overlay.get("colorScheme").map(String::as_str),
            Some("light")
        );
        assert_eq!(
            overlay.get("backgroundPath").map(String::as_str),
            Some("/walls/day.jpg")
        );
    }

    #[test]
    fn generated_colours_map_material_roles_and_follow_the_variant() {
        let t = theme(ALL);
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        let v = values(&[
            ("variant", "dark"),
            ("color_source", DESKTOP),
            ("accent", GENERATE),
            ("text_color", "#ffffff"),
            ("colorLamp", GENERATE),
        ]);
        apply(&t, &v, &Fake, &mut overlay, &mut issues);
        assert_eq!(
            overlay.get("colorAccent").map(String::as_str),
            Some("#112233")
        );
        assert_eq!(
            overlay.get("colorText").map(String::as_str),
            Some("#ffffff"),
            "a manual colour wins"
        );
        assert_eq!(
            overlay.get("colorLamp").map(String::as_str),
            Some("#000001"),
            "night wallpaper for dark"
        );
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn a_generate_by_default_theme_gets_the_whole_palette_until_the_user_picks_colours() {
        let t = theme(
            "[supports]\ncolors = true\nmaterial_palette = true\ngenerate_by_default = true\nvariants = [\"light\", \"dark\"]\ndefault_variant = \"light\"\n",
        );
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        apply(
            &t,
            &values(&[("color_source", DESKTOP)]),
            &Fake,
            &mut overlay,
            &mut issues,
        );
        assert_eq!(
            overlay.get("material_primary").map(String::as_str),
            Some("#112233")
        );
        assert!(
            !overlay.contains_key("colorAccent"),
            "the theme maps the palette itself"
        );
        for (k, v) in &overlay {
            assert_eq!(check_contract(&t, k, v), Some(Ok(())), "{k}");
        }
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        apply(
            &t,
            &values(&[("accent", "#ff0000")]),
            &Fake,
            &mut overlay,
            &mut issues,
        );
        assert!(
            !overlay.contains_key("material_primary"),
            "a picked colour turns generation off"
        );
        assert_eq!(
            overlay.get("colorAccent").map(String::as_str),
            Some("#ff0000")
        );
    }

    #[test]
    fn a_missing_desktop_wallpaper_is_an_issue_and_the_theme_keeps_its_own() {
        let t = theme(ALL);
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        apply(
            &t,
            &values(&[("background", DESKTOP)]),
            &Offline,
            &mut overlay,
            &mut issues,
        );
        assert!(!overlay.contains_key("backgroundType"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn motion_fonts_and_contract_checks() {
        let t = theme(ALL);
        let (mut overlay, mut issues) = (BTreeMap::new(), Vec::new());
        let v = values(&[
            ("motion_speed", "1.5"),
            ("motion_curve", "snappy"),
            ("reduce_motion", "true"),
            ("font_text", "Inter"),
        ]);
        apply(&t, &v, &Offline, &mut overlay, &mut issues);
        assert_eq!(
            overlay.get("animEasing").map(String::as_str),
            Some("OutExpo")
        );
        assert_eq!(overlay.get("fontText").map(String::as_str), Some("Inter"));
        for (k, v) in &overlay {
            assert_eq!(check_contract(&t, k, v), Some(Ok(())), "{k}={v}");
        }
        assert!(
            check_contract(&t, "animEasing", "InBounce")
                .unwrap()
                .is_err()
        );
        assert!(
            check_contract(&theme(""), "animSpeed", "1")
                .unwrap()
                .is_err(),
            "unsupported"
        );
        assert_eq!(check_contract(&t, "themeMode", "dark"), None);
    }
}
