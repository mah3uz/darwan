<p align="center">
  <img src="./banner.png" alt="Darwan" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.5);"/>
</p>

<p align="center">
  <a href="#sddm"><img src="https://img.shields.io/badge/SDDM-black?style=for-the-badge&color=7aa2f7&labelColor=1a1b26&logo=linux&logoColor=white"/></a>&nbsp;<a href="#usage"><img src="https://img.shields.io/badge/QUICKSHELL-black?style=for-the-badge&color=bb9af7&labelColor=1a1b26&logo=qt&logoColor=white"/></a>&nbsp;<a href="#installation"><img src="https://img.shields.io/badge/ARCH%20LINUX-black?style=for-the-badge&color=1793d1&labelColor=1a1b26&logo=archlinux&logoColor=white"/></a>&nbsp;<a href="#architecture"><img src="https://img.shields.io/badge/RUST%20%2B%20QT-black?style=for-the-badge&color=e0af68&labelColor=1a1b26&logo=rust&logoColor=white"/></a>&nbsp;<a href="#license"><img src="https://img.shields.io/badge/GPL--3.0-black?style=for-the-badge&color=9ece6a&labelColor=1a1b26&logo=gnu&logoColor=white"/></a>
</p>

<div align="center">
  <pre>
    <a href="#features">ꜰᴇᴀᴛᴜʀᴇꜱ</a>  •  <a href="#installation">ɪɴꜱᴛᴀʟʟ</a>  •  <a href="#usage">ᴜꜱᴀɢᴇ</a>  •  <a href="#configuration">ᴄᴏɴꜰɪɢ</a>  •  <a href="#preview">ᴘʀᴇᴠɪᴇᴡ</a>  •  <a href="#sddm">ꜱᴅᴅᴍ</a>  •  <a href="#faq">ꜰᴀǫ</a>  •  <a href="#gallery">ɢᴀʟʟᴇʀʏ</a>  •  <a href="#acknowledgements">ᴀᴄᴋɴᴏᴡʟᴇᴅɢᴇᴍᴇɴᴛꜱ</a>
  </pre>
</div>

<br>

<p>Welcome to <b>Darwan</b>! It's one app, with a CLI, a TUI and a GUI, for choosing, configuring, previewing and applying a collection of hand-crafted themes to both of your gates:</p>

- **the login screen**, through SDDM
- **the lockscreen**, through Quickshell

> [!NOTE]
> **What's in the name?**
> **Darwan** (দারোয়ান, said *dar-waan*) is Bangla for *gatekeeper*: the guard who sits by the gate, checks who's coming in and politely sends everyone else away. This app does that job at both of your computer's gates.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="features"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-FEATURES-9ece6a?style=for-the-badge&labelColor=1a1b26&logo=sparkles&logoColor=white" height="60" />
</p>

<br>

- **One app, three faces.** `darwan` (CLI), `darwan` with no arguments (TUI) and `darwan-gui` (a Qt GUI) all share the same core, so they behave identically.
- **One config file.** `~/.config/darwan/config.toml` holds everything. Nothing inside the theme folders is ever edited.
- **Separate lockscreen and login themes.** Use Orbital for the lock and Rainy Room for SDDM if you like.
- **Options that know the theme.** Each theme declares what it supports. Options a theme can't use are shown disabled with the reason, and options that depend on another option unlock when it's set.
- **Test without locking yourself out.** A live preview in the GUI, a full-screen preview with a mock password, a preview through SDDM's own greeter, and headless checks that load every theme and type the password to make sure it unlocks.
- **Safe SDDM setup.** A small helper does the root-only work through polkit and checks every value again; nothing else runs as root.
- **Never locked out.** If a theme fails to load, Darwan shows a plain fallback password prompt instead of a black screen.
- **Global settings.** 12h/24h clock, AM/PM and a date format apply to every theme that supports them. Left unset, each theme keeps its own design.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="installation"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-INSTALLATION-1793d1?style=for-the-badge&labelColor=1a1b26&logo=archlinux&logoColor=white" height="60" />
</p>

<br>

> [!NOTE]
> Darwan supports **Arch Linux only**, with Qt6 SDDM and Quickshell on Wayland.

