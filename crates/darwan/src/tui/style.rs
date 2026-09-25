use ratatui::style::{Color, Modifier, Style};

pub struct Icons {
    pub themes: &'static str,
    pub preview: &'static str,
    pub details: &'static str,
    pub settings: &'static str,
    pub fonts: &'static str,
    pub section: &'static str,
    pub pointer: &'static str,
    pub lock: &'static str,
    pub sddm: &'static str,
    pub video: &'static str,
    pub image: &'static str,
    pub color: &'static str,
    pub author: &'static str,
    pub ok: &'static str,
    pub warn: &'static str,
    pub disabled: &'static str,
}

const NERD: Icons = Icons {
    themes: "\u{f03d8}",
    preview: "\u{f02e9}",
    details: "\u{f05a}",
    settings: "\u{f013}",
    fonts: "\u{f031}",
    section: "\u{f07b}",
    pointer: "\u{f0da}",
    lock: "\u{f023}",
    sddm: "\u{f0379}",
    video: "\u{f0567}",
    image: "\u{f03e}",
    color: "\u{f1fc}",
    author: "\u{f007}",
    ok: "\u{f00c}",
    warn: "\u{f071}",
    disabled: "\u{f05e}",
};

const GLYPH: Icons = Icons {
    themes: "◈",
    preview: "▣",
    details: "◇",
    settings: "≡",
    fonts: "A",
    section: "▸",
    pointer: "›",
    lock: "L",
    sddm: "S",
    video: "▶",
    image: "▣",
    color: "●",
    author: "@",
    ok: "✓",
    warn: "!",
    disabled: "–",
};

pub struct Palette {
    pub text: Color,
    pub subtext: Color,
    pub muted: Color,
    pub border: Color,
    pub accent: Color,
    pub lock: Color,
    pub sddm: Color,
    pub ok: Color,
    pub warn: Color,
    pub surface: Color,
}

const TRUECOLOR: Palette = Palette {
    text: Color::Rgb(0xcd, 0xd6, 0xf4),
    subtext: Color::Rgb(0xa6, 0xad, 0xc8),
    muted: Color::Rgb(0x6c, 0x70, 0x86),
    border: Color::Rgb(0x45, 0x47, 0x5a),
    accent: Color::Rgb(0xb4, 0xbe, 0xfe),
    lock: Color::Rgb(0x89, 0xdc, 0xeb),
    sddm: Color::Rgb(0xf5, 0xc2, 0xe7),
    ok: Color::Rgb(0xa6, 0xe3, 0xa1),
    warn: Color::Rgb(0xf9, 0xe2, 0xaf),
    surface: Color::Rgb(0x31, 0x32, 0x44),
};

const BASIC: Palette = Palette {
    text: Color::White,
    subtext: Color::Gray,
    muted: Color::DarkGray,
    border: Color::DarkGray,
    accent: Color::LightBlue,
    lock: Color::Cyan,
    sddm: Color::Magenta,
    ok: Color::Green,
    warn: Color::Yellow,
    surface: Color::Black,
};

pub struct Look {
    pub icons: &'static Icons,
    pub palette: &'static Palette,
}

impl Look {
    // A terminal can't report its font, so Nerd icons are used when a Nerd Font is installed.
    pub fn detect() -> Self {
        let nerd = match std::env::var("DARWAN_ICONS").as_deref() {
            Ok("nerd") => true,
            Ok("glyph") => false,
            _ => std::process::Command::new("fc-list")
                .args([":", "family"])
                .output()
                .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("Nerd Font")),
        };
        let truecolor = std::env::var("COLORTERM").is_ok_and(|c| c == "truecolor" || c == "24bit");
        Self {
            icons: if nerd { &NERD } else { &GLYPH },
            palette: if truecolor { &TRUECOLOR } else { &BASIC },
        }
    }

    #[cfg(test)]
    pub fn plain() -> Self {
        Self {
            icons: &GLYPH,
            palette: &BASIC,
        }
    }

    pub fn fg(&self, c: Color) -> Style {
        Style::new().fg(c)
    }

    pub fn heading(&self) -> Style {
        Style::new()
            .fg(self.palette.accent)
            .add_modifier(Modifier::BOLD)
    }
}
