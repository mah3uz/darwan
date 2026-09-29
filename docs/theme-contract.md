# Theme contract

A Darwan theme is a folder under `themes/` with a `Main.qml`. The same file runs in three hosts: the SDDM greeter, the Quickshell lockscreen (`darwan lock`) and the previews (`darwan preview`, `darwan check`, the GUI). A theme may rely only on what this page lists, and never imports Quickshell.

## Files

| File | Purpose |
|---|---|
| `Main.qml` | the theme |
| `theme.conf` | `[General]` defaults for every option; SDDM reads it directly |
| `metadata.desktop` | SDDM metadata; must set `MainScript=Main.qml` and `ConfigFile=theme.conf` |
| `darwan.toml` | Darwan manifest: name, options, required fonts (see `darwan_core::manifest`) |
| `preview.jpg` or `preview.png` | gallery still, 1280×720 (`darwan preview --shot`) |
| `preview.webp` (optional) | the GUI's hover loop, 640×360, made from the theme's demo by darwan-assets' `just loops` |
| `font/` | bundled, openly licensed fonts, or the user-supplied fonts listed in `darwan.toml` |

## Root names

These names resolve from anywhere in `Main.qml`, as they do under SDDM.

**`config`**: the keys of `theme.conf` `[General]`, as strings, with the user's overlay applied. Like SDDM's `theme.conf.user`, an empty overlay value does not override. Booleans are the strings `"true"`/`"false"`.

**`sddm`**
- `login(user, password, sessionIndex)`. On the lockscreen the user argument is ignored and the session owner is always authenticated.
- `powerOff()`, `reboot()`, `suspend()`; `hibernate()` and `hybridSleep()` do nothing.
- `canPowerOff`, `canReboot`, `canSuspend` (true), `canHibernate`, `canHybridSleep` (false).
- `hostName`: the machine name under SDDM, and **`undefined` on the lockscreen**. Themes use `sddm.hostName === undefined` to hide the session picker and power buttons when locked.
- Signals `loginSucceeded()`, `loginFailed()`, `informationMessage(message)`.

**`userModel`**: one row for the current user. Roles `name`, `realName`, `homeDir`, `icon`, `needsPassword`; `lastUser`, `lastIndex`, `count`, `rowCount()`, `index(row, column)`, `data(row, role)` with SDDM's role numbers (`Qt.UserRole + 1` = name, `+2` realName, `+3` homeDir, `+4` icon, `+5` needsPassword).

**`sessionModel`**: the installed Wayland and X11 sessions. Roles `name`, `file`, `type`, `exec`, `comment`; `lastIndex`, `count`, `rowCount()`, `index()`, `data()` (`Qt.UserRole + 2` file, `+3` type, `+4` name, `+5` exec, `+6` comment).

**`keyboard`**: writable `numLock` and `capsLock`; `layouts`, `currentLayout`, `enabled`. Setting `numLock` has no effect outside SDDM.

`primaryScreen` and `screenModel` are not provided; size the theme from `Screen.width`/`Screen.height` or its own size.

## Clock and date

A theme that sets `clock_format` or `date_format` under `[supports]` in `darwan.toml` follows the user's global settings through these `config` keys.

| Key | Values | Unset |
|---|---|---|
| `clockFormat` | `"12h"`, `"24h"` | `theme.conf` must set the theme's own |
| `clockShowAmPm` | `"true"`, `"false"`; only meaningful with `12h` | `theme.conf` must set the theme's own |
| `dateFormat` | a Qt date format from the presets in `darwan_core::settings::DATE_PRESETS` | `""`: use the theme's own format |
| `dateFormatNoWeekday` | the same preset without the weekday, for themes that show the weekday on its own line | `""` |

```qml
readonly property string clockFmt: config.clockFormat === "12h" ? (config.clockShowAmPm === "true" ? "h:mm AP" : "h:mm") : "HH:mm"
Text { text: Qt.formatTime(new Date(), clockFmt) }
Text { text: Qt.formatDate(new Date(), config.dateFormat || "dddd, MMMM d") }
```
In Qt formats `hh` without `AP` is still a 24-hour hour; a separate hour text needs `d.getHours() % 12 || 12` for the 12-hour clock.

## Customisation

A theme can let users change its background, colours, fonts, light/dark look and animation, through settings that
work the same in every theme. Declare what the theme supports in `darwan.toml`:

```toml
background_file = "bg.png"        # optional: the image colours are generated from by default

[supports]
background = true                 # the user's image, animated image, video, colour or desktop wallpaper
colors = true                     # accent and text colours, plus the [[color]] roles below
fonts = ["text", "clock"]         # which font roles the user can change
motion = true                     # animation speed, curve and reduce motion
variants = ["light", "dark"]      # only for a designed second look; theme.conf sets colorScheme
default_variant = "light"
material_palette = true           # the theme reads material_<role> keys: generated, or seeded from a picked accent
generate_by_default = false       # true: generate the palette until the user picks colours

# theme.conf states colorAccent and colorText for themes without variants (the form shows them as the defaults)

[[color]]                         # a colour beyond accent and text; its default lives in theme.conf
key = "colorLamp"
label = "Lamp"
material = "tertiary"             # the generated Material role it follows
```

