import QtQuick
import QtQuick.Window
import "contract"

// Qt's qml runner hosts `preview --at` because Quickshell hangs under libfaketime.
Window {
    id: win

    readonly property var opts: JSON.parse(Qt.application.arguments[Qt.application.arguments.length - 1])

    function quit() {
        host.unload()
        Qt.callLater(() => Qt.quit())
    }

    visible: true
    title: "darwan preview: " + opts.themeId
    visibility: Window.FullScreen
    color: "black"

    // Escape belongs to some themes (menus, clearing the password), so closing uses Ctrl+Q.
    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: win.quit()
    }

    // Themes size themselves from Screen, so render at screen size and scale down.
    ThemeHost {
        id: host
        width: Screen.width
        height: Screen.height
        transformOrigin: Item.TopLeft
        scale: Math.min(win.width / width, win.height / height)
        themePath: win.opts.themePath
        overlayPath: win.opts.overlay
        hostMode: win.opts.mode
        userName: win.opts.user || "user"
        machineName: win.opts.hostName || ""
        sessionList: win.opts.sessions || []
        authBackend: MockAuth {}
        onUnlocked: {
            console.log("darwan: unlocked")
            win.quit()
        }
        onPowerOffRequested: console.log("darwan: power off requested (ignored in preview)")
        onRebootRequested: console.log("darwan: reboot requested (ignored in preview)")
        onSuspendRequested: console.log("darwan: suspend requested (ignored in preview)")
    }

    onClosing: win.quit()
}
