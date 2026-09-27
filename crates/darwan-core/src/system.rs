use std::path::{Path, PathBuf};

use crate::custom::{Host, Palette, PaletteRequest, Wallpaper};
use crate::{palette, paths, wallpaper};

// The Host backed by the running desktop: the freedesktop portal, the wallpaper tools, the palette cache.
pub struct SystemHost {
    env: wallpaper::Env,
    cache_dir: PathBuf,
}

impl SystemHost {
    pub fn new() -> Self {
        Self {
            env: wallpaper::Env::system(),
            cache_dir: paths::cache_dir().join("palettes"),
        }
    }
}

impl Default for SystemHost {
    fn default() -> Self {
        Self::new()
    }
}

// org.freedesktop.appearance color-scheme: 1 prefers dark, 2 prefers light, 0 no preference.
fn portal_color_scheme() -> Option<u32> {
    let conn = zbus::blocking::Connection::session().ok()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Settings",
    )
    .ok()?;
    let value: zbus::zvariant::OwnedValue = proxy
        .call("ReadOne", &("org.freedesktop.appearance", "color-scheme"))
        .ok()?;
    u32::try_from(&value).ok()
}

impl Host for SystemHost {
    fn desktop_prefers_dark(&self) -> Option<bool> {
        match portal_color_scheme()? {
            1 => Some(true),
            2 => Some(false),
            _ => None,
        }
    }

    fn desktop_wallpaper(&self, dark: Option<bool>) -> Option<Wallpaper> {
        wallpaper::detect(&self.env, dark)
    }

    fn palette(&self, image: &Path, request: &PaletteRequest) -> Result<Palette, String> {
        palette::cached(&self.cache_dir, image, request)
    }
}
