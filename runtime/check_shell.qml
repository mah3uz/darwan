import QtQuick
import QtQuick.Window
import QtTest
import Quickshell
import "contract"

ShellRoot {
    id: root

    readonly property string shotPath: Quickshell.env("DARWAN_SHOT") || ""
    readonly property bool checkFonts: Quickshell.env("DARWAN_CHECK_FONTS") === "1"
    readonly property int settleMs: parseInt(Quickshell.env("DARWAN_SETTLE_MS")) || 3000
    readonly property bool checkLogin: Quickshell.env("DARWAN_CHECK_LOGIN") === "1"

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
            onUnlocked: {
                if (loginTimer.running) {
                    loginTimer.stop()
                    root.done(0)
                }
            }
        }

        // Real key events, so each theme submits the way it does for a person.
        TestCase {
            id: keys
            when: false
            optional: true
        }
    }

    // Long enough for animated submits such as clockwork's windup (about 1.6 s before login).
    Timer {
        id: loginTimer
        interval: 6000
        onTriggered: {
            console.error("darwan: typing the password and pressing Return did not unlock")
            root.done(4)
        }
    }

    // A person first presses a key to wake "press to start" screens, then types.
    function tryLogin() {
        keys.keyClick(Qt.Key_Return)
        typeTimer.start()
    }

    Timer {
        id: typeTimer
        interval: 1000
        onTriggered: {
            for (const k of [Qt.Key_T, Qt.Key_E, Qt.Key_S, Qt.Key_T, Qt.Key_Return])
                keys.keyClick(k)
            loginTimer.start()
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
        const next = () => (code === 0 && checkLogin) ? tryLogin() : done(code)
        if (shotPath === "") {
            next()
            return
        }
        win.contentItem.grabToImage(result => {
            result.saveToFile(root.shotPath)
            next()
        })
    }

    // Quitting with a video still playing crashes Qt's FFmpeg backend.
    function done(code) {
        host.unload()
        Qt.callLater(() => Qt.exit(code))
    }
}
