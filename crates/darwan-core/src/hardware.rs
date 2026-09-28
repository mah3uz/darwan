use std::path::{Path, PathBuf};

use crate::saver::Quality;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Intel,
    Amd,
    Nvidia,
    Other,
}

impl Vendor {
    fn from_pci(id: &str) -> Self {
        match id.trim().trim_start_matches("0x") {
            "8086" => Vendor::Intel,
            "1002" => Vendor::Amd,
            "10de" => Vendor::Nvidia,
            _ => Vendor::Other,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Vendor::Intel => "Intel",
            Vendor::Amd => "AMD",
            Vendor::Nvidia => "NVIDIA",
            Vendor::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gpu {
    pub vendor: Vendor,
    // /sys/class/drm/cardN
    pub card: PathBuf,
    pub integrated: bool,
    // Whether the driver that decodes video on this GPU is installed.
    pub hw_decode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    // None: no GPU to render with (llvmpipe).
    pub gpu: Option<Gpu>,
    pub on_battery: bool,
    pub power_saver: bool,
    pub ram_gib: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Full,
    Eco,
    Still,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Full => "full",
            Tier::Eco => "eco",
            Tier::Still => "still",
        }
    }
}

// What `auto` means (docs/theme-contract.md "Screensaver"): software rendering gets stills; batteries, power saving,
// shared-memory GPUs, small RAM and missing decoders get eco copies at 1080p30 that every decoder handles.
pub fn tier(quality: Quality, facts: &Facts) -> (Tier, &'static str) {
    match quality {
        Quality::Full => return (Tier::Full, "saver.quality = full"),
        Quality::Eco => return (Tier::Eco, "saver.quality = eco"),
        Quality::Still => return (Tier::Still, "saver.quality = still"),
        Quality::Auto => {}
    }
    let Some(gpu) = &facts.gpu else {
        return (Tier::Still, "no GPU to render with");
    };
    if facts.power_saver {
        return (Tier::Still, "the power-saver profile is on");
    }
    if facts.on_battery {
        return (Tier::Eco, "on battery");
    }
    if !gpu.hw_decode {
        return (Tier::Eco, "no hardware video decoder driver");
    }
    if gpu.integrated {
        return (Tier::Eco, "integrated graphics share system memory");
    }
    if facts.ram_gib < 8 {
        return (Tier::Eco, "less than 8 GB of memory");
    }
    (Tier::Full, "a dedicated GPU with a video decoder")
}

pub fn probe() -> Facts {
    Facts {
        gpu: render_card().map(|card| gpu(&card)),
        on_battery: on_battery().unwrap_or(false),
        power_saver: power_saver().unwrap_or(false),
        ram_gib: ram_gib(std::fs::read_to_string("/proc/meminfo").ok()).unwrap_or(16),
    }
}

// The card Hyprland renders on: the first of AQ_DRM_DEVICES, else the boot VGA device, else the first card.
fn render_card() -> Option<PathBuf> {
    let drm = Path::new("/sys/class/drm");
    if let Ok(list) = std::env::var("AQ_DRM_DEVICES") {
        let first = list.split(':').next().unwrap_or("");
        if let Ok(real) = std::fs::canonicalize(first) {
            let name = real.file_name()?.to_owned();
            let card = drm.join(name);
            if card.exists() {
                return Some(card);
            }
        }
    }
    let mut cards: Vec<PathBuf> = std::fs::read_dir(drm)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("card") && !n.contains('-'))
        })
        .collect();
    cards.sort();
    cards
        .iter()
        .find(|c| read(&c.join("device/boot_vga")).as_deref() == Some("1"))
        .or(cards.first())
        .cloned()
}