#### 📦 DEPENDENCIES

`makepkg -s` installs these for you.

| | Packages |
|--:|:---|
| **Required** | `quickshell` `qt6-base` `qt6-declarative` `qt6-5compat` `qt6-multimedia` `qt6-multimedia-ffmpeg` `polkit` `ttf-jetbrains-mono-nerd` |
| **Optional** | `sddm` (the login screen) · `libfaketime` (`darwan preview --at`) · `noto-fonts-cjk` (Chinese text in Genshin Impact) |
| **Build** | `rust` `lld` `git` |

#### 🚀 BUILD & INSTALL

```sh
git clone https://github.com/mah3uz/darwan.git
cd darwan/packaging/arch
makepkg -si
```

The PKGBUILD builds the tagged release from GitHub, runs the tests and installs one `darwan` package:

| Path | What |
|:---|:---|
| `/usr/bin/darwan` | CLI + TUI |
| `/usr/bin/darwan-gui` | GUI, also in your app launcher as *Darwan* |
| `/usr/lib/darwan/darwan-helper` | privileged SDDM helper, run through `pkexec` |
| `/usr/share/darwan/runtime/` | the QML runtime shared by the lockscreen and the previews |
| `/usr/share/darwan/themes/` | all 41 themes |
| `/usr/share/polkit-1/actions/org.darwan.policy` | lets the helper ask for your password once per session |

#### 🧪 RUN FROM SOURCE (DEVELOPMENT)

```sh
cargo build --release
DARWAN_DATA_DIR=$PWD target/release/darwan preview nier-automata
DARWAN_DATA_DIR=$PWD target/release/darwan-gui
```

`DARWAN_DATA_DIR` points every binary at `./runtime` and `./themes`, so nothing needs installing. Applying to SDDM and importing fonts still need the installed helper.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="usage"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-USAGE-bb9af7?style=for-the-badge&labelColor=1a1b26&logo=gnubash&logoColor=white" height="60" />
</p>

<br>

#### ⌨️ CLI

