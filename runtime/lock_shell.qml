import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import Darwan
import "contract"

// One process for the screensaver and the lock. The saver draws each screen's theme on an overlay layer above the
// desktop; locking moves the same, already running theme into the lock surface, so nothing reloads or flashes.
ShellRoot {
    id: root

    readonly property string themePath: Quickshell.env("DARWAN_THEME_PATH")
    readonly property var sessions: JSON.parse(Quickshell.env("DARWAN_SESSIONS") || "[]")
    readonly property bool testMode: (parseInt(Quickshell.env("DARWAN_UNLOCK_AFTER")) || 0) > 0
    // "lock": widgets at once; "saver": ambient over the desktop; "sleep": black lock surfaces until wake or input.
    readonly property string startMode: Quickshell.env("DARWAN_START") || "lock"
    // Milliseconds from the saver showing to the lock; negative: never.
    readonly property int lockAfter: {
        const v = parseInt(Quickshell.env("DARWAN_LOCK_AFTER"))
        return isNaN(v) ? 0 : v
    }
    readonly property string mediaTier: Quickshell.env("DARWAN_MEDIA_TIER") || "full"

    // Only a successful authentication may end the lock; anything else locks again.
    property bool authenticated: false
    property bool themeLoaded: startMode !== "sleep"
    property bool lockWanted: startMode !== "saver"
    property bool ambient: startMode === "saver"
    // The unlocked saver: "loading" (drawn invisibly, input still goes to the desktop), "shown", "leaving"; "" once locked.
    property string saverPhase: startMode === "saver" ? "loading" : ""
    // Hidden widgets are drawn once while loading, so the first reveal doesn't stall on shaders and textures.
    property bool warming: startMode === "saver"
    // Locking a shown saver moves its live theme into the lock surface. Each overlay first shows a still of itself, so
    // it is never empty while the compositor switches to the lock (the desktop would show). The theme then leaves the
    // overlay, which draws one more frame to free its video renderer: libmpv allows one render context per player.
    property bool frozen: false
    property int stills: 0
    property bool released: false
    property int releasedWindows: 0
    // screen name -> the overlay's still; the lock surface shows it until the moved video draws again.
    property var stillUrls: ({})
    readonly property bool lockNow: lockWanted && (released || (saverPhase !== "shown" && saverPhase !== "leaving"))
    property var surfaceList: []
    property var saverSlots: ({})
    property var lockSlots: ({})
    property var hosts: []

    // Exit codes the supervisor reads: 0 authenticated, 4 the unlocked saver ended, anything else restarts it locked.
    function leave(code) {
        Qt.callLater(() => Qt.exit(code))
    }

    function setSlot(map, name, item) {
        const next = Object.assign({}, map)
        if (item)
            next[name] = item
        else
            delete next[name]
        return next
    }

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

    // Input on a saver or lock surface.
    function userActive() {
        if (saverPhase === "shown" && !lockWanted) {
            saverPhase = "leaving"
            return
        }
        if (!themeLoaded)
            themeLoaded = true
        ambient = false
        ambientTimer.restart()
    }

    function allReady() {
        return hosts.length > 0 && hosts.every(h => h.item && (h.item.themeReady || h.item.usingFallback))
    }

    function fieldHasText() {
        for (const h of hosts) {
            const f = h.Window.activeFocusItem
            if (f && typeof f.text === "string" && f.text.length > 0)
                return true
        }
        return false
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
            sessionLock.locked = false
            root.leave(0)
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

    // Loading: wait for every screen's theme, draw it revealed for a moment, then hide the widgets and fade in.
    Timer {
        interval: 50
        repeat: true
        running: root.saverPhase === "loading" && root.warming
        onTriggered: {
            if (root.allReady()) {
                stop()
                warmTimer.start()
            }
        }
    }
    Timer {
        id: warmTimer
        interval: 120
        onTriggered: {
            root.warming = false
            showTimer.start()
        }
    }
    // Lets the widgets start hiding before the saver fades in over them.
    Timer {
        id: showTimer
        interval: 300
        onTriggered: if (root.saverPhase === "loading") root.saverPhase = "shown"
    }
    Timer {
        interval: Math.max(0, root.lockAfter)
        running: root.saverPhase === "shown" && root.lockAfter >= 0 && !root.lockWanted
        onTriggered: root.lockWanted = true
    }
    Timer {
        interval: 350
        running: root.saverPhase === "leaving"
        onTriggered: root.leave(4)
    }
    // A revealed lock settles back to ambient when nobody touches it, unless something is typed.
    Timer {
        id: ambientTimer
        interval: 30000
        running: root.lockWanted && root.themeLoaded && !root.ambient && !root.authenticated
        onTriggered: {
            if (root.fieldHasText())
                restart()
            else
                root.ambient = true
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
                saver: root.saverPhase,
                ambient: root.ambient,
                surfaces: root.surfaceList.map(s => ({ screen: s.screenName, frames: s.frames, focused: s.focused }))
            })
        }

        // A black surface with nothing animating would never draw again, so each gets one frame to prove it can.
        function resetFrames(): void {
            for (const s of root.surfaceList) {
                s.frames = 0
                s.redraw()
            }
        }

        // After wake or outputs coming back: the widgets show, never the saver.
        function loadTheme(): void {
            root.ambient = false
            root.themeLoaded = true
        }

        function unloadTheme(): void {
            root.themeLoaded = false
        }

        function lock(): void {
            root.lockWanted = true
        }

        // User activity the supervisor saw while the saver was still loading and not yet taking input. Once the saver
        // shows, its own surfaces judge input (pointer jitter included).
        function activity(): string {
            if (root.saverPhase !== "loading")
                return "ignored"
            if (!root.lockWanted) {
                root.leave(4)
                return "cancelled"
            }
            root.userActive()
            return "revealed"
        }

        // Test locks only: end the lock as a compositor would, to exercise the relock path.
        function dropLock(): string {
            if (!root.testMode)
                return "refused: not a test lock"
            sessionLock.locked = false
            return "dropped"
        }
    }

    function stillTaken() {
        stills++
        if (stills >= Quickshell.screens.length)
            frozen = true
    }

    function windowReleased() {
        releasedWindows++
        if (releasedWindows >= Quickshell.screens.length)
            released = true
    }

    // A still or a frame that doesn't arrive must not hold the lock back.
    Timer {
        interval: 300
        running: root.lockWanted && !root.frozen && (root.saverPhase === "shown" || root.saverPhase === "leaving")
        onTriggered: root.frozen = true
    }
    Timer {
        interval: 200
        running: root.frozen && !root.released
        onTriggered: root.released = true
    }

    // Each screen's theme. It lives outside any window and is shown in the lock surface once there is one, else in
    // the saver overlay.
    Variants {
        model: Quickshell.screens

        Loader {
            id: slotContent

            required property var modelData
            readonly property string screenName: modelData.name

            parent: root.lockSlots[screenName] || (root.frozen ? null : root.saverSlots[screenName]) || null
            anchors.fill: parent
            visible: parent !== null
            focus: true
            active: root.themeLoaded
            onParentChanged: if (parent) Qt.callLater(() => { if (item) item.refocus() })

            Component.onCompleted: root.hosts = root.hosts.concat([slotContent])
            Component.onDestruction: root.hosts = root.hosts.filter(h => h !== slotContent)

            sourceComponent: ThemeHost {
                themePath: root.themePath
                overlayPath: Quickshell.env("DARWAN_OVERLAY") || ""
                hostMode: "lock"
                userName: Quickshell.env("DARWAN_USER") || Quickshell.env("USER") || ""
                userRealName: Quickshell.env("DARWAN_REAL_NAME") || userName
                sessionList: root.sessions
                authBackend: PamAuth {}
                ambient: root.ambient && !root.warming
                mediaTier: root.mediaTier
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

    Region {
        id: noInput
    }

    Variants {
        model: root.startMode === "saver" ? Quickshell.screens : []

        PanelWindow {
            id: saverWindow

            required property var modelData
            readonly property bool taking: root.saverPhase === "shown"

            screen: modelData
            visible: root.saverPhase !== ""
            anchors { left: true; right: true; top: true; bottom: true }
            exclusionMode: ExclusionMode.Ignore
            color: "transparent"
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.namespace: "darwan-saver"
            WlrLayershell.keyboardFocus: taking ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None
            mask: taking ? null : noInput

            FocusScope {
                id: saverSlot
                anchors.fill: parent
                focus: true
                opacity: root.saverPhase === "shown" || (root.saverPhase === "" && root.lockWanted) ? 1 : root.saverPhase === "loading" ? 0.004 : 0
                Behavior on opacity {
                    NumberAnimation { duration: root.saverPhase === "leaving" ? 300 : 800; easing.type: Easing.InOutQuad }
                }
                Component.onCompleted: root.saverSlots = root.setSlot(root.saverSlots, saverWindow.modelData.name, saverSlot)
                Component.onDestruction: root.saverSlots = root.setSlot(root.saverSlots, saverWindow.modelData.name, null)
            }

            Connections {
                target: root.frozen && !root.released ? saverSlot.Window.window : null
                function onFrameSwapped() { root.windowReleased() }
            }

            Image {
                id: still
                property var grab: null
                anchors.fill: parent
                visible: root.frozen && source != ""
            }
            Connections {
                target: root
                function onLockWantedChanged() {
                    if (root.lockWanted && (root.saverPhase === "shown" || root.saverPhase === "leaving"))
                        saverSlot.grabToImage(result => {
                            still.grab = result
                            still.source = result.url
                            root.stillUrls = root.setSlot(root.stillUrls, saverWindow.modelData.name, result.url)
                            root.stillTaken()
                        })
                }
            }

            InputGate {
                active: saverWindow.taking
                onActivity: if (saverWindow.taking) root.userActive()
            }
        }
    }

    WlSessionLock {
        id: sessionLock

        locked: root.lockNow
        onLockedChanged: {
            if (!locked && !root.authenticated)
                root.relock()
        }
        // The overlay is hidden under the lock from here on; drop it.
        onSecureChanged: {
            if (secure) {
                root.saverPhase = ""
                root.warming = false
            }
        }

        WlSessionLockSurface {
            id: surface
            color: "black"

            readonly property string screenName: screen ? screen.name : ""
            property int frames: 0
            readonly property bool focused: lockSlot.Window.active

            function redraw() {
                if (lockSlot.Window.window)
                    lockSlot.Window.window.update()
            }

            Component.onCompleted: root.surfaceList = root.surfaceList.concat([surface])
            Component.onDestruction: {
                root.surfaceList = root.surfaceList.filter(s => s !== surface)
                if (root.lockSlots[screenName] === lockSlot)
                    root.lockSlots = root.setSlot(root.lockSlots, screenName, null)
            }
            onScreenNameChanged: if (screenName !== "") root.lockSlots = root.setSlot(root.lockSlots, screenName, lockSlot)

            FocusScope {
                id: lockSlot
                anchors.fill: parent
                focus: true

                Connections {
                    target: lockSlot.Window.window
                    function onFrameSwapped() { surface.frames++ }
                }
            }

            Image {
                id: lockStill
                anchors.fill: parent
                source: root.stillUrls[surface.screenName] || ""
                visible: source != "" && opacity > 0
                Behavior on opacity { NumberAnimation { duration: 150 } }
                Timer {
                    interval: 400
                    running: lockStill.source != ""
                    onTriggered: lockStill.opacity = 0
                }
            }

            // Ambient: printable keys go on to the hidden password field, so typing the password straight away works.
            InputGate {
                active: root.ambient || !root.themeLoaded
                passText: root.themeLoaded
                onActivity: root.userActive()
            }
        }
    }
}
