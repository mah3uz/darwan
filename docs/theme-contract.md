# Theme contract

A Darwan theme is a folder under `themes/` with a `Main.qml`. The same file runs in three hosts: the SDDM greeter, the Quickshell lockscreen (`darwan lock`) and the previews (`darwan preview`, `darwan check`, the GUI). A theme may rely only on what this page lists, and never imports Quickshell.

## Files

| File | Purpose |
|---|---|
| `Main.qml` | the theme |
| `theme.conf` | `[General]` defaults for every option; SDDM reads it directly |
| `metadata.desktop` | SDDM metadata; must set `MainScript=Main.qml` and `ConfigFile=theme.conf` |
| `darwan.toml` | Darwan manifest: name, options, required fonts (see `darwan_core::manifest`) |
| `preview.gif` or `preview.png` | gallery preview |
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
- **Persist settings with QtCore `Settings`** and an explicit `location`; `Qt.labs.settings` is deprecated and needs an organisation name.