| Command | What it does |
|:---|:---|
| `darwan` | open the TUI |
| `darwan list` | list the themes; `L` marks the lock theme, `S` the SDDM theme |
| `darwan show <theme>` | a theme's details, fonts and settings |
| `darwan get [key]` · `set <key> <value>` · `unset <key>` | read, change or reset a setting, e.g. `darwan set clock.format 12h` |
| `darwan lock [theme]` | lock the screen now (default: the `[lock]` theme) |
| `darwan preview [theme]` | full-screen preview, no real lock ([details](#preview)) |
| `darwan check <theme>… \| --all` | headless test: QML errors, missing fonts, and whether typing the password unlocks |
| `darwan sddm apply` · `preview` · `status` · `reset` | manage the login screen ([details](#sddm)) |
| `darwan font import <theme> <file>` | install a licensed font a theme needs |
| `darwan doctor` | check the session, Quickshell, fonts, the helper and SDDM's config |

Setting keys are `lock.theme`, `sddm.theme`, `clock.format`, `clock.show_ampm`, `date.format` and `<theme>.<option>`.

#### 🖥️ TUI

Run `darwan` in a terminal. Themes are grouped into Clockwork, Pixel and Other themes, with a still preview of the selected one (kitty, sixel or iTerm graphics, falling back to block characters). Keys that can't work on your system are greyed out and say why.

| Key | Action | Key | Action |
|:---|:---|:---|:---|
| `⏎` | settings for the theme | `/` | search by name or id |
| `p` | preview as the lockscreen | `P` | preview with the SDDM layout |
| `l` | use as the lock theme | `L` | lock now |
| `s` | apply to the SDDM login screen | `S` | preview in SDDM's test mode |
| `f` | import a missing font | `c` | check the theme |
| `d` | doctor | `?` · `q` | all keys · quit |

#### 🎨 GUI

Run `darwan-gui`, or open *Darwan* from your launcher.

| Left | Centre | Right |
|:---|:---|:---|
| theme gallery with search | live preview that reloads as you change settings; click it and type `test` to unlock | the theme's settings, saved as you change them |

Below the preview: *Use for lock*, *Lock now*, *Full-screen preview*, *Apply to SDDM*, *SDDM test mode* and *Check*, plus *Import…* for missing fonts. *Doctor* is at the top right. The *Lockscreen* / *Login screen layout* switch shows the theme as each host would.

#### 🔒 LOCKSCREEN KEYBIND

*Use for lock* only chooses the theme. To lock with Darwan, point your lock keybind and idle locker at `darwan lock`, and let a new locker take over if one ever crashes. In Hyprland's Lua config:

```lua
hl.config({ misc = { allow_session_lock_restore = true } })
hl.bind("SUPER + L", hl.dsp.exec_cmd("darwan lock"), { desc = "Lock" })
```

or in a classic `hyprland.conf`:

```ini
misc {
    allow_session_lock_restore = true
}
bind = SUPER, L, exec, darwan lock
```

For hypridle, set `lock_cmd = darwan lock`. Pressing the keybind while already locked does nothing, because only one lockscreen runs at a time, and `darwan lock` refuses to lock while `allow_session_lock_restore` is off.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="configuration"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-CONFIGURATION-7aa2f7?style=for-the-badge&labelColor=1a1b26&logo=toml&logoColor=white" height="60" />
</p>

<br>

Everything lives in `~/.config/darwan/config.toml`. The CLI, TUI and GUI all edit this file and keep your comments. You can also edit it by hand.

```toml
[lock]
theme = "clockwork/orbital"

[sddm]
theme = "pixel-rainyroom"

[clock]
format = "12h"          # "12h" | "24h"; unset, each theme keeps its own
show_ampm = false

[date]
format = "ddd, MMM d"   # one of the presets below; unset, each theme keeps its own

[themes."clockwork/orbital"]
themeMode = "light"
enableWindup = false

[themes.terraria]
background_mode = "static"
background_index = "3"
```

#### 🎛️ PER-THEME OPTIONS

| Theme | Option | Values |
|:---|:---|:---|
| Clockwork · Orbital | `themeMode` | `dark` `light` |
| Clockwork · Orbital | `enableWindup` | `true` `false` |
| Clockwork · Tape | `themeMode` | `dark` `light` |
| osu! / osu! mania | `gameMode` | `menu` (straight to password) · `game` (rhythm-game gate) |
| Genshin Impact | `background_mode` · `background_index` | `time` `random` `static` · `1`–`4` (day, night, dawn, dusk) |
| Terraria | `background_mode` · `background_index` | `time` `random` `static` · `1`–`5` |

`darwan show <theme>` always lists the current set.

The clock settings reach the 39 themes that show a clock and the date setting the 38 that show a date: Nine Sols and Terraria show neither, and osu! has no date. Date presets:

| `date.format` | Looks like |
|:---|:---|
| `dddd, MMMM d` | Saturday, September 26 |
| `ddd, MMM d` | Sat, Sep 26 |
| `d MMMM yyyy` | 26 September 2026 |
| `yyyy-MM-dd` | 2026-09-26 |
| `dd/MM/yyyy` | 26/09/2026 |
| `MM/dd/yyyy` | 09/26/2026 |

#### 🔤 FONTS

Some themes use fonts that can't be bundled for copyright reasons. Until you add them, those themes fall back to a generic font. Get the font, then use *Import…* in the GUI, `f` in the TUI, or `darwan font import <theme> <file>`. In a source checkout, drop the file into the theme's `font/` folder instead.

| Theme | Font | Filename | Licence | Get it |
|--:|:---|:---|:---|:---|
| NieR: Automata | FOT-Rodin Pro DB | `FOT-Rodin Pro DB.otf` | Commercial (Fontworks) | [Adobe Fonts](https://fonts.adobe.com/fonts/fot-rodin-pron) |
| Terraria | Andy Bold | `Andy Bold.ttf` | Commercial (Monotype) | [MyFonts](https://www.myfonts.com/collections/andy-font-matteson-typographics/) |
| Genshin Impact | HYWenHei-85W | `zhcn.ttf` | Commercial (Hanyi) | [Hanyi](https://www.hanyi.com.cn/) |
| Sword | The Last Shuriken | `The Last Shuriken.ttf` | Free for personal use (Arterfak Project) | [DaFont](https://www.dafont.com/the-last-shuriken.font) |
| Honkai: Star Rail | DIN Next | `font.ttf` | Commercial (Monotype) | — |
| osu! | Torus Regular | `Torus Regular.otf` | Commercial (Paulo Goode) | [Paulo Goode](https://paulogoode.com/torus/) |
| Nothing | NDot 55 | `NDot55.otf` | Nothing brand use only | No public release |

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="preview"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-PREVIEW%20%26%20TESTING-e0af68?style=for-the-badge&labelColor=1a1b26&logo=testinglibrary&logoColor=white" height="60" />
</p>

<br>

You never have to lock your real session to try a theme.

| Mode | How | Password | Good for |
|:---|:---|:---|:---|
| **Live preview** | GUI centre pane | `test` | tweaking settings and seeing the change at once |
| **Full-screen preview** | `darwan preview <theme>` | `test`, or your real one with `--pam` | the exact lockscreen runtime, without locking; `Ctrl+Q` closes it |
| **SDDM preview** | `darwan sddm preview <theme>` | — (visual only) | how it looks in SDDM's own greeter before applying |
| **Headless check** | `darwan check --all --shots ./shots` | typed for you | QML errors, missing fonts and a real unlock test across every theme |

Useful preview flags:

```sh
darwan preview genshin --at 18:30          # fake the time of day (needs libfaketime)
darwan preview nier-automata --at 00:00    # see 12h vs 24h at midnight
darwan preview pixel-coffee --sddm         # the login screen layout
darwan preview osu --shot osu.png          # save a 1280x720 still once it has settled
```

`--at` runs the preview in Qt's own `qml` runner, because Quickshell can't start under libfaketime, so it always uses the mock password.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="sddm"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-SDDM-7aa2f7?style=for-the-badge&labelColor=1a1b26&logo=linux&logoColor=white" height="60" />
</p>

<br>

```sh
darwan sddm apply     # apply [sddm] theme + options (asks for your password via polkit)
darwan sddm preview   # show it in SDDM's test mode first
darwan sddm status    # show what SDDM will use and any conflicts
darwan sddm reset     # remove everything Darwan added
```

The SDDM greeter runs as its own user and can't read your home folder, so `apply` writes system files through `darwan-helper`. The helper checks every value again before writing, and it only ever touches these files:

| File | Purpose |
|:---|:---|
| `/usr/share/sddm/themes/darwan` | points at the chosen theme |
| `/usr/share/darwan/themes/<theme>/theme.conf.user` | your options, in SDDM's own override format |
| `/etc/sddm.conf.d/zz-darwan.conf` | `[Theme] Current=darwan` |
| `/usr/share/darwan/themes/<theme>/font/<file>` | fonts you import |

> [!TIP]
> SDDM reads `/etc/sddm.conf.d/` in alphabetical order, and later files win. If another file or `/etc/sddm.conf` also sets `Current=`, `darwan doctor` will point it out.

<br>
<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="architecture"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-ARCHITECTURE-9ece6a?style=for-the-badge&labelColor=1a1b26&logo=rust&logoColor=white" height="60" />
</p>

<br>

```
crates/darwan-core     Rust library: themes, config, validation, settings forms (no Qt)
crates/darwan          CLI + TUI (clap, ratatui); runs Quickshell for the lock, previews and checks
crates/darwan-gui      GUI (cxx-qt + QML) with the live preview
crates/darwan-helper   root-only SDDM and font writer, launched through pkexec
runtime/               QML: ThemeHost, the SDDM contract, the Quickshell lock/preview/check shells
themes/<id>/           Main.qml, theme.conf, metadata.desktop, darwan.toml, preview.jpg
```

- **Themes stay plain SDDM themes.** They read `config.<key>` exactly as they would under SDDM.
- **One override file for both runtimes.** Darwan resolves your settings into that override format, so the lockscreen and the login screen see the same values.
- **Themes describe themselves.** Each theme's `darwan.toml` lists its options (labels, types, choices and what they depend on) and which global settings it supports. The TUI and GUI build their forms from it.
- **Writing a theme?** [`docs/theme-contract.md`](./docs/theme-contract.md) lists everything a theme can rely on and the rules `darwan check` enforces.

<br>
<p align="center">━━━━━━━ ◈ ━━━━━━━</p>

<a id="faq"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-FAQ-7aa2f7?style=for-the-badge&labelColor=1a1b26&logo=helpdesk&logoColor=white" height="60" />
</p>

<br>

#### 🔓 A theme broke and I'm stuck on the lockscreen?
Darwan shows its fallback password prompt when a theme fails to load. If the locker itself crashes, switch to a text console (`Ctrl+Alt+F3`), log in and run `darwan lock --replace`, then switch back and unlock. [`docs/lock-recovery.md`](./docs/lock-recovery.md) covers every case. Afterwards, run `darwan check <theme>` and open an issue with the output.

<br>

#### ⌨️ Virtual keyboard popping up on the login screen?
Open `/etc/sddm.conf.d/virtualkeyboard.conf` as root and empty `InputMethod`:

```ini
[General]
InputMethod=
```

<br>

#### 📺 Low quality background video?
Videos are compressed to keep the download small. For the full HD/4K version, grab the original from the [Acknowledgements](#acknowledgements) table, rename it to `bg.mp4` and replace the one in the theme's folder.

<br>

#### ❄️ Lockscreen not working on KDE Plasma?
KWin doesn't support the `ext-session-lock-v1` protocol that the Quickshell lockscreen needs. The SDDM login screen still works fine on Plasma.

<br>

<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="gallery"></a>
<br>
<p align="center">
<img src="https://img.shields.io/badge/-GALLERY-9ece6a?style=for-the-badge&labelColor=1a1b26&logo=unsplash&logoColor=white" height="60" />
</p>
<br>
<div align="center">
<table style="border-collapse: collapse; border: none;">
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Clockwork · Neo-Orbital</b><br><br>
<img src="./themes/clockwork/neo-orbital/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Clockwork · Orbital</b><br><br>
<img src="./themes/clockwork/orbital/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Clockwork · Tape</b><br><br>
<img src="./themes/clockwork/tape/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Coffee</b><br><br>
<img src="./themes/pixel-coffee/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Cyberpunk</b><br><br>
<img src="./themes/pixel-cyberpunk/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Dusk City</b><br><br>
<img src="./themes/pixel-dusk-city/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Emerald</b><br><br>
<img src="./themes/pixel-emerald/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Hollow Knight</b><br><br>
<img src="./themes/pixel-hollowknight/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Munchlax</b><br><br>
<img src="./themes/pixel-munchlax/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Night City</b><br><br>
<img src="./themes/pixel-night-city/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Rainy Room</b><br><br>
<img src="./themes/pixel-rainyroom/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Sakura</b><br><br>
<img src="./themes/pixel-sakura/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Skyscrapers</b><br><br>
<img src="./themes/pixel-skyscrapers/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Pixel · Waterfall</b><br><br>
<img src="./themes/pixel-waterfall/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Dog Samurai</b><br><br>
<img src="./themes/dog-samurai/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Enfield</b><br><br>
<img src="./themes/enfield/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Field</b><br><br>
<img src="./themes/field/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Forest</b><br><br>
<img src="./themes/forest/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Genshin Impact</b><br><br>
<img src="./themes/genshin/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Girl · Coffee</b><br><br>
<img src="./themes/girl-coffee/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Girl · Pillow</b><br><br>
<img src="./themes/girl-pillow/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Honkai: Star Rail</b><br><br>
<img src="./themes/star-rail/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Man · Bicycle</b><br><br>
<img src="./themes/man-bicycle/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Material You</b><br><br>
<img src="./themes/material-you/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Material You Dark</b><br><br>
<img src="./themes/material-you-dark/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Minecraft</b><br><br>
<img src="./themes/minecraft/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>NieR: Automata</b><br><br>
<img src="./themes/nier-automata/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Nine Sols</b><br><br>
<img src="./themes/ninesols/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Ninja Gaiden</b><br><br>
<img src="./themes/ninja-gaiden/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Nothing</b><br><br>
<img src="./themes/nothing/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>osu!</b><br><br>
<img src="./themes/osu/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>osu! mania</b><br><br>
<img src="./themes/osumania/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Reverse: 1999 - I</b><br><br>
<img src="./themes/reverse-1999-1/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Reverse: 1999 - II</b><br><br>
<img src="./themes/reverse-1999-2/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Sword</b><br><br>
<img src="./themes/sword/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Terraria</b><br><br>
<img src="./themes/terraria/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>The Last of Us</b><br><br>
<img src="./themes/last-of-us/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Windows 7</b><br><br>
<img src="./themes/windows-7/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Winter</b><br><br>
<img src="./themes/winter/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Women · Umbrella</b><br><br>
<img src="./themes/women-umbrella/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
</tr>
<tr>
<td align="center" width="50%" style="padding: 15px; border: none;">
<b>Wuthering Waves</b><br><br>
<img src="./themes/wuwa/preview.jpg" width="100%" style="border-radius: 12px; box-shadow: 0 4px 15px rgba(0,0,0,0.3);"/>
</td>
<td></td>
</tr>
</table>
</div>

<br>

<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="acknowledgements"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-ACKNOWLEDGEMENTS-bb9af7?style=for-the-badge&labelColor=1a1b26&logo=google-photos&logoColor=white" height="60" />
</p>

<br>

> [!IMPORTANT]
> ### 💜 Built on qylock
> Darwan would not exist without **[qylock](https://github.com/Darkkal44/qylock)** by **[Darkkal44](https://github.com/Darkkal44)**.
>
> Every theme in this collection was designed and built there: the pixel worlds, the game tributes, Clockwork, and the SDDM/Quickshell shim that made one theme run in both places. Darwan only builds an app around that work. The hard, creative part is theirs.
>
> Thank you, Darkkal, and thank you to everyone who helped qylock grow:
> - the contributors
> - supporters **Max**, **Awkward**, **Chương Kính**, **MerhawiGhebrekal**, **Silenett**, **wawzi**, **franchecol** and **Trench Martyr**
> - special thanks to **Pumphium**, **kaizky** and **DragonChicken**
>
> If you enjoy these themes, please ⭐ [star qylock](https://github.com/Darkkal44/qylock) and [support Darkkal on Ko-fi](https://ko-fi.com/darkkal).

<p align="center">
  <a href="https://ko-fi.com/darkkal">
    <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support Darkkal on Ko-fi">
  </a>
</p>

<br>

Huge thanks to all the amazing artists for these wallpapers and fonts! Here's where everything comes from:

| Theme | Wallpaper | Font | Theme | Wallpaper | Font |
|:---|:---|:---|:---|:---|:---|
| **Pixel · Coffee** | [MoeWalls](https://moewalls.com/pixel-art/cyberpunk-coffee-pixel-live-wallpaper/) | Pixelify Sans | **Pixel · Munchlax** | [MoeWalls](https://moewalls.com/pixel-art/munchlax-sleeping-on-the-field-pixel-live-wallpaper/) | Pixelify Sans |
| **Pixel · Dusk City** | [WallsFlow](https://wallsflow.com/live-wallpapers/pixel-art/505-pixel-dusk-city-retro-anime-streets-live-wallpaper.html) | Pixelify Sans | **Pixel · Night City** | [WallsFlow](https://wallsflow.com/live-wallpapers/pixel-art/400-night-city-pixel-art-cyberpunk-live-wallpaper.html) | Pixelify Sans |
| **Pixel · Hollow Knight** | [MoeWalls](https://moewalls.com/pixel-art/hollow-knight-3-live-wallpaper/) | Pixelify Sans | **Pixel · Rainy Room** | [MoeWalls](https://moewalls.com/pixel-art/pixel-room-rainy-night-live-wallpaper/) | Pixelify Sans |
| **Pixel · Skyscrapers** | [WallsFlow](https://wallsflow.com/live-wallpapers/pixel-art/61-pixel-city.html) | Pixelify Sans | **Pixel · Cyberpunk** | [Pixiv](https://www.pixiv.net/en/artworks/84120766) | Pixelify Sans |
| **Pixel · Emerald** | - | Pixelify Sans | **Pixel · Sakura** | - | Pixelify Sans |
| **Pixel · Waterfall** | - | Pixelify Sans | **Enfield** | [WallsFlow](https://wallsflow.com/live-wallpapers/games/777-arknights-endfield-sakura-sanctuary-live-wallpaper.html) | Orbitron |
| **Sword** | [WallsFlow](https://wallsflow.com/live-wallpapers/anime/761-silent-katana-forest-samurai-live-wallpaper.html) | The Last Shuriken | **The Last of Us** | [MoeWalls](https://moewalls.com/games/the-last-of-us-sunset-live-wallpaper/) | Outfit |
| **Field** | [MoeWalls](https://moewalls.com/anime/fading-away-live-wallpaper/) | - | **Girl · Coffee** | [MoeWalls](https://moewalls.com/anime/chill-afternoon-girl-live-wallpaper/) | - |
| **Girl · Pillow** | [MoeWalls](https://moewalls.com/anime/lazy-afternoon-girl-live-wallpaper/) | Itim | **Man · Bicycle** | [MoeWalls](https://moewalls.com/landscape/traveling-with-the-bicycle-live-wallpaper/) | Itim |
| **Women · Umbrella** | [MoeWalls](https://moewalls.com/anime/women-with-umbrella-live-wallpaper/) | Itim | **Forest** | [MoeWalls](https://moewalls.com/landscape/in-the-early-morning-forest-live-wallpaper/) | Figtree |
| **Winter** | [MoeWalls](https://moewalls.com/landscape/winter-forest-snow-live-wallpaper/) | Orbitron | **Dog Samurai** | [MoeWalls](https://moewalls.com/others/doge-samurai-crying-live-wallpaper/) | Orbitron |
| **Honkai: Star Rail** | [YouTube](https://www.youtube.com/watch?v=Pz7Tu25EyXI) | DIN Next | **Genshin Impact** | [YouTube](https://www.youtube.com/watch?v=XG3vTgitBLE) | HYWenHei |
| **Wuthering Waves** | [YouTube](https://www.youtube.com/watch?v=xKKqi1zLrZ4) | Orbitron | **osu!** | [Official](https://osu.ppy.sh) | Torus Regular |
| **osu! mania** | [Official](https://osu.ppy.sh) | Torus Regular | **Minecraft** | [Minecraft Wiki](https://www.google.com/url?sa=t&source=web&rct=j&url=https%3A%2F%2Fminecraft.fandom.com%2Fwiki%2FBackground&ved=0CBkQjhxqFwoTCIC45qWs4pMDFQAAAAAdAAAAABAH&opi=89978449) | Minecraft |
| **NieR: Automata** | [Reddit](https://www.reddit.com/r/nier/comments/7nqcy7/the_final_nier_automata_title_screen_made_into/) | FOT-Rodin Pro DB | **Reverse: 1999** | [Taptap](https://www.taptap.com/topic/21175628) | Outfit |
| **Clockwork** | [WallsFlow](https://wallsflow.com/live-wallpapers/abstract/321-clock-mechanism-live-wallpaper.html) | Orbitron | **Terraria** | [Terraria Forums](https://forums.terraria.org/index.php?threads/terraria-desktop-wallpapers.12644/) | Andy Bold |
| **Ninja Gaiden** | [Noisy Pixel](https://noisypixel.net/ninja-gaiden-4-wallpapers-art-team/) | Tektur | **Windows 7** | [WallpaperAccess](https://wallpaperaccess.com/windows-7-lock-screen) | Segoe UI |

<br>

<p align="center">━━━━━━━ ❖ ━━━━━━━</p>

<a id="license"></a>
<br>

<p align="center">
  <img src="https://img.shields.io/badge/-LICENSE-9ece6a?style=for-the-badge&labelColor=1a1b26&logo=gnu&logoColor=white" height="60" />
</p>

<br>

Darwan is licensed under the [GNU GPL v3.0](./LICENSE), the same license as qylock, which it is derived from. Wallpapers and fonts belong to their respective creators listed above.

<br>
<p align="center">━━━━━━━ ༓ ━━━━━━━</p>

<div align="center">
  <p><i>Make your login your own. The darwan will guard it.</i></p>
</div>
