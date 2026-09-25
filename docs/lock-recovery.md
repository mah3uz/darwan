# Recovering a crashed lock

If the lockscreen crashes or hangs while the session is locked, the compositor keeps the session locked (Hyprland shows a "lockscreen died" message). Your session and open windows are still there.

## With darwan

1. Switch to a text console: <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>F3</kbd> (try F4 if F3 is blank).
2. Log in as the same user and run:
   ```
   darwan lock --replace
   ```
   From a repo checkout (nothing installed), use the built binary and point it at the repo:
   ```
   DARWAN_DATA_DIR=~/Projects/darwan ~/Projects/darwan/target/release/darwan lock --replace <theme>
   ```
   It finds your Wayland session, kills the hung locker if one is still running, and starts a new one.
3. Switch back to your session (<kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>F2</kbd>, or whichever console it runs on) and unlock with your password.

A new locker can only take over when Hyprland's `misc:allow_session_lock_restore` is on. `darwan lock` checks it before every lock and refuses to lock if it is off. With a Lua Hyprland config, `hyprctl keyword` can't change it, so set it in your config.

## If darwan itself is broken

A `.qml` file is not a program; start it through Quickshell. From the text console:

```
export XDG_RUNTIME_DIR=/run/user/$(id -u) WAYLAND_DISPLAY=wayland-1 XDG_SESSION_TYPE=wayland QML_XHR_ALLOW_FILE_READ=1
setsid -f quickshell -p ~/Projects/darwan/runtime/lock_shell.qml
```

Use `/usr/share/darwan/runtime/lock_shell.qml` once darwan is installed. Without `DARWAN_THEME_PATH` it shows the built-in password prompt, which is enough to unlock. Don't use `sudo`: the locker must run as the session's user.

## Last resort

End the whole graphical session. Unsaved work is lost, but you are no longer locked out.

- Started with uwsm: `uwsm stop`
- Any session: `loginctl terminate-session $(loginctl show-user $USER -p Display --value)`