fn gpu(card: &Path) -> Gpu {
    let vendor = read(&card.join("device/vendor"))
        .map(|v| Vendor::from_pci(&v))
        .unwrap_or(Vendor::Other);
    // An AMD APU reports its small carve-out as VRAM; a discrete card has gigabytes.
    let vram = read(&card.join("device/mem_info_vram_total")).and_then(|v| v.parse::<u64>().ok());
    let integrated = match vendor {
        Vendor::Intel => true,
        Vendor::Amd => vram.is_some_and(|b| b < 2 << 30),
        _ => false,
    };
    let lib = |names: &[&str]| {
        names.iter().any(|n| {
            ["/usr/lib/dri", "/usr/lib/x86_64-linux-gnu/dri"]
                .iter()
                .any(|d| Path::new(d).join(n).exists())
        })
    };
    let hw_decode = match vendor {
        Vendor::Intel => lib(&["iHD_drv_video.so", "i965_drv_video.so"]),
        Vendor::Amd => lib(&["radeonsi_drv_video.so"]),
        Vendor::Nvidia => Path::new("/usr/lib/libnvcuvid.so.1").exists(),
        Vendor::Other => false,
    };
    Gpu {
        vendor,
        card: card.to_path_buf(),
        integrated,
        hw_decode,
    }
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

fn ram_gib(text: Option<String>) -> Option<u64> {
    let text = text?;
    let kib: u64 = text
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))?
        .trim()
        .trim_end_matches("kB")
        .trim()
        .parse()
        .ok()?;
    Some(kib.div_ceil(1 << 20))
}

fn on_battery() -> Option<bool> {
    let conn = zbus::blocking::Connection::system().ok()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.freedesktop.UPower",
        "/org/freedesktop/UPower",
        "org.freedesktop.UPower",
    )
    .ok()?;
    proxy.get_property("OnBattery").ok()
}

fn power_saver() -> Option<bool> {
    let conn = zbus::blocking::Connection::system().ok()?;
    for (dest, path) in [
        (
            "org.freedesktop.UPower.PowerProfiles",
            "/org/freedesktop/UPower/PowerProfiles",
        ),
        ("net.hadess.PowerProfiles", "/net/hadess/PowerProfiles"),
    ] {
        let Ok(proxy) = zbus::blocking::Proxy::new(&conn, dest, path, dest) else {
            continue;
        };
        if let Ok(profile) = proxy.get_property::<String>("ActiveProfile") {
            return Some(profile == "power-saver");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(vendor: Vendor, integrated: bool, hw_decode: bool) -> Facts {
        Facts {
            gpu: Some(Gpu {
                vendor,
                card: PathBuf::from("/sys/class/drm/card0"),
                integrated,
                hw_decode,
            }),
            on_battery: false,
            power_saver: false,
            ram_gib: 32,
        }
    }

    #[test]
    fn auto_keeps_full_video_only_where_it_is_cheap() {
        assert_eq!(
            tier(Quality::Auto, &facts(Vendor::Nvidia, false, true)).0,
            Tier::Full
        );
        assert_eq!(
            tier(Quality::Auto, &facts(Vendor::Intel, true, true)).0,
            Tier::Eco,
            "an iGPU decodes into system RAM"
        );
        assert_eq!(
            tier(Quality::Auto, &facts(Vendor::Intel, true, false)).0,
            Tier::Eco,
            "without intel-media-driver Qt and mpv decode on the CPU"
        );
        let mut laptop = facts(Vendor::Amd, false, true);
        laptop.on_battery = true;
        assert_eq!(tier(Quality::Auto, &laptop).0, Tier::Eco);
        laptop.power_saver = true;
        assert_eq!(tier(Quality::Auto, &laptop).0, Tier::Still);
        let mut small = facts(Vendor::Amd, false, true);
        small.ram_gib = 4;
        assert_eq!(tier(Quality::Auto, &small).0, Tier::Eco);
        let none = Facts {
            gpu: None,
            ..facts(Vendor::Other, false, false)
        };
        assert_eq!(
            tier(Quality::Auto, &none).0,
            Tier::Still,
            "software rendering can't afford video"
        );
    }

    #[test]
    fn an_explicit_quality_wins_over_the_hardware() {
        let none = Facts {
            gpu: None,
            ..facts(Vendor::Other, false, false)
        };
        assert_eq!(tier(Quality::Full, &none).0, Tier::Full);
        assert_eq!(
            tier(Quality::Eco, &facts(Vendor::Nvidia, false, true)).0,
            Tier::Eco
        );
    }

    #[test]
    fn vendors_and_memory_parse_from_sysfs_and_proc() {
        assert_eq!(Vendor::from_pci("0x10de\n"), Vendor::Nvidia);
        assert_eq!(Vendor::from_pci("0x1002"), Vendor::Amd);
        assert_eq!(Vendor::from_pci("0x8086"), Vendor::Intel);
        assert_eq!(Vendor::from_pci("0x1af4"), Vendor::Other);
        assert_eq!(
            ram_gib(Some("MemTotal:       65536000 kB\nMemFree: 1 kB\n".into())),
            Some(63)
        );
        assert_eq!(ram_gib(Some("nothing".into())), None);
    }
}