Then copy the theme kit into the theme (`just theme-kit <id>`) and use it. The kit reads `config` the same way the
theme does, so it works under SDDM too:

```qml
import "darwan"

Custom { id: kit }

Background { id: userBg; anchors.fill: parent; z: -1 }         // the user's own background, when set
Image { source: "bg.png"; visible: !userBg.active }             // the theme's, otherwise

Text {
    color: kit.color("text", "#ffffff")                           // the theme's value when the user set none
    font.family: kit.font("clock", pixelFont)
}
Behavior on opacity { NumberAnimation { duration: kit.dur(300); easing.type: kit.ease(Easing.OutExpo) } }
readonly property color lamp: kit.color("colorLamp", "#e6bb5c")
readonly property bool isDark: kit.dark                           // with variants
```

| Kit | What it gives |
|---|---|
| `kit.dur(ms)` | `ms` divided by the user's speed, 0 with reduce motion |
| `kit.ease(themeDefault)` | the user's curve, or the theme's |
| `kit.color(role, themeDefault)` | role `accent`, `text` or a `[[color]]` key |
| `kit.font(role, themeDefault)` | role `text` or `clock`; an attached font file wins over a family |
| `kit.reduceMotion`, `kit.speed`, `kit.dark`, `kit.scheme` | for the theme's own logic |
| `Background { }` | `active` while the user's background shows; `failed` if its file didn't load |

Rules:
- **Stop ambient loops on reduce motion** (`running: !kit.reduceMotion`): `dur()` returns 0 for transitions, and an
  infinite animation with no duration would spin.
- **Hide the theme's own background while `userBg.active`**, and don't decode a video nobody sees: give the theme's
  `MediaPlayer` an empty `source`, or put it in a `Loader`, while the user's background shows.
- A missing or broken user file leaves `active` false, so the theme's own background shows; never an empty screen.
- The kit files in `darwan/` must stay identical to `runtime/theme-kit` (a test checks); change the kit there and
  run `just theme-kit` to refresh every theme.

## Screensaver

Every theme is also the screensaver (`darwan saver`), so every theme has two looks:
**ambient**, only its background and ambient animation (rain, drifting particles, a turning dial), and **revealed**, the
lock screen as usual. The saver starts ambient; the first key or click reveals the widgets, and a revealed lock settles
back to ambient after `saver.return_after` seconds (30 by default) without input while the password field is empty.
Under SDDM there is no screensaver and themes stay revealed.

Use the kit's `Ambient` (copied in by `just theme-kit`). Its `wake` is 1 while revealed and 0 in ambient, animated both
ways; multiply the widgets' opacity by it and keep the theme's own intro:

```qml
Custom { id: kit }
Ambient { id: saver; duration: kit.dur(700) }

property real intro: 0                                   // the theme's intro animates this from 0 to 1
readonly property real ui: intro * saver.wake            // what the widgets' opacity follows
```

`saver.active` (or the host's `darwan.ambient`) is the plain flag, for themes that change more than opacity.

Rules:
- **Hide widgets with opacity, and keep the password field focused.** The first printable key while ambient reveals the
  widgets and is typed into the focused field, so typing the password straight away works. `darwan check` unlocks each
  theme from ambient this way, and the lint fails a theme that uses neither `Ambient` nor `darwan.ambient`.
- **The background and its ambient effects stay.** A big clock is the theme's choice (Orbital keeps its dial).
- **No Timer under 100 ms that always runs** (`running: true`): the saver runs for hours, and such a timer runs
  JavaScript on nearly every frame. Use a native animation (`NumberAnimation`, `FrameAnimation`) or a slower timer. The
  lint checks this.
- **Videos play through `MediaPlayer` and `VideoOutput`** as before. Under darwan's hosts they play through libmpv on the
  GPU that renders, silent and looping; the user's `saver.quality` picks the tier: `full`, `eco` (a copy at up to 1080p
  and 30 fps, made when the theme is picked) or `still` (the first frame). A hidden `VideoOutput` pauses.

## Rules the checks enforce

`darwan check --all` fails a theme on any QML warning or error. `darwan check --all --no-fonts` hides `font/` and fails on any visible text whose `font.family` is empty.

- **Guard every font.** Never use `FontLoader.name` directly: it is `""` while loading and when the file is missing.
  ```qml
  FolderListModel { id: fontFolder; showDirs: false; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
  FontLoader { id: mainFont; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }
  readonly property string mainFontFamily: mainFont.status === FontLoader.Ready ? mainFont.name : "sans-serif"
  ```
- **`showDirs: false`** on every `FolderListModel`: when the folder is missing, the model falls back to the working directory and lists its folders.
- **Guard `currentItem`** of helper views before reading from it; it is null until the view creates its delegate.
- **Point only at files that exist.** An `Image` with a missing `source` is a warning; check with a `FolderListModel` first.
- **Don't assign objects to signal handlers** (`Component.onCompleted: NumberAnimation {...}`); declare the animation with `running: true`.
- **Back every `[supports]` flag.** `clock_format` needs `clockFormat` and `clockShowAmPm` defaults in `theme.conf` and QML that reads `config.clockFormat`; `date_format` needs QML that reads `config.dateFormat` (the lint in `cargo test`).
- **Persist settings with QtCore `Settings`** and an explicit `location`; `Qt.labs.settings` is deprecated and needs an organisation name.
