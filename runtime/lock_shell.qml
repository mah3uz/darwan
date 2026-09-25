import QtQuick
import Quickshell
import Quickshell.Wayland
import "contract"

ShellRoot {
    id: root

    property bool locked: true
    readonly property string themePath: Quickshell.env("DARWAN_THEME_PATH")
    readonly property var sessions: JSON.parse(Quickshell.env("DARWAN_SESSIONS") || "[]")

    function unlock(windup) {
        if (!locked)
            return
        Quickshell.execDetached(["loginctl", "unlock-session"])
        exitTimer.interval = windup ? 500 : 100
        exitTimer.start()
    }

    function run(action) {
        Quickshell.execDetached(["sh", "-c", "if [ -d /run/systemd/system ]; then systemctl " + action + "; else loginctl " + action + "; fi"])
    }

    Timer {
        interval: (parseInt(Quickshell.env("DARWAN_UNLOCK_AFTER")) || 0) * 1000
        running: interval > 0
        onTriggered: {
            console.warn("darwan: test lock timed out; unlocking")
            root.unlock(false)
        }
    }

    Timer {
        id: exitTimer
        onTriggered: {
            root.locked = false
            Qt.callLater(() => Qt.quit())
        }
    }

    WlSessionLock {
        locked: root.locked

        WlSessionLockSurface {
            color: "black"

            ThemeHost {
                id: host
                anchors.fill: parent
                themePath: root.themePath
                overlayPath: Quickshell.env("DARWAN_OVERLAY") || ""
                hostMode: "lock"
                userName: Quickshell.env("DARWAN_USER") || Quickshell.env("USER") || ""
                userRealName: Quickshell.env("DARWAN_REAL_NAME") || userName
                sessionList: root.sessions
                authBackend: PamAuth {}
                // Unload before quitting: a playing video crashes Qt's FFmpeg backend on exit.
                onUnlocked: {
                    unload()
                    root.unlock(config.enableWindup === "true")
                }
                onPowerOffRequested: root.run("poweroff")
                onRebootRequested: root.run("reboot")
                onSuspendRequested: root.run("suspend")
            }
        }
    }
}
