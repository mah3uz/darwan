import QtQuick
import QtQuick.Window
import Quickshell
import Darwan
import "contract"

// The theme full-screen on every screen, as the lock shows it; a screenshot uses one window.
ShellRoot {
    id: root

    readonly property bool usePam: Quickshell.env("DARWAN_AUTH") === "pam"
    readonly property string shotPath: Quickshell.env("DARWAN_SHOT") || ""
    // The screensaver's look: ambient until input, back to ambient after saver.return_after idle with an empty field.
    readonly property bool saver: Quickshell.env("DARWAN_PREVIEW_SAVER") === "1"
    readonly property int returnAfter: parseInt(Quickshell.env("DARWAN_RETURN_AFTER")) || 30000
    property bool ambient: saver
    property var hosts: []

    function quit() {
        for (const h of hosts)
            h.unload()
        Qt.callLater(() => Qt.quit())
    }

    function wake() {
        ambient = false
        if (saver)
            idle.restart()
    }

    function mirror(from, text) {
        for (const h of hosts) {
            const f = h !== from ? h.field : null
            if (f && typeof f.text === "string" && f.text !== text && f.echoMode === from.field.echoMode)
                f.text = text
        }
    }

    // Qt asks the compositor for full screen on the window's own output, so each window lands on its screen. The
    // position is for platforms that place windows by coordinates; Wayland ignores it.
    Instantiator {
        id: windows
        model: root.shotPath !== "" ? [Qt.application.screens[0]] : Qt.application.screens

        delegate: Window {
            id: win

            required property var modelData
            readonly property alias host: host

            screen: modelData
            x: modelData.virtualX
            y: modelData.virtualY
            width: modelData.width
            height: modelData.height
            visible: true
            title: "darwan preview: " + (Quickshell.env("DARWAN_THEME_ID") || "")
            visibility: Window.FullScreen
            color: "black"

            // Escape belongs to some themes (menus, clearing the password), so closing uses Ctrl+Q.
            Shortcut {
                sequence: "Ctrl+Q"
                onActivated: root.quit()
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
                ambient: root.ambient
                onUnlocked: {
                    console.log("darwan: unlocked")
                    root.quit()
                }
                onPowerOffRequested: console.log("darwan: power off requested (ignored in preview)")
                onRebootRequested: console.log("darwan: reboot requested (ignored in preview)")
                onSuspendRequested: console.log("darwan: suspend requested (ignored in preview)")

                Component.onCompleted: root.hosts = root.hosts.concat([host])
                Component.onDestruction: root.hosts = root.hosts.filter(h => h !== host)
            }

            Connections {
                target: host.field
                ignoreUnknownSignals: true
                function onTextEdited() { root.mirror(host, host.field.text) }
            }

            InputGate {
                active: root.ambient
                passText: true
                onActivity: root.wake()
            }

            MockAuth {
                id: mock
            }

            Loader {
                id: pamLoader
                active: root.usePam
                source: "contract/PamAuth.qml"
            }

            onClosing: root.quit()
        }
    }

    // Long enough for a video theme to show real frames and for intro animations to finish.
    Timer {
        readonly property var win: windows.count > 0 ? windows.objectAt(0) : null
        interval: 4000
        running: root.shotPath !== "" && win !== null && (win.host.themeReady || win.host.usingFallback)
        onTriggered: win.contentItem.grabToImage(result => {
            if (!result.saveToFile(root.shotPath))
                console.error("darwan: cannot save " + root.shotPath)
            root.quit()
        }, Qt.size(1280, 720))
    }

    Timer {
        id: idle
        interval: root.returnAfter
        running: root.saver && !root.ambient
        onTriggered: {
            if (root.hosts.some(h => h.field && typeof h.field.text === "string" && h.field.text.length > 0))
                restart()
            else
                root.ambient = true
        }
    }
}
