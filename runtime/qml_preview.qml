import QtQuick
import QtQuick.Window
import "contract"

// Qt's qml runner hosts `preview --at` because Quickshell hangs under libfaketime. The root is not an Item, so the
// runner adds no window of its own; every screen gets one, as the lock shows it.
QtObject {
    id: root

    readonly property var opts: JSON.parse(Qt.application.arguments[Qt.application.arguments.length - 1])
    property var hosts: []

    function quit() {
        for (const h of hosts)
            h.unload()
        Qt.callLater(() => Qt.quit())
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
    property Instantiator windows: Instantiator {
        model: Qt.application.screens

        delegate: Window {
            id: win

            required property var modelData

            screen: modelData
            x: modelData.virtualX
            y: modelData.virtualY
            width: modelData.width
            height: modelData.height
            visible: true
            title: "darwan preview: " + root.opts.themeId
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
                themePath: root.opts.themePath
                overlayPath: root.opts.overlay
                hostMode: root.opts.mode
                userName: root.opts.user || "user"
                machineName: root.opts.hostName || ""
                sessionList: root.opts.sessions || []
                authBackend: MockAuth {}
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

            onClosing: root.quit()
        }
    }
}
