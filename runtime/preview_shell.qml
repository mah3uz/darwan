import QtQuick
import QtQuick.Window
import Quickshell
import Darwan
import "contract"

ShellRoot {
    id: root

    readonly property bool usePam: Quickshell.env("DARWAN_AUTH") === "pam"
    readonly property string shotPath: Quickshell.env("DARWAN_SHOT") || ""
    // The screensaver's look: ambient until input, back to ambient after 30 s idle with an empty field.
    readonly property bool saver: Quickshell.env("DARWAN_PREVIEW_SAVER") === "1"

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
            ambient: root.saver
            onUnlocked: {
                console.log("darwan: unlocked")
                unload()
                Qt.callLater(() => Qt.quit())
            }
            onPowerOffRequested: console.log("darwan: power off requested (ignored in preview)")
            onRebootRequested: console.log("darwan: reboot requested (ignored in preview)")
            onSuspendRequested: console.log("darwan: suspend requested (ignored in preview)")
        }

        InputGate {
            active: host.ambient
            passText: true
            onActivity: {
                host.ambient = false
                if (root.saver)
                    idle.restart()
            }
        }

        onClosing: {
            host.unload()
            Qt.callLater(() => Qt.quit())
        }
    }

    // Long enough for a video theme to show real frames and for intro animations to finish.
    Timer {
        interval: 4000
        running: root.shotPath !== "" && (host.themeReady || host.usingFallback)
        onTriggered: win.contentItem.grabToImage(result => {
            if (!result.saveToFile(root.shotPath))
                console.error("darwan: cannot save " + root.shotPath)
            host.unload()
            Qt.callLater(() => Qt.quit())
        }, Qt.size(1280, 720))
    }

    Timer {
        id: idle
        interval: 30000
        running: root.saver && !host.ambient
        onTriggered: {
            const f = win.activeFocusItem
            if (f && typeof f.text === "string" && f.text.length > 0)
                restart()
            else
                host.ambient = true
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
