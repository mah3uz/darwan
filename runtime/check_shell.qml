import QtQuick
import QtQuick.Window
import Quickshell
import "contract"

ShellRoot {
    id: root

    readonly property string shotPath: Quickshell.env("DARWAN_SHOT") || ""
    readonly property bool checkFonts: Quickshell.env("DARWAN_CHECK_FONTS") === "1"
    readonly property int settleMs: parseInt(Quickshell.env("DARWAN_SETTLE_MS")) || 3000

    Window {
        id: win
        visible: true
        width: Screen.width
        height: Screen.height
        color: "black"

        ThemeHost {
            id: host
            anchors.fill: parent
            themePath: Quickshell.env("DARWAN_THEME_PATH")
            overlayPath: Quickshell.env("DARWAN_OVERLAY") || ""
            hostMode: Quickshell.env("DARWAN_MODE") || "lock"
            userName: Quickshell.env("DARWAN_USER") || "traveler"
            machineName: "darwan"
            sessionList: JSON.parse(Quickshell.env("DARWAN_SESSIONS") || "[]")
            authBackend: MockAuth {}
        }
    }

    // Qt.exit is ignored until the engine is fully up, so the verdict comes from a timer.
    Timer {
        interval: root.settleMs
        running: host.themeReady || host.usingFallback
        onTriggered: root.finish()
    }

    function textItemsWithoutFont(item, out) {
        if (!item || !item.visible)
            return
        if (typeof item.text === "string" && item.font !== undefined && item.font.family === "")
            out.push((item.objectName || item.toString()) + " \"" + item.text.slice(0, 30) + "\"")
        for (var i = 0; i < item.children.length; i++)
            textItemsWithoutFont(item.children[i], out)
    }

    function finish() {
        var code = host.usingFallback ? 2 : 0
        if (code === 0 && checkFonts) {
            var bad = []
            textItemsWithoutFont(host.themeItem, bad)
            if (bad.length > 0) {
                console.error("darwan: " + bad.length + " text items have no font family: " + bad.join(", "))
                code = 3
            }
        }
        if (shotPath === "") {
            done(code)
            return
        }
        win.contentItem.grabToImage(result => {
            result.saveToFile(root.shotPath)
            root.done(code)
        })
    }

    // Quitting with a video still playing crashes Qt's FFmpeg backend.
    function done(code) {
        host.unload()
        Qt.callLater(() => Qt.exit(code))
    }
}
