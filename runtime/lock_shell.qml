import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "contract"

ShellRoot {
    id: root

    readonly property string themePath: Quickshell.env("DARWAN_THEME_PATH")
    readonly property var sessions: JSON.parse(Quickshell.env("DARWAN_SESSIONS") || "[]")
    readonly property bool testMode: (parseInt(Quickshell.env("DARWAN_UNLOCK_AFTER")) || 0) > 0

    // Only a successful authentication may end the lock; anything else locks again.
    property bool authenticated: false
    // Locking for sleep shows plain black surfaces until the supervisor, or any input, loads the theme.
    property bool themeLoaded: Quickshell.env("DARWAN_FOR_SLEEP") !== "1"
    property var surfaceList: []

    function unlock(windup) {
        if (authenticated)
            return
        authenticated = true
        Quickshell.execDetached(["loginctl", "unlock-session"])
        exitTimer.interval = windup ? 500 : 100
        exitTimer.start()
    }

    function run(action) {
        Quickshell.execDetached(["sh", "-c", "if [ -d /run/systemd/system ]; then systemctl " + action + "; else loginctl " + action + "; fi"])
    }

    function relock() {
        if (relockCheck.running)
            return
        console.warn("darwan: the session lock ended without authentication; locking again")
        Qt.callLater(() => {
            sessionLock.locked = true
            relockCheck.restart()
        })
    }

    Component.onCompleted: sessionLock.locked = true

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
            sessionLock.locked = false
            Qt.callLater(() => Qt.quit())
        }
    }

    // Exit non-zero so the supervisor starts a fresh, locked instance.
    Timer {
        id: relockCheck
        interval: 1000
        onTriggered: {
            if (!sessionLock.secure) {
                console.error("darwan: could not lock again; exiting for a restart")
                Qt.exit(3)
            }
        }
    }

    IpcHandler {
        target: "lock"

        function health(): string {
            return JSON.stringify({
                locked: sessionLock.locked,
                secure: sessionLock.secure,
                authenticated: root.authenticated,
                themeLoaded: root.themeLoaded,
                surfaces: root.surfaceList.map(s => ({ screen: s.screenName, frames: s.frames, focused: s.focused }))
            })
        }

        function resetFrames(): void {
            for (const s of root.surfaceList)
                s.frames = 0
        }

        function loadTheme(): void {
            root.themeLoaded = true
        }

        function unloadTheme(): void {
            root.themeLoaded = false
        }

        // Test locks only: end the lock as a compositor would, to exercise the relock path.
        function dropLock(): string {
            if (!root.testMode)
                return "refused: not a test lock"
            sessionLock.locked = false
            return "dropped"
        }
    }

    WlSessionLock {
        id: sessionLock

        onLockedChanged: {
            if (!locked && !root.authenticated)
                root.relock()
        }

        WlSessionLockSurface {
            id: surface
            color: "black"

            readonly property string screenName: screen ? screen.name : ""
            property int frames: 0
            readonly property bool focused: probe.Window.active

            Component.onCompleted: root.surfaceList = root.surfaceList.concat([surface])
            Component.onDestruction: root.surfaceList = root.surfaceList.filter(s => s !== surface)

            Item {
                id: probe
                anchors.fill: parent

                Connections {
                    target: probe.Window.window
                    function onFrameSwapped() { surface.frames++ }
                }
            }

            Loader {
                id: themeLoader
                anchors.fill: parent
                active: root.themeLoaded
                focus: true
                sourceComponent: ThemeHost {
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

            // With no theme loaded there is no password field, so input brings the theme back. The pointer
            // entering a freshly mapped surface is not input: only real movement counts.
            MouseArea {
                property point origin: Qt.point(-1, -1)
                anchors.fill: parent
                enabled: !root.themeLoaded
                visible: enabled
                hoverEnabled: true
                onPositionChanged: mouse => {
                    if (origin.x < 0)
                        origin = Qt.point(mouse.x, mouse.y)
                    else if (Math.hypot(mouse.x - origin.x, mouse.y - origin.y) > 8)
                        root.themeLoaded = true
                }
                onPressed: root.themeLoaded = true
                onWheel: root.themeLoaded = true
            }
            Item {
                anchors.fill: parent
                focus: !root.themeLoaded
                Keys.onPressed: root.themeLoaded = true
            }
        }
    }
}
