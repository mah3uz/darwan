# Changelog

What changed in each release of Darwan, newest first. Each release's section is also its GitHub Release notes.

## Unreleased

### Multi-monitor

**The lock shows on every screen.** With several monitors, the lock drew its theme on every screen but the primary one,
which stayed black. Every screen now shows it, whatever your compositor calls the monitors.

**Previews open on every monitor too**, as the lock does: `darwan preview`, with `--saver`, `--sddm` or `--at`, and the
GUI's *Full-screen preview* and *Screensaver preview*. What you type shows on each; Ctrl+Q or unlocking closes them
all. `--shot` still takes one picture.

**Your password stays hidden on the other screens.** What you type is copied only into fields that hide it too, so a
screen left on a username field never shows your password.

### When an untouched lock goes back to the screensaver

A lock left alone brings the screensaver back after 30 seconds, whether you locked by hand or the screensaver locked.
The new setting `saver.return_after` changes that: 5 seconds or more, nothing hides while something is typed. The
GUI has it under *When you step away* → *When you lock*, with 10 and 15 seconds, half a minute, a minute and five
minutes.

### The screen turns off while locked

On Hyprland, a lock nobody touches turns the screens off after 5 minutes, counted from the lock and from every touch,
whether you locked by hand or the screensaver did. Any key or mouse move turns them back on. `saver.screen_off_locked`
changes it (10 seconds or more, or `never`); the GUI has it under *When you step away* → *When you lock*.

### Other shells' idle timers

When DMS, Noctalia, Caelestia, Omarchy's shell or swayidle also locks, turns the screens off or suspends on idle,
*When you step away* says so and where to turn it off, and `darwan doctor` warns. Those timers act whatever Darwan's
settings say.

### Wallpapers

**Wallpapers in the GUI.** The pill at the top of the window now reads *Themes · Home · Library · Explore*, over
one of your wallpapers shown at full size.

- *Home* features a picture from your Library, with *View Wallpaper* and a strip of others to feature instead, then
  rows to browse: Today on Bing, Popular on Wallhaven, NASA's Astronomy Picture of the Day, Wikimedia Commons, and
  what you added last.
- *Explore* mixes every source by default; pick one with the source chips. Search Wallhaven, choose a shape
  (ultrawide, 16:9, 16:10) and Popular, Latest or Random, or open a topic (Nature, Space, City, Mountains…). The
  first 50 pictures load at once, 25 more each time you scroll to the end. *Filter* holds the switches for the
  optional groups.
- *Library* is your wallpaper folder and its subfolders, four levels down, newest first, with NEW and IN USE badges,
  colour chips and a search. **+** adds pictures or videos to it; **⚙** changes the folder and holds the colour
  generators.
- Open a wallpaper to see it full-window: its thumbnail at once, then a copy the size of your screen crossfades in
  (an online one is fetched in the background first). Step with ‹ › or the arrow keys; each step crossfades too.
  *Set Wallpaper*: with several screens, choose one or All. An online picture you've already
  downloaded shows DOWNLOADED and isn't fetched again. Once set, *Use on lockscreen too* makes your lock theme show
  your desktop wallpaper. When your wallpaper tool can only change by restarting (swaybg, for one), Darwan asks
  first: just this time, or always.

**Colours from the wallpaper.** When a shell makes your colours from the wallpaper (DMS, Caelestia, Noctalia),
Darwan sets the wallpaper through it, so the colours follow as they do from the shell's own settings, and runs
nothing else. When a plain wallpaper tool draws it and you use matugen, pywal, wallust or hellwal, *Colours* on the
Wallpapers page (or `wallpaper.colours`) runs it after each change; pywal is told not to set the wallpaper itself, and
a matugen set up to change the wallpaper is left alone.

