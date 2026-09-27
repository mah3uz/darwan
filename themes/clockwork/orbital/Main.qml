import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import Qt.labs.folderlistmodel
import SddmComponents 2.0
import "darwan"

Rectangle {
    // Cursor
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }
    id: root
    readonly property real s: Screen.height / 768
    width: Screen.width
    height: Screen.height
    color: root.bgColor

    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined

    Custom { id: kit }
    Background { id: userBg; anchors.fill: parent }

    // Config
    readonly property string themeMode: config.colorScheme || "dark"
    readonly property bool enableWindup: config.enableWindup !== "false" && config.enableWindup !== false
    readonly property bool isLight: themeMode === "light"

    readonly property color bgColor: isLight ? "#ffffff" : "#000000"
    readonly property color mainText: kit.color("text", isLight ? "#000000" : "#ffffff")
    readonly property color accent: kit.color("accent", isLight ? "#000000" : "#ffffff")
    readonly property color dimText: isLight ? "#666666" : "#666666"
    readonly property color subColor: isLight ? "#666666" : "#555555"
    readonly property color pillColor: isLight ? "#e8e8e8" : "#080808"
    readonly property color pillBorder: isLight ? (root.isWindup ? "#aaaaaa" : "#cccccc") : (root.isWindup ? "#444" : "#1a1a1a")
    readonly property color pillInnerLine: isLight ? (root.isWindup ? "#000000" : "#bbbbbb") : (root.isWindup ? "#ffffff" : "#222222")
    readonly property color sparkColor: root.accent
    readonly property color blastColor: isLight ? "#000000" : "#ffffff"
    readonly property color userItemInactive: isLight ? "#cccccc" : "#444"
    readonly property color inputWaitColor: isLight ? "#bbbbbb" : "#333333"

    // State
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property bool userMenuOpen: false
    property bool isWindup: false
    property real uiOpacity: 0
    readonly property real marginR: 80 * s

    // Time
    property int curH: new Date().getHours()
    readonly property bool clock12: config.clockFormat === "12h"
    readonly property string amPm: clock12 && config.clockShowAmPm === "true" ? (curH < 12 ? "AM" : "PM") : ""
    function clockHour(h) { return String(clock12 ? h % 12 || 12 : h).padStart(2, "0") }
    property int curM: new Date().getMinutes()
    // Milliseconds into the day, swept by one native animation instead of a script every frame.
    property real localTimeMS: 0
    NumberAnimation { id: clockSweep; target: root; property: "localTimeMS"; duration: 86400000 }

    function syncClock() {
        var d = new Date()
        root.curH = d.getHours(); root.curM = d.getMinutes()
        var ms = ((d.getHours() * 60 + d.getMinutes()) * 60 + d.getSeconds()) * 1000 + d.getMilliseconds()
        clockSweep.stop()
        root.localTimeMS = ms
        // With reduced motion the dial steps once a second instead of sweeping.
        if (!kit.reduceMotion) {
            clockSweep.from = ms
            clockSweep.to = ms + 86400000
            clockSweep.start()
        }
    }

    Timer {
        interval: 1000; running: true; repeat: true
        onTriggered: {
            var d = new Date()
            root.curH = d.getHours(); root.curM = d.getMinutes()
            if (kit.reduceMotion || d.getSeconds() === 0)
                root.syncClock()
        }
    }

    // Animations
    property real windupOffset: 0
    property real windupProgress: windupOffset / 150000
    property real boomScale: 1.0
    property real boomOpacity: 0.0
    property real jitterX: 0
    property real jitterY: 0
    property real sparkIntensity: 0.0

    Timer {
        interval: 16; running: root.isWindup; repeat: true
        onTriggered: {
            var intensity = root.windupProgress * 32 * s
            root.jitterX = (Math.random() - 0.5) * intensity
            root.jitterY = (Math.random() - 0.5) * intensity
            root.sparkIntensity = root.windupProgress > 0.2 ? (root.windupProgress - 0.2) * 2.2 : 0
        }
    }

    NumberAnimation { id: windupAnim; target: root; property: "windupOffset"; from: 0; to: 150000; duration: kit.dur(1600); easing.type: Easing.InQuint }

    ParallelAnimation {
        id: boomSequence
        onFinished: root.doLogin()
        NumberAnimation { target: root; property: "boomScale"; to: 35.0; duration: kit.dur(150); easing.type: Easing.InQuad }
        NumberAnimation { target: root; property: "boomOpacity"; to: 1.0; duration: kit.dur(120); easing.type: Easing.InQuad }
    }

    readonly property real smoothSecAngle: -((localTimeMS % 60000) / 60000.0) * 360.0 - windupOffset * 10.0
    readonly property real smoothMinAngle: -((localTimeMS % 3600000) / 3600000.0) * 360.0 - windupOffset * 5.0

    // Fonts
    FolderListModel { showDirs: false; id: fontFolder; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
    FontLoader { id: outfitFont; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }
    readonly property string outfitFontFamily: kit.font("text", outfitFont.status === FontLoader.Ready ? outfitFont.name : "sans-serif")
    readonly property string clockFontFamily: kit.font("clock", outfitFont.status === FontLoader.Ready ? outfitFont.name : "sans-serif")
    TextConstants { id: textConstants }

    // Models
    ListView {
        id: userHelper; width: 1; height: 1; opacity: 0; currentIndex: root.userIndex
        model: typeof userModel !== "undefined" ? userModel : null
        delegate: Item { property string uName: model.realName || model.name || ""; property string uLogin: model.name || "" }
    }
    ListView {
        id: sessionHelper; width: 1; height: 1; opacity: 0; currentIndex: root.sessionIndex
        model: typeof sessionModel !== "undefined" ? sessionModel : null
        delegate: Item { property string sName: model.name || "" }
    }

    // Focus
    Timer { interval: 300; running: true; onTriggered: passInput.forceActiveFocus() }

    Component.onCompleted: { syncClock(); fadeIn.start(); keyboard.numLock = true }
    NumberAnimation { id: fadeIn; target: root; property: "uiOpacity"; to: 1; duration: kit.dur(350); easing.type: kit.ease(Easing.OutCubic) }

    // Layout
    Item {
        id: blastContainer
        anchors.fill: parent; opacity: root.uiOpacity
        x: root.jitterX; y: root.jitterY
        transform: Scale { origin.x: 400 * s; origin.y: blastContainer.height * 0.5; xScale: root.boomScale; yScale: root.boomScale }

        Item {
            id: clockContainer
            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
            width: 800 * s; height: parent.height
            readonly property real cx: 40 * s 
            readonly property real cy: height * 0.5
            readonly property real minR: 320 * s 
            readonly property real secR: 480 * s 

            Rectangle {
                id: indicatorPill; z: 1; x: clockContainer.cx + 230 * s; anchors.verticalCenter: parent.verticalCenter
                width: 330 * s; height: 90 * s; radius: 45 * s; color: root.pillColor; border.color: root.pillBorder; border.width: 1 * s
                Rectangle { x: 170 * s; anchors.verticalCenter: parent.verticalCenter; width: 1 * s; height: 35 * s; color: root.pillInnerLine }
            }

            Repeater {
                model: 60
                delegate: Rectangle {
                    z: 50; property real randA: Math.random() * 6.28; property real randV: 400 * s + Math.random() * 900 * s
                    x: (clockContainer.cx + 400 * s) + Math.cos(randA) * (randV * root.sparkIntensity)
                    y: (clockContainer.cy) + Math.sin(randA) * (randV * root.sparkIntensity)
                    width: (1 + Math.random() * 2) * s; height: (1 + 12 * root.sparkIntensity) * s 
                    rotation: randA * 180 / Math.PI + 90; radius: width / 2; color: root.sparkColor
                    opacity: root.sparkIntensity * (Math.random() > 0.4 ? 1.0 : 0.2); visible: root.sparkIntensity > 0
                }
            }

            OrbitalRing { angle: root.smoothMinAngle; radius: clockContainer.minR; numberInset: 35 * s; majorTick: 18 * s; minorTick: 10 * s; majorWidth: 2 * s; numberSize: 22 * s }
            OrbitalRing { angle: root.smoothSecAngle; radius: clockContainer.secR; numberInset: 30 * s; majorTick: 13 * s; minorTick: 8 * s; majorWidth: 1.5 * s; numberSize: 16 * s }

            Text {
                anchors.right: indicatorPill.left; anchors.rightMargin: 40 * s; anchors.verticalCenter: parent.verticalCenter
                text: root.clockHour(root.curH); font.family: root.clockFontFamily; font.pixelSize: 110 * s; font.weight: Font.Black; color: root.mainText
            }
            Column {
                anchors.left: indicatorPill.right; anchors.leftMargin: 110 * s; anchors.verticalCenter: parent.verticalCenter; spacing: 5 * s
                Text { text: Qt.formatDate(new Date(), config.dateFormatNoWeekday || "dd MMM yyyy").toUpperCase() + (root.amPm ? " · " + root.amPm : ""); font.family: outfitFontFamily; font.pixelSize: 13 * s; font.letterSpacing: 4 * s; color: root.subColor }
                Text { text: Qt.formatDate(new Date(), "dddd").toUpperCase(); font.family: outfitFontFamily; font.pixelSize: 18 * s; font.letterSpacing: 8 * s; font.weight: Font.Bold; color: root.mainText }
            }
        }
    }

    // Flash
    Rectangle { anchors.fill: parent; color: root.blastColor; opacity: root.boomOpacity; z: 9999 }

    // HUD
    Item {
        id: hudContainer; anchors.fill: parent; opacity: root.uiOpacity * (root.boomOpacity > 0 ? 0 : 1)
        Row {
            anchors.right: parent.right; anchors.rightMargin: root.marginR; anchors.top: parent.top; anchors.topMargin: 50 * s; spacing: 25 * s
            CwAction { visible: !root.isQuickshell; label: (sessionHelper.currentItem ? sessionHelper.currentItem.sName : "Session"); onClicked: { if (typeof sessionModel !== "undefined") root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount() } }
            Rectangle { visible: !root.isQuickshell; width: 1 * s; height: 10 * s; color: root.pillBorder; anchors.verticalCenter: parent.verticalCenter }
            CwAction { label: "Reboot"; onClicked: { if (typeof sddm !== "undefined") sddm.reboot() } }
            Rectangle { width: 1 * s; height: 10 * s; color: root.pillBorder; anchors.verticalCenter: parent.verticalCenter }
            CwAction { label: "Shutdown"; onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() } }
        }

        Item {
            id: uMenuContainer; z: 6000
            anchors.bottom: loginPanel.top; anchors.bottomMargin: 15 * s; anchors.right: loginPanel.right; width: 280 * s
            height: root.userMenuOpen ? ((typeof userModel !== "undefined" ? userModel.rowCount() : 0) * 30 * s) + 20 * s : 0; clip: true
            Behavior on height { NumberAnimation { duration: kit.dur(400); easing.type: kit.ease(Easing.OutExpo) } }
            Column {
                anchors.bottom: parent.bottom; anchors.right: parent.right; spacing: 6 * s
                Repeater {
                    model: typeof userModel !== "undefined" ? userModel : null
                    delegate: Item {
                        width: 260 * s; height: 26 * s; property bool itemHover: uItemMa.containsMouse
                        Text {
                            id: uItemTxt; text: (model.realName || model.name || "").toUpperCase(); font.family: outfitFontFamily; font.pixelSize: 13 * s; font.letterSpacing: 2 * s; color: (root.userIndex === index || itemHover) ? root.mainText : root.userItemInactive; anchors.right: parent.right; anchors.rightMargin: itemHover ? 30 * s : 10 * s; anchors.verticalCenter: parent.verticalCenter; Behavior on anchors.rightMargin { NumberAnimation { duration: kit.dur(200) } }
                        }
                        Text { text: "✦"; anchors.left: uItemTxt.right; anchors.leftMargin: 8 * s; anchors.verticalCenter: parent.verticalCenter; color: root.accent; opacity: itemHover ? 1.0 : 0; font.pixelSize: 10 * s; Behavior on opacity { NumberAnimation { duration: kit.dur(200) } } }
                        MouseArea { id: uItemMa; anchors.fill: parent; hoverEnabled: true; onClicked: { root.userIndex = index; root.userMenuOpen = false } }
                    }
                }
            }
        }

        Column {
            id: loginPanel; anchors.right: parent.right; anchors.rightMargin: root.marginR; anchors.bottom: parent.bottom; anchors.bottomMargin: 80 * s; width: 350 * s; spacing: 8 * s
            Item {
                width: parent.width; height: 32 * s; z: 5000
                Text {
                    id: userNameDisp; anchors.right: parent.right; anchors.rightMargin: (uMa.containsMouse || root.userMenuOpen) ? 25 * s : 0
                    text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : ((typeof userModel !== "undefined" && userModel.lastUser) ? capitalizeFirst(userModel.lastUser) : "USER")).toUpperCase()
                    font.family: outfitFontFamily; font.pixelSize: 18 * s; font.weight: Font.Bold; font.letterSpacing: 8 * s; color: (uMa.containsMouse || root.userMenuOpen) ? root.mainText : root.dimText; Behavior on color { ColorAnimation { duration: kit.dur(200) } } Behavior on anchors.rightMargin { NumberAnimation { duration: kit.dur(250); easing.type: kit.ease(Easing.OutCubic) } }
                }
                Text { text: "✦"; anchors.left: userNameDisp.right; anchors.leftMargin: 8 * s; anchors.verticalCenter: userNameDisp.verticalCenter; color: root.accent; opacity: (uMa.containsMouse || root.userMenuOpen) ? 1.0 : 0; font.pixelSize: 12 * s; Behavior on opacity { NumberAnimation { duration: kit.dur(200) } } }
                MouseArea { id: uMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.userMenuOpen = !root.userMenuOpen } }
            }
            Item {
                width: parent.width; height: 30 * s
                TextInput {
                    id: passInput; anchors.fill: parent; echoMode: TextInput.Password; passwordCharacter: "✦"; color: root.dimText; font.family: outfitFontFamily; font.pixelSize: 14 * s; font.letterSpacing: 10 * s; horizontalAlignment: TextInput.AlignRight; verticalAlignment: TextInput.AlignVCenter; focus: true; property bool wasClicked: false; cursorVisible: false; cursorDelegate: Item { width: 0; height: 0 }
                    Keys.onReturnPressed: startLoginSequence()
                    Text { anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: "WAITING FOR KEY"; font.family: outfitFontFamily; font.pixelSize: 10 * s; font.letterSpacing: 4 * s; color: root.inputWaitColor; opacity: passInput.text.length === 0 ? 0.4 : 0; Behavior on opacity { NumberAnimation { duration: kit.dur(400); easing.type: kit.ease(Easing.InOutSine) } } }
                    Rectangle {
                        id: needleCursor; width: 1.5 * s; height: 12 * s; color: root.accent; anchors.verticalCenter: parent.verticalCenter; x: passInput.cursorRectangle.x; visible: passInput.focus && (passInput.text.length > 0 || passInput.wasClicked)
                        SequentialAnimation { loops: Animation.Infinite; running: needleCursor.visible && !kit.reduceMotion; NumberAnimation { target: needleCursor; property: "opacity"; from: 1; to: 0.1; duration: kit.dur(450) } NumberAnimation { target: needleCursor; property: "opacity"; from: 0.1; to: 1; duration: kit.dur(450) } }
                    }
                }
                MouseArea { id: pMa_FixedFinal_Simple; anchors.fill: parent; cursorShape: Qt.ArrowCursor; onClicked: { passInput.forceActiveFocus(); passInput.wasClicked = true } }
            }
            Item {
                width: parent.width; height: 40 * s
                Text {
                    id: loginBtn; anchors.right: parent.right; anchors.rightMargin: btnMa.containsMouse ? 25 * s : 0; text: "ENTER KEY"; font.family: outfitFontFamily; font.pixelSize: 11 * s; font.letterSpacing: 4 * s; font.weight: Font.Bold; color: passInput.text.length > 0 ? (btnMa.containsMouse ? root.mainText : root.dimText) : "transparent"; opacity: passInput.text.length > 0 ? 1.0 : 0; Behavior on anchors.rightMargin { NumberAnimation { duration: kit.dur(250); easing.type: kit.ease(Easing.OutCubic) } }
                }
                Text { text: "✦"; anchors.left: loginBtn.right; anchors.leftMargin: 8 * s; anchors.verticalCenter: loginBtn.verticalCenter; color: root.accent; opacity: (btnMa.containsMouse && passInput.text.length > 0) ? 1.0 : 0; font.pixelSize: 10 * s; Behavior on opacity { NumberAnimation { duration: kit.dur(200) } } }
                MouseArea { id: btnMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { startLoginSequence() } }
            }
            Text { id: errText; width: parent.width; height: 15 * s; verticalAlignment: Text.AlignBottom; horizontalAlignment: Text.AlignRight; text: ""; color: "#ff4444"; font.family: outfitFontFamily; font.pixelSize: 10 * s; font.letterSpacing: 2 * s }
        }
    }

    Timer { id: boomTriggerTimer; interval: kit.dur(1450); onTriggered: { boomSequence.start() } }
    function startLoginSequence() {
        if (passInput.text.length === 0 || isWindup) return
        if (root.enableWindup && !kit.reduceMotion) {
            isWindup = true
            windupAnim.start()
            boomTriggerTimer.start()
        } else {
            doLogin()
        }
    }
    function doLogin() { var uname = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "user"); if (typeof sddm !== "undefined") sddm.login(uname, passInput.text, root.sessionIndex) }
    function capitalizeFirst(str) { if (!str) return ""; return str.charAt(0).toUpperCase() + str.slice(1) }
    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginSucceeded() { }
        function onLoginFailed() { isWindup = false; windupAnim.stop(); boomTriggerTimer.stop(); boomSequence.stop(); root.windupOffset = 0; root.boomScale = 1.0; root.boomOpacity = 0.0; root.sparkIntensity = 0; errText.text = "ACCESS DENIED"; passInput.text = ""; passInput.forceActiveFocus(); shake.start() }
    }
    SequentialAnimation {
        id: shake
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; from: root.marginR; to: root.marginR + 10 * s; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: root.marginR - 10 * s; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: root.marginR; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
    }
    // One ring of the dial: rotating the ring as a whole keeps per-frame work to each tick's highlight.
    component OrbitalRing: Item {
        id: ring
        property real angle: 0
        property real radius: 0
        property real numberInset: 0
        property real majorTick: 0
        property real minorTick: 0
        property real majorWidth: 0
        property real numberSize: 0
        x: clockContainer.cx; y: clockContainer.cy; z: 10
        rotation: ring.angle
        Repeater {
            model: 60
            delegate: Item {
                readonly property real base: index * 6
                readonly property real rad: base * Math.PI / 180
                readonly property real relAngle: { var a = (base + ring.angle) % 360; if (a > 180) a -= 360; if (a < -180) a += 360; return a }
                readonly property real spotlight: Math.max(0, 1.0 - Math.abs(relAngle) / 4.0)
                readonly property bool isMajor: index % 5 == 0
                Rectangle {
                    width: isMajor ? ring.majorWidth : 1 * s; height: isMajor ? ring.majorTick : ring.minorTick
                    x: ring.radius * Math.cos(rad) - width / 2; y: ring.radius * Math.sin(rad) - height / 2
                    color: root.isLight ? Qt.rgba(0, 0, 0, spotlight > 0 ? 1.0 : (isMajor ? 0.8 : 0.6)) : Qt.rgba(1, 1, 1, spotlight > 0 ? 1.0 : (isMajor ? 0.3 : 0.15))
                    rotation: base + 90
                    antialiasing: true
                    transformOrigin: Item.Center
                }
                Text {
                    visible: isMajor; readonly property real nRad: ring.radius - ring.numberInset
                    x: nRad * Math.cos(rad) - width / 2; y: nRad * Math.sin(rad) - height / 2
                    text: String(index).padStart(2, '0'); font.family: root.clockFontFamily; font.pixelSize: ring.numberSize; font.weight: spotlight > 0.5 ? Font.Bold : Font.Normal
                    color: root.isLight ? Qt.rgba(0, 0, 0, spotlight > 0 ? (0.6 + 0.4 * spotlight) : 0.6) : Qt.rgba(1, 1, 1, spotlight > 0 ? (0.4 + spotlight * 0.6) : 0.25)
                    rotation: base; transformOrigin: Item.Center
                    antialiasing: true
                }
            }
        }
    }
    component CwAction: Item {
        id: actItem; width: actTxt.width + 20 * s; height: 15 * s; property string label: ""; signal clicked()
        Text { id: actTxt; anchors.right: parent.right; anchors.rightMargin: actM.containsMouse ? 15 * s : 0; text: label.toUpperCase(); color: actM.containsMouse ? root.mainText : root.dimText; font.family: outfitFontFamily; font.pixelSize: 10 * s; font.letterSpacing: 3 * s; Behavior on color { ColorAnimation { duration: kit.dur(200) } } Behavior on anchors.rightMargin { NumberAnimation { duration: kit.dur(200) } } }
        Text { text: "✦"; anchors.left: actTxt.right; anchors.leftMargin: 4 * s; anchors.verticalCenter: actTxt.verticalCenter; color: root.accent; opacity: actM.containsMouse ? 1.0 : 0; font.pixelSize: 8 * s; Behavior on opacity { NumberAnimation { duration: kit.dur(200) } } }
        MouseArea { id: actM; anchors.fill: parent; hoverEnabled: true; onClicked: { actItem.clicked() } cursorShape: Qt.PointingHandCursor }
    }
}
