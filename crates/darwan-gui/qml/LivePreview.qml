import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Window
import org.darwan

Rectangle {
    id: pane

    required property Backend backend
    property string themeId: ""
    property string themePath: ""
    property string mode: "lock"
    readonly property bool loaded: hostLoader.item !== null && (hostLoader.item.themeReady || hostLoader.item.usingFallback)
    readonly property bool usingFallback: loaded && hostLoader.item.usingFallback

    readonly property string runtime: "file://" + backend.runtimeDir + "/"
    property QtObject auth: null

    // The Stage shows the theme's still under it while the live theme loads.
    color: "transparent"
    clip: true

    signal unlocked()
    signal failed(string message)

    Component.onCompleted: {
        const c = Qt.createComponent(runtime + "contract/MockAuth.qml")
        if (c.status === Component.Ready)
            auth = c.createObject(pane)
        else
            console.error("darwan-gui: " + c.errorString())
        reload()
    }

    onThemeIdChanged: reload()
    onModeChanged: reload()

    // A playing video must be unloaded before its item goes away, or Qt's FFmpeg backend crashes.
    function unload() {
        if (hostLoader.item)
            hostLoader.item.unload()
    }

    function reload() {
        unload()
        swap.restart()
    }

    // Coalesces quick changes (holding an arrow key, stepping a choice) into one theme load.
    Timer {
        id: swap
        interval: 150
        onTriggered: {
            hostLoader.source = ""
            if (pane.themeId === "" || pane.themePath === "" || pane.auth === null)
                return
            const overlay = pane.backend.writeOverlay(pane.themeId) ? pane.backend.overlayPath : ""
            hostLoader.setSource(pane.runtime + "ThemeHost.qml", {
                themePath: pane.themePath,
                overlayPath: overlay,
                hostMode: pane.mode,
                userName: pane.backend.userName,
                machineName: pane.backend.hostName,
                sessionList: JSON.parse(pane.backend.sessions),
                authBackend: pane.auth,
            })
        }
    }

    // Themes size themselves from Screen, so they render at screen size and are scaled to fit.
    // Built across frames, and the theme's own Loaders with it (they follow an asynchronous parent), so loading a
    // theme doesn't freeze the window.
    Loader {
        id: hostLoader
        asynchronous: true
        width: Screen.width
        height: Screen.height
        transformOrigin: Item.TopLeft
        scale: Math.min(pane.width / width, pane.height / height)
        x: (pane.width - width * scale) / 2
        y: (pane.height - height * scale) / 2
        clip: true
        onStatusChanged: {
            if (status === Loader.Error)
                pane.failed("the preview host failed to load; see the terminal for QML errors")
        }
    }

    Connections {
        target: hostLoader.item
        ignoreUnknownSignals: true
        function onUnlocked() {
            pane.unlocked()
            pane.reload()
        }
    }

    // Every theme grabs keyboard focus when it loads; until the preview is clicked, give it back.
    property bool engaged: false
    property Item fallbackFocus: null
    property Item focusOutside: null
    property Item focusInside: null

    function inside(item) {
        for (let f = item; f; f = f.parent)
            if (f === hostLoader)
                return true
        return false
    }

    function pastPreview(item, forward) {
        let n = item.nextItemInFocusChain(forward)
        while (n && n !== item && inside(n))
            n = n.nextItemInFocusChain(forward)
        return n
    }

    Connections {
        target: pane.Window.window
        function onActiveFocusItemChanged() {
            const f = pane.Window.activeFocusItem
            if (!f)
                return
            if (!pane.inside(f)) {
                pane.engaged = false
                pane.focusOutside = f
                return
            }
            if (f !== hostLoader)
                pane.focusInside = f
            if (pane.engaged)
                return
            const from = pane.focusOutside || pane.fallbackFocus
            if (!from)
                return
            // Tab and Shift+Tab pass over the preview instead of bouncing back from it.
            const target = f === from.nextItemInFocusChain(true) ? pane.pastPreview(f, true)
                         : f === from.nextItemInFocusChain(false) ? pane.pastPreview(f, false)
                         : from
            Qt.callLater(() => target?.forceActiveFocus())
        }
    }

    // Passive and on top, so the theme still gets every press, including the one that focuses its field.
    Item {
        anchors.fill: parent
        z: 1
        TapHandler {
            gesturePolicy: TapHandler.DragThreshold
            grabPermissions: PointerHandler.ApprovesTakeOverByAnything
            onPressedChanged: {
                if (!pressed)
                    return
                pane.engaged = true
                Qt.callLater(() => {
                    if (!pane.inside(pane.Window.activeFocusItem) && pane.focusInside)
                        pane.focusInside.forceActiveFocus()
                })
            }
        }
    }

    // Dropping an image or video on the preview makes it the theme's background.
    DropArea {
        id: drop
        anchors.fill: parent
        z: 2
        readonly property var media: ["png", "jpg", "jpeg", "webp", "bmp", "gif", "mp4", "mkv", "webm", "mov"]
        function pathOf(d) {
            return d.hasUrls ? decodeURIComponent(d.urls[0].toString().replace(/^file:\/\//, "")) : ""
        }
        onEntered: d => d.accepted = media.includes(pathOf(d).split(".").pop().toLowerCase())
        onDropped: d => {
            const path = pathOf(d)
            if (path !== "" && pane.themeId !== "")
                pane.backend.setValue(pane.themeId + ".background", path)
        }
        Rectangle {
            anchors.fill: parent
            visible: drop.containsDrag
            color: Qt.rgba(0, 0, 0, 0.55)
            border.color: Style.accent
            border.width: 2
            radius: pane.radius
            Label {
                anchors.centerIn: parent
                text: "Drop to use as the background"
                color: "white"
                font.pixelSize: 18
            }
        }
    }

}
