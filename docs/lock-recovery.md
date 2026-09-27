# Lock recovery

If the lockscreen crashes while your session is locked, Darwan starts a new one by itself within a second or two, and it
takes the lock back. It also checks the lock after the screens wake up and restarts it if it isn't drawing or doesn't
take your keyboard. You only need this page if that recovery fails, or the lockscreen hangs instead of crashing: the
compositor keeps the session locked and Hyprland shows a "lockscreen died" message. Nothing is lost: your session and
open windows are still there. Work through these in order.

## 1. Start a new lockscreen

1. Switch to a text console with <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>F3</kbd>. If it stays blank, try
   <kbd>F4</kbd>.
2. Log in as the same user, then run:

   ```sh
   darwan lock --replace
   ```

   It finds your Wayland session, stops the hung lockscreen (and the process watching it) if it's still running, and
   starts a new one.
3. Switch back to your session with <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>F2</kbd> (or whichever console it runs on) and
   unlock with your password.

A new lockscreen can only take over when Hyprland's `misc:allow_session_lock_restore` option is on. `darwan lock`
checks it before every lock and refuses to lock while it's off, so if you've locked with darwan before, it's on. With a
Lua Hyprland config, `hyprctl keyword` can't change it; set it in your config (see the
[README](../README.md#lockscreen-keybind)).

Running from a source checkout instead of the package? Point the built binary at the checkout, and name the theme:

```sh
DARWAN_DATA_DIR=~/Projects/darwan ~/Projects/darwan/target/release/darwan lock --replace pixel-coffee
```

## 2. If `darwan` itself won't run

Start the lockscreen directly through Quickshell. Without a theme it shows Darwan's plain password prompt, which is
enough to unlock. From the text console, set up the session's environment:

```sh
export XDG_RUNTIME_DIR=/run/user/$(id -u) WAYLAND_DISPLAY=wayland-1 XDG_SESSION_TYPE=wayland QML_XHR_ALLOW_FILE_READ=1
```

Then start it:

```sh
setsid -f quickshell -p /usr/share/darwan/runtime/lock_shell.qml
```

From a source checkout, use `~/Projects/darwan/runtime/lock_shell.qml` instead. Don't use `sudo`: the lockscreen must
run as the session's user.

## 3. Last resort: end the session

This logs you out. Unsaved work is lost, but you're no longer locked out. If your session was started with uwsm:

```sh
uwsm stop
```

Otherwise:

```sh
loginctl terminate-session $(loginctl show-user $USER -p Display --value)
```

## Afterwards

Find out what broke, and include the output if you open an issue:

```sh
darwan check pixel-coffee
```

(with your lock theme's id), and:

```sh
darwan doctor
```