`darwan wallpaper set <file>` sets the desktop wallpaper through whatever draws it on your desktop: DMS, Noctalia,
Caelestia, Omarchy, KDE Plasma, GNOME, Cinnamon, MATE, Xfce, sway, waypaper, hyprpaper, awww/swww, wpaperd,
mpvpaper, gSlapper, swaybg or wbg. Name screens with `-o DP-1`. Darwan checks the result by asking the tool what it
shows. A tool that can only change by being restarted (swaybg, mpvpaper without its socket) is restarted only with
`--allow-restart`, and never when systemd runs it. `darwan wallpaper status` shows what draws the wallpaper, what it
can do and what each screen shows.

Your wallpapers live in `wallpaper.folder`: by default `Wallpapers` (or `wallpapers`) in your Pictures folder, found
through `user-dirs.dirs` so a localised Pictures folder works. `darwan wallpaper list` lists them and
`darwan wallpaper prepare` makes their thumbnails and colours ahead of time. Thumbnails go to the shared freedesktop
cache (`~/.cache/thumbnails`), so your file manager and Darwan make each one once.

`darwan wallpaper online` browses free wallpapers: Wallhaven (searchable), Bing's image of the day, NASA's Astronomy
Picture of the Day and Wikimedia Commons' featured pictures (nature, space, city, night). `--download N` saves one
into your wallpaper folder with its credit (author, licence and source page), and `--set` also sets it. Sexual
content in any form is never shown, from any source. People and portraits, anime and manga, games, films and TV,
war and weapons, gore and violence, and horror are hidden too until you allow them with `wallpaper.allow`
(e.g. `darwan set wallpaper.allow anime,games`). No account or API key is needed.

### Also

- **Smoother scrolling.** A mouse-wheel notch on *Themes* and the wallpaper pages now glides a whole row of cards
  instead of 72 pixels; notches in quick succession add up, and a touchpad scrolls as before. The *Themes* cards are
  drawn more cheaply, so the page opens and scrolls with less work however many themes you have.
- Building the `darwan` package from source now also needs `qt6-shadertools`.
- The GUI's hints are shorter.
- The *Themes* page opens as the wallpaper pages do: your lockscreen theme fills the window behind its name, with
  *Customise* and *Lock now*, and a strip under it to feature your login screen instead. Scrolling down blurs it into
  the background of the theme cards.
- The *Darwan* / *System* look moved from the window's left edge into **⚙** (Settings → Appearance); Darwan's own
  look is still the default.
- Using your desktop wallpaper as a theme's background no longer mistakes an editor open on Omarchy's or Caelestia's
  files for those shells, and ignores a second awww or swww daemon drawing niri's overview backdrop.
- The GUI's floating *Unsaved changes* bar no longer lets its Save button touch the bar's right edge.
- The lock and screensaver keep a `QSG_RENDER_LOOP` you set yourself instead of always choosing `threaded` on NVIDIA.

## 0.4.0 - 2026-09-29 18:38 +06:00

### A new GUI

**Every theme is a card.** The start screen shows your lockscreen and login screen, what happens when you step away,
and every theme as a card that plays its unlock when you point at it. Filter by family, background or font, or search
with Ctrl+F.

