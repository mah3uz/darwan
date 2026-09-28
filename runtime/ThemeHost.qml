import QtQuick
import "contract/Ini.js" as Ini

// Themes find sddm, config, userModel, sessionModel and keyboard through this root's scope.
Item {
    id: host

    property string themePath
    property string overlayPath
    property string hostMode: "lock"
    property string userName: "user"
    property string userRealName: userName
    property string machineName: ""
    property var sessionList: []
    property QtObject authBackend
    property int loadTimeout: 8000
    // Screensaver: the theme shows only its background and ambient animation. Unset under SDDM, where themes stay revealed.
    property bool ambient: false
    // "full", "eco" or "still": what darwan's VideoOutput plays (docs/theme-contract.md, "Screensaver").
    property string mediaTier: "full"

    readonly property bool themeReady: themeLoader.status === Loader.Ready
    readonly property bool usingFallback: (configReady && themePath === "") || themeLoader.status === Loader.Error || loadTimer.expired
    readonly property Item themeItem: themeLoader.item

    signal unlocked()
    signal powerOffRequested()
    signal rebootRequested()
    signal suspendRequested()

    property var config: ({})
    property bool configReady: false
    property bool unloading: false

    function unload() {
        unloading = true
    }

    function inTheme(f) {
        const item = themeLoader.item
        while (f && f !== item)
            f = f.parent
        return item !== null && f === item
    }

    // Themes that mark their own field `focus: true` keep it, as they do under SDDM. After the host moves to another
    // window (saver to lock), the field the theme had focused gets it back.
    function refocus() {
        const item = themeLoader.item
        if (!item || inTheme(themeLoader.Window.activeFocusItem))
            return
        if (lastFocus && inTheme(lastFocus))
            lastFocus.forceActiveFocus()
        else
            item.forceActiveFocus()
    }

    property Item lastFocus: null
    // The theme's focused input (its password field), also for a host that isn't the focused window.
    readonly property Item field: inTheme(themeLoader.Window.activeFocusItem) ? themeLoader.Window.activeFocusItem : lastFocus
    Connections {
        target: themeLoader.Window.window
        function onActiveFocusItemChanged() {
            const f = themeLoader.Window.activeFocusItem
            if (f && host.inTheme(f))
                host.lastFocus = f
        }
    }

    readonly property QtObject darwan: QtObject {
        readonly property bool ambient: host.ambient
    }

    readonly property QtObject sddm: QtObject {
        signal loginSucceeded()
        signal loginFailed()
        signal informationMessage(string message)

        // 38 themes treat an undefined hostName as "running on the lockscreen".
        readonly property var hostName: host.hostMode === "sddm" ? host.machineName : undefined
        readonly property bool canPowerOff: true
        readonly property bool canReboot: true
        readonly property bool canSuspend: true
        readonly property bool canHibernate: false
        readonly property bool canHybridSleep: false

        // The lock only ever authenticates the session owner, whatever user the theme picked.
        function login(user, password, sessionIndex) {
            host.authBackend.authenticate(host.hostMode === "lock" ? host.userName : user, password)
        }
        function powerOff() { host.powerOffRequested() }
        function reboot() { host.rebootRequested() }
        function suspend() { host.suspendRequested() }
        function hibernate() {}
        function hybridSleep() {}
    }

    readonly property ListModel userModel: ListModel {
        property string lastUser: host.userName
        property int lastIndex: 0
        property int disableAvatarsThreshold: 7
        property bool containsAllUsers: true

        function rowCount() { return count }
        function index(row, column) { return row }
        function data(row, role) {
            var u = get(row)
            if (!u)
                return undefined
            switch (role) {
            case Qt.UserRole + 2: return u.realName
            case Qt.UserRole + 3: return u.homeDir
            case Qt.UserRole + 4: return u.icon
            case Qt.UserRole + 5: return u.needsPassword
            default: return u.name
            }
        }
    }

    readonly property ListModel sessionModel: ListModel {
        property int lastIndex: 0

        function rowCount() { return count }
        function index(row, column) { return row }
        function data(row, role) {
            var s = get(row)
            if (!s)
                return undefined
            switch (role) {
            case Qt.UserRole + 2: return s.file
            case Qt.UserRole + 3: return s.type
            case Qt.UserRole + 5: return s.exec
            case Qt.UserRole + 6: return s.comment
            default: return s.name
            }
        }
    }

    readonly property QtObject keyboard: QtObject {
        property bool numLock: false
        property bool capsLock: false
        property var layouts: []
        property int currentLayout: 0
        property bool enabled: false
    }

    Connections {
        target: host.authBackend
        function onSucceeded() {
            host.sddm.loginSucceeded()
            host.unlocked()
        }
        function onFailed() { host.sddm.loginFailed() }
    }

    function readFile(path, done) {
        var xhr = new XMLHttpRequest()
        xhr.onreadystatechange = function() {
            if (xhr.readyState === XMLHttpRequest.DONE)
                done(xhr.status === 200 || xhr.status === 0 ? xhr.responseText : null)
        }
        try {
            xhr.open("GET", "file://" + path)
            xhr.send()
        } catch (e) {
            console.warn("darwan: cannot read " + path + ": " + e)
            done(null)
        }
    }

    // A config error must never stop the lock from loading: fall back to what was read.
    function loadConfig() {
        if (themePath === "") {
            configReady = true
            return
        }
        readFile(themePath + "/theme.conf", function(base) {
            if (base === null)
                console.warn("darwan: cannot read " + themePath + "/theme.conf")
            var merged = Ini.parseGeneral(base)
            if (overlayPath === "") {
                host.config = merged
                host.configReady = true
                return
            }
            readFile(overlayPath, function(overlay) {
                if (overlay === null)
                    console.warn("darwan: cannot read overlay " + overlayPath)
                host.config = Ini.merge(merged, Ini.parseGeneral(overlay))
                host.configReady = true
            })
        })
    }

    Component.onCompleted: {
        userModel.append({ name: userName, realName: userRealName, homeDir: "", icon: "", needsPassword: true })
        for (var i = 0; i < sessionList.length; i++) {
            var s = sessionList[i]
            sessionModel.append({ name: s.name || "", file: s.file || "", type: s.type || "wayland", exec: s.exec || "", comment: s.comment || "" })
        }
        if (sessionModel.count === 0)
            sessionModel.append({ name: "Session", file: "", type: "wayland", exec: "", comment: "" })
        loadConfig()
    }

    // Qt never answers a file XHR when QML_XHR_ALLOW_FILE_READ is unset; don't wait forever.
    Timer {
        interval: 2000
        running: !host.configReady
        onTriggered: {
            console.warn("darwan: config not readable; using the theme without it")
            host.configReady = true
        }
    }

    Loader {
        id: themeLoader
        anchors.fill: parent
        focus: true
        active: host.configReady && !host.unloading && host.themePath !== ""
        source: active ? "file://" + host.themePath + "/Main.qml" : ""
        onLoaded: Qt.callLater(host.refocus)
        onStatusChanged: {
            if (status === Loader.Error)
                console.error("darwan: failed to load " + source)
        }
    }

    Timer {
        id: loadTimer
        property bool expired: false
        interval: host.loadTimeout
        running: host.configReady && !host.themeReady && !host.unloading
        onTriggered: {
            expired = true
            console.error("darwan: theme did not load within " + interval + " ms")
        }
    }

    FallbackPrompt {
        id: fallback
        anchors.fill: parent
        visible: host.usingFallback && !host.unloading
        userName: host.userName
        onSubmit: password => host.sddm.login(host.userName, password, 0)
        Connections {
            target: host.sddm
            function onLoginFailed() { if (fallback.visible) fallback.rejected() }
        }
    }
}
