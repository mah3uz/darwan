import QtQuick
import QtQuick.Window
import Quickshell
import "contract"

ShellRoot {
    id: root

    readonly property bool usePam: Quickshell.env("DARWAN_AUTH") === "pam"

    Window {
        id: win
        visible: true
        title: "darwan preview: " + (Quickshell.env("DARWAN_THEME_ID") || "")
        visibility: Window.FullScreen
        color: "black"

        // Escape belongs to some themes (menus, clearing the password), so closing uses Ctrl+Q.
        Shortcut {
            sequence: "Ctrl+Q"
            onActivated: {
                host.unload()
                Qt.callLater(() => Qt.quit())
            }
        }

        // Themes size themselves from Screen, so render at screen size and scale down.
        ThemeHost {
            id: host
            width: Screen.width
            height: Screen.height
            transformOrigin: Item.TopLeft
            scale: Math.min(win.width / width, win.height / height)
            themePath: Quickshell.env("DARWAN_THEME_PATH")
            overlayPath: Quickshell.env("DARWAN_OVERLAY") || ""
            hostMode: Quickshell.env("DARWAN_MODE") || "lock"
            userName: Quickshell.env("DARWAN_USER") || "user"
            userRealName: Quickshell.env("DARWAN_REAL_NAME") || userName
            machineName: Quickshell.env("DARWAN_HOSTNAME") || ""
            sessionList: JSON.parse(Quickshell.env("DARWAN_SESSIONS") || "[]")
            authBackend: root.usePam ? pamLoader.item : mock
            onUnlocked: {
                console.log("darwan: unlocked")
                unload()
                Qt.callLater(() => Qt.quit())
            }
            onPowerOffRequested: console.log("darwan: power off requested (ignored in preview)")
            onRebootRequested: console.log("darwan: reboot requested (ignored in preview)")
            onSuspendRequested: console.log("darwan: suspend requested (ignored in preview)")
        }

        onClosing: {
            host.unload()
            Qt.callLater(() => Qt.quit())
        }
    }

    MockAuth {
        id: mock
    }

    Loader {
        id: pamLoader
        active: root.usePam
        source: "contract/PamAuth.qml"
    }
}