**A theme opens live, full-window**, growing out of its card. Try it, Check it, hold Compare changes (or `\`) to see
it as it ships, or Use it as your lockscreen or login screen. Arrow keys and a strip along the bottom step through the
themes, and a theme you opened before comes back at once.

**Its settings sit beside it** (Ctrl+I hides them), grouped, with a tab for this theme and one for every theme. Every
change shows in the preview at once, and nothing is written until you Save (Ctrl+S).

**The screensaver is a timeline**: when it starts, when it locks, when the screen goes off and when the machine
sleeps, on one line. Change it from "When you step away" on the start screen. Its changes to `hypridle.conf` wait for
Save like every other setting.

**Two looks**: Darwan's own, frosted glass over a blurred theme, or your system's Qt theme. Switch on the window's left
edge; it's saved at once (`gui.look = darwan | system`).

### Upgrading from 0.3.0

- **New dependencies:** `qt6-imageformats` and `qt6-svg`, installed with the package.
- The Screensaver window is now the "When you step away" row on the start screen.

## 0.3.0 - 2026-09-28 20:04 +06:00

### Every theme is a screensaver

When you're idle, your lock theme fades in over the desktop with only its background and its animation (rain,
drifting ash, a turning dial), no password field; clock themes keep their clock. Any key or click fades it away to
your desktop, or, once it has locked, brings in the lock's widgets, and the first key you type goes straight into the
password field. After `saver.lock_after` seconds (10 by default) it locks by turning the same running theme into the
lock, so nothing reloads or flashes. A revealed lock settles back to the screensaver after 30 seconds without input.

All 40 themes have a screensaver mode. Videos play through libmpv on the GPU that draws them.

### Setting it up

hypridle starts it. The GUI's **Screensaver** window says whether hypridle is installed and running, how to start it
with your session, and has a Start now button. It writes `~/.config/hypr/hypridle.conf` for you (the screensaver after
5 minutes, the screen off after 10, Darwan locking before sleep) while keeping your own lines, changes the idle,
screen-off and suspend times, and restarts hypridle to apply them. The README has the same setup by hand.

`darwan doctor` checks all of it: hypridle running, its screensaver listener, `darwan resumed` after sleep, locking
before sleep, the screensaver plugin, and which video quality this machine gets and why.

### Video quality

`saver.quality` picks what the screensaver and lock play: `full` (as shipped, the default), `auto` (chosen for this
machine: lighter on integrated graphics, on battery, without a video decoder or with less than 8 GB of memory), `eco`
(up to 1080p and 30 fps, from copies `darwan prepare-media` makes) or `still`. With several monitors, every screen
plays at `full`; at the other qualities only the focused one does and the rest show a still.

### Also

- `darwan preview <theme> --saver` shows a theme's screensaver without locking.
- With several monitors, what you type shows in every screen's password field.
- The screensaver doesn't come back after waking from sleep without input.
- NieR's rings turn by native animation, and NieR's emblem and Forest's glass rest while the widgets are hidden.
- Full-screen stills at 4K and above load at screen size.
- If Darwan's plugin is missing, the lock still locks and unlocks; only the screensaver mode is lost.

### Upgrading from 0.2.1

- **New dependency:** `mpvqt`, installed with the package.
- **Turn the screensaver on:** open the GUI's Screensaver window, or add the `darwan saver` listener and
  `after_sleep_cmd = darwan resumed` from the README to your `hypridle.conf`. Then run `darwan doctor`.

## 0.2.1 - 2026-09-28 13:13 +06:00

### The login screen shows the theme you picked

After switching the login screen from one theme to another, SDDM could show the new theme's settings on the old
theme's widgets: every theme was reached through the same link, and Qt's QML cache kept running the previous one. Each
theme now has its own link (`/usr/share/sddm/themes/darwan-<id>`), and applying a theme removes Darwan's other links.

### Colours keep what you pick

- In Material You, generating builds the palette from the background, and a colour you pick replaces only its own
  part: a picked accent leads the primary colours while the image still gives the surfaces and text. A picked accent
  used to be forgotten as soon as any colour was set to generate.
- Short and alpha codes such as `#f00` now seed the colour they name.

### A colour picker

In the GUI each colour is one chip showing the colour the preview really uses, generated ones included, and where it
comes from: the theme, the background or you. It opens a picker with Theme, From background and Custom: a colour
wheel with brightness, a hex field with Paste, and swatches of the background's colours and the theme's own. Every
choice shows in the preview at once, and Cancel or Escape puts back what was there.

### Put a theme back to its defaults

Reset theme in the GUI (into the unsaved draft, so Discard undoes it), `R` twice in the TUI, or `darwan unset <theme>`.

### Also

- The README's new Desktop shells section shows how to lock with Darwan under DankMaterialShell, Noctalia, Caelestia,
  illogical-impulse (end-4) and Omarchy: sending every lock to `darwan lock` and turning the shell's own lock off.
- Confirmations in the GUI's status bar fade like the TUI's, while problems stay until something replaces them.

### Upgrading from 0.2.0

If you use Darwan on the login screen, run `darwan sddm apply` once: it moves to the new per-theme link and removes the
old one.

## 0.2.0 - 2026-09-28 02:03 +06:00

### Make every theme yours

- **Your own background** behind any theme: an image, an animated GIF, a video, a plain colour, or `desktop` for your desktop wallpaper. Darwan finds what draws it: DankMaterialShell, Omarchy, Caelestia, Noctalia, hyprpaper, awww, swww, gSlapper, mpvpaper, swaybg or waypaper.
- **Colours** typed, or generated from the background the way Material You does, with matugen's schemes. In Material You, one accent colour recolours the whole theme.
- **Fonts** for the text and the clock, from your installed fonts or a font file.
- **Motion**: animation speed, curve, and reduce motion.
- **Light and dark looks** for Nothing, Neo-Orbital, Tape, Girl · Coffee, NieR: Automata, Orbital and Material You, designed for each theme, and `auto` to follow your desktop.
- The **login screen** gets your customisations too: `darwan sddm apply` sends your background and font files to the helper, which checks and stores its own copies.

All 40 themes take these settings where their design allows. See `darwan show <theme>`, or the settings on the right in the GUI.

### A lock that heals itself

`darwan lock` now runs under a small supervisor. If the lockscreen crashes, for example when a monitor drops out, a new one takes the lock back within a second or two and your desktop never shows. It also checks the lock after the screens wake up, and holds sleep until the lock is up. For hypridle, add `inhibit_sleep = 3` (see the README); `darwan doctor` checks it.

### Also

- The GUI keeps your changes as a draft: the live preview shows each one at once, and nothing is written until you press Save (Ctrl+S). It asks before switching themes, closing, or running a command with unsaved changes.
- The CLI is in colour on a terminal (plain in pipes, and with `NO_COLOR`), `darwan list` and `darwan show` are grouped like the GUI, and tab completion knows every customisation and its values.
- Orbital, Neo-Orbital and Tape sweep their dials natively instead of redrawing each frame, and Forest no longer draws a hidden full-screen blur.

### Upgrading from 0.1.0

- **Material You Dark is now Material You's dark look.** If you used `material-you-dark`, switch with `darwan set lock.theme material-you` (or `sddm.theme`) and `darwan set material-you.variant dark`.
- **Orbital's `themeMode` is now `variant`**: `darwan set clockwork/orbital.variant light` (or `dark`), then `darwan unset clockwork/orbital.themeMode`. Tape's `themeMode` option is gone; it never took effect.
- Run `darwan sddm apply` again if you use Darwan on the login screen.

## 0.1.0 - 2026-09-27 02:32 +06:00

The first release: 41 hand-crafted themes for the SDDM login screen and the Quickshell lockscreen, from one app.

- **A CLI, a TUI and a GUI** (`darwan`, `darwan` with no arguments, and `darwan-gui`) sharing one core.
- **One config file**, `~/.config/darwan/config.toml`; nothing inside the theme folders is ever edited.
- **Separate lockscreen and login themes.**
- **Options that know the theme**: options a theme can't use are shown disabled with the reason.
- **Test without locking yourself out**: a live preview in the GUI, a full-screen preview with a mock password, SDDM's
  own test mode, and headless checks that load every theme and type the password.
- **Safe SDDM setup**: a small helper does the root-only work through polkit and checks every value again.
- **Never locked out**: if a theme fails to load, Darwan shows a plain password prompt instead of a black screen.
- **Global clock and date settings** for every theme that shows them.
