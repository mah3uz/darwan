// SDDM Theme
import QtQuick
import QtQuick.Window
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import Qt.labs.folderlistmodel
import SddmComponents 2.0
import "darwan"

Rectangle {
    Custom { id: kit }
    id: root
    readonly property real s: Screen.height / 768
    width: Screen.width
    height: Screen.height
    color: isDark ? "#1d1530" : "#faf0e6"

    // Wayland Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }

    // Colors. Dark keeps the pastel chips (neo-brutalism reads even harder on a dark field) and lightens only
    // what is drawn straight onto the background.
    readonly property bool isDark: config.colorScheme === "dark"
    readonly property color mainText: kit.color("text", "#231c1a")
    readonly property color dimText: "#6a5a54"
    readonly property color outlineColor: isDark ? "#0b0812" : "#161110"
    readonly property color dialColor: isDark ? "#e8def5" : outlineColor
    readonly property color numeralColor: isDark ? "#e8def5" : mainText
    readonly property color pillBg: "#faf0e6"
    readonly property color peachAccent: kit.color("accent", "#ffb3a1")
    readonly property color peachHover: config.colorAccent ? Qt.darker(peachAccent, 1.1) : "#ff9e85"
    readonly property color greenAccent: kit.color("colorGreen", "#c5e1a5")
    readonly property color lavenderAccent: kit.color("colorLavender", "#e1bee7")
    readonly property color roseAccent: "#ffcbd5"

    // State
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property real uiOpacity: 0
    readonly property real marginR: 80 * s

    // Time Engine
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

    readonly property real smoothSecAngle: -((localTimeMS % 60000) / 60000.0) * 360.0
    readonly property real smoothMinAngle: -((localTimeMS % 3600000) / 3600000.0) * 360.0

    // Fonts
    FolderListModel { showDirs: false; id: fontFolder; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
    FontLoader { id: outfitFont; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }
    readonly property string outfitFontFamily: kit.font("text", outfitFont.status === FontLoader.Ready ? outfitFont.name : "sans-serif")
    readonly property string clockFontFamily: kit.font("clock", outfitFont.status === FontLoader.Ready ? outfitFont.name : "sans-serif")
    TextConstants { id: textConstants }

    // Helpers
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

    // Logic
    Timer { interval: 300; running: true; onTriggered: passInput.forceActiveFocus() }
    Component.onCompleted: { syncClock(); fadeAnim.start(); keyboard.numLock = true }
    NumberAnimation { id: fadeAnim; target: root; property: "uiOpacity"; from: 0; to: 1; duration: kit.dur(1200); easing.type: kit.ease(Easing.OutCubic) }

    // Background Image
    Image {
        anchors.fill: parent
        source: "bg.png"
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        opacity: root.uiOpacity
        visible: !root.isDark && !userBg.active
    }
    // The light background is only a gradient, so the dark one is drawn: aubergine into midnight.
    Rectangle {
        anchors.fill: parent
        opacity: root.uiOpacity
        visible: root.isDark && !userBg.active
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#2a1f3d" }
            GradientStop { position: 1.0; color: "#1d1530" }
        }
    }
    Background { id: userBg; anchors.fill: parent; opacity: root.uiOpacity }

    // Layout Container
    Item {
        id: blastContainer
        anchors.fill: parent
        opacity: root.uiOpacity

        // Clock Section
        Item {
            id: clockContainer
            anchors.left: parent.left; anchors.leftMargin: 20 * s
            anchors.verticalCenter: parent.verticalCenter
            width: 700 * s; height: parent.height

            readonly property real cx: 0 * s 
            readonly property real cy: height * 0.5
            readonly property real minR: 270 * s 
            readonly property real secR: 420 * s 

            // Hour Box
            Item {
                id: hourBox
                x: 10 * s
                anchors.verticalCenter: parent.verticalCenter
                width: 150 * s; height: 120 * s
                z: 100

                Rectangle {
                    anchors.fill: parent
                    anchors.topMargin: 4 * s; anchors.leftMargin: 4 * s; anchors.bottomMargin: -4 * s; anchors.rightMargin: -4 * s
                    radius: 20 * s; color: root.outlineColor
                }

                Rectangle {
                    anchors.fill: parent
                    radius: 20 * s; color: root.peachAccent
                    border.color: root.outlineColor; border.width: 2.5 * s

                    Text {
                        anchors.centerIn: parent
                        text: root.clockHour(root.curH)
                        font.family: root.clockFontFamily; font.pixelSize: 76 * s; font.weight: Font.Black
                        color: root.mainText
                    }
                }
            }

// Indicator Pill
            Item {
                id: indicatorPill; z: 10
                x: hourBox.x + hourBox.width + 24 * s
                anchors.verticalCenter: parent.verticalCenter
                width: 260 * s; height: 76 * s

                // Glass Lens
                Rectangle {
                    anchors.fill: parent
                    radius: 38 * s
                    color: "#35faf0e6"
                    border.color: root.outlineColor
                    border.width: 2.5 * s

                    // Center Line
                    Rectangle {
                        x: 129 * s
                        anchors.verticalCenter: parent.verticalCenter
                        width: 2 * s; height: 32 * s
                        color: root.outlineColor
                        opacity: 0.35
                    }
                }
            }

            // Minute and second rings
            NeoRing { angle: root.smoothMinAngle; radius: clockContainer.minR; numberInset: 32 * s; majorTick: 18 * s; minorTick: 10 * s; majorWidth: 2.5 * s; minorWidth: 1.2 * s; numberSize: 18 * s }
            NeoRing { angle: root.smoothSecAngle; radius: clockContainer.secR; numberInset: 28 * s; majorTick: 14 * s; minorTick: 8 * s; majorWidth: 2.0 * s; minorWidth: 1.0 * s; numberSize: 14 * s }

            // Date & Day Column
            Column {
                anchors.left: indicatorPill.right; anchors.leftMargin: 35 * s; anchors.verticalCenter: parent.verticalCenter; spacing: 10 * s

                Item {
                    width: 190 * s; height: 38 * s

                    Rectangle {
                        anchors.fill: parent
                        anchors.topMargin: 2.5 * s; anchors.leftMargin: 2.5 * s; anchors.bottomMargin: -2.5 * s; anchors.rightMargin: -2.5 * s
                        radius: 8 * s; color: root.outlineColor
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: 8 * s; color: root.greenAccent
                        border.color: root.outlineColor; border.width: 1.5 * s

                        Text {
                            anchors.centerIn: parent
                            text: Qt.formatDate(new Date(), config.dateFormatNoWeekday || "dd MMM yyyy").toUpperCase() + (root.amPm ? " · " + root.amPm : "")
                            font.family: outfitFontFamily; font.pixelSize: 12 * s; font.letterSpacing: 2 * s; font.weight: Font.Bold
                            color: root.mainText
                        }
                    }
                }

                Item {
                    width: 190 * s; height: 42 * s

                    Rectangle {
                        anchors.fill: parent
                        anchors.topMargin: 2.5 * s; anchors.leftMargin: 2.5 * s; anchors.bottomMargin: -2.5 * s; anchors.rightMargin: -2.5 * s
                        radius: 8 * s; color: root.outlineColor
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: 8 * s; color: root.lavenderAccent
                        border.color: root.outlineColor; border.width: 1.5 * s

                        Text {
                            anchors.centerIn: parent
                            text: Qt.formatDate(new Date(), "dddd").toUpperCase()
                            font.family: outfitFontFamily; font.pixelSize: 14 * s; font.letterSpacing: 4 * s; font.weight: Font.Bold
                            color: root.mainText
                        }
                    }
                }
            }
        }
    }

    // HUD Actions
    Item {
        id: hudContainer; anchors.fill: parent; opacity: root.uiOpacity
        Row {
            anchors.right: parent.right; anchors.rightMargin: root.marginR; anchors.top: parent.top; anchors.topMargin: 40 * s; spacing: 14 * s

            CwAction {
                label: (sessionHelper.currentItem ? sessionHelper.currentItem.sName : "Session")
                bgColor: root.lavenderAccent
                onClicked: { if (typeof sessionModel !== "undefined") root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount() }
            }

            CwAction {
                label: "Reboot"
                bgColor: root.peachAccent
                onClicked: { if (typeof sddm !== "undefined") sddm.reboot() }
            }

            CwAction {
                label: "Shutdown"
                bgColor: root.roseAccent
                onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() }
            }
        }

        // Login Panel
        Column {
            id: loginPanel; anchors.right: parent.right; anchors.rightMargin: root.marginR; anchors.bottom: parent.bottom; anchors.bottomMargin: 80 * s; width: 320 * s; spacing: 12 * s

            // Username Button (Click Cycles Users)
            Item {
                width: parent.width; height: 46 * s; z: 5000

                Rectangle {
                    anchors.fill: parent
                    anchors.topMargin: 3 * s; anchors.leftMargin: 3 * s; anchors.bottomMargin: -3 * s; anchors.rightMargin: -3 * s
                    radius: 12 * s; color: root.outlineColor
                }

                Rectangle {
                    anchors.fill: parent; radius: 12 * s
                    color: uMa.containsMouse ? root.peachAccent : "#f5ebe6"
                    border.color: root.outlineColor; border.width: 2.0 * s

                    transform: Translate {
                        x: uMa.pressed ? 3 * s : (uMa.containsMouse ? -2 * s : 0)
                        y: uMa.pressed ? 3 * s : (uMa.containsMouse ? -2 * s : 0)
                        Behavior on x { NumberAnimation { duration: kit.dur(100) } }
                        Behavior on y { NumberAnimation { duration: kit.dur(100) } }
                    }

                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 10 * s; anchors.rightMargin: 12 * s
                        spacing: 10 * s

                        // User Socket
                        Item {
                            width: 32 * s; height: 32 * s
                            anchors.verticalCenter: parent.verticalCenter

                            Rectangle {
                                anchors.fill: parent
                                anchors.topMargin: 2 * s; anchors.leftMargin: 2 * s; anchors.bottomMargin: -2 * s; anchors.rightMargin: -2 * s
                                radius: 8 * s; color: root.outlineColor
                            }

                            Rectangle {
                                anchors.fill: parent
                                radius: 8 * s; color: root.greenAccent
                                border.color: root.outlineColor; border.width: 1.5 * s

                                Text {
                                    anchors.centerIn: parent
                                    text: "󰨞"
                                    font.family: "JetBrainsMono Nerd Font"
                                    font.pixelSize: 16 * s; font.weight: Font.Bold
                                    color: root.mainText
                                }
                            }
                        }

                        Text {
                            width: parent.width - 70 * s
                            text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : ((typeof userModel !== "undefined" && userModel.lastUser) ? capitalizeFirst(userModel.lastUser) : "USER")).toUpperCase()
                            font.family: outfitFontFamily; font.pixelSize: 12 * s; font.weight: Font.Bold; font.letterSpacing: 2 * s
                            color: root.mainText; anchors.verticalCenter: parent.verticalCenter
                            elide: Text.ElideRight
                        }

                        Text {
                            text: "󰅀"
                            font.family: "JetBrainsMono Nerd Font"
                            font.pixelSize: 12 * s; font.weight: Font.Bold
                            color: root.mainText; anchors.verticalCenter: parent.verticalCenter
                        }
                    }

                    MouseArea {
                        id: uMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (typeof userModel !== "undefined" && userModel.rowCount() > 0) {
                                root.userIndex = (root.userIndex + 1) % userModel.rowCount();
                            }
                        }
                    }
                }
            }

            // Password Field
            Item {
                width: parent.width; height: 48 * s

                Rectangle {
                    anchors.fill: parent
                    anchors.topMargin: 3 * s; anchors.leftMargin: 3 * s; anchors.bottomMargin: -3 * s; anchors.rightMargin: -3 * s
                    radius: 12 * s; color: root.outlineColor
                }

                Rectangle {
                    anchors.fill: parent; radius: 12 * s
                    color: errText.text !== "" ? root.roseAccent : "#faf0e6"
                    border.color: errText.text !== "" ? "#ff4444" : root.outlineColor
                    border.width: 2.0 * s

                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 10 * s; anchors.rightMargin: 12 * s
                        spacing: 12 * s

                        // Lock Socket
                        Item {
                            width: 32 * s; height: 32 * s
                            anchors.verticalCenter: parent.verticalCenter

                            Rectangle {
                                anchors.fill: parent
                                anchors.topMargin: 2 * s; anchors.leftMargin: 2 * s; anchors.bottomMargin: -2 * s; anchors.rightMargin: -2 * s
                                radius: 8 * s; color: root.outlineColor
                            }

                            Rectangle {
                                anchors.fill: parent
                                radius: 8 * s
                                color: errText.text !== "" ? root.roseAccent : root.lavenderAccent
                                border.color: root.outlineColor; border.width: 1.5 * s

                                Text {
                                    anchors.centerIn: parent
                                    text: "󰌆"
                                    font.family: "JetBrainsMono Nerd Font"
                                    font.pixelSize: 16 * s; font.weight: Font.Bold
                                    color: root.mainText
                                }
                            }
                        }

                        // Input Container
                        Item {
                            width: parent.width - 54 * s
                            height: parent.height
                            anchors.verticalCenter: parent.verticalCenter
                            clip: true

                            TextInput {
                                id: passInput
                                anchors.fill: parent
                                echoMode: TextInput.NoEcho
                                color: "transparent"
                                focus: true
                                cursorVisible: false
                                cursorDelegate: Item { width: 0; height: 0 }

                                Keys.onReturnPressed: doLogin()
                            }

                            // Placeholder
                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: 2 * s
                                anchors.verticalCenter: parent.verticalCenter
                                text: "Enter Password..."
                                font.family: outfitFontFamily
                                font.pixelSize: 11 * s
                                font.letterSpacing: 1 * s
                                color: root.dimText
                                visible: passInput.text.length === 0
                            }

                            // Neobrutalist Keycaps
                            Row {
                                id: dotsRow
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: 2 * s
                                spacing: 6 * s
                                visible: passInput.text.length > 0

                                Repeater {
                                    model: passInput.text.length
                                    delegate: Rectangle {
                                        width: 10 * s; height: 10 * s
                                        radius: 3 * s
                                        color: root.peachAccent
                                        border.color: root.outlineColor
                                        border.width: 1.5 * s
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                }

                                Rectangle {
                                    width: 2 * s; height: 14 * s
                                    color: root.mainText
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: passInput.activeFocus

                                    SequentialAnimation on opacity {
                                        loops: Animation.Infinite
                                        running: passInput.activeFocus && !kit.reduceMotion
                                        NumberAnimation { from: 1.0; to: 0.1; duration: kit.dur(450) }
                                        NumberAnimation { from: 0.1; to: 1.0; duration: kit.dur(450) }
                                    }
                                }
                            }

                            // Empty Cursor
                            Rectangle {
                                width: 2 * s; height: 14 * s
                                color: root.mainText
                                anchors.left: parent.left
                                anchors.leftMargin: 2 * s
                                anchors.verticalCenter: parent.verticalCenter
                                visible: passInput.activeFocus && passInput.text.length === 0

                                SequentialAnimation on opacity {
                                    loops: Animation.Infinite
                                    running: passInput.activeFocus && passInput.text.length === 0 && !kit.reduceMotion
                                    NumberAnimation { from: 1.0; to: 0.1; duration: kit.dur(450) }
                                    NumberAnimation { from: 0.1; to: 1.0; duration: kit.dur(450) }
                                }
                            }
                        }
                    }
                }
            }

            // Submit Button
            Item {
                width: parent.width; height: 44 * s

                Rectangle {
                    anchors.fill: parent
                    anchors.topMargin: 3 * s; anchors.leftMargin: 3 * s; anchors.bottomMargin: -3 * s; anchors.rightMargin: -3 * s
                    radius: 12 * s; color: root.outlineColor
                }

                Rectangle {
                    anchors.fill: parent
                    radius: 12 * s
                    color: btnMa.containsMouse ? root.peachHover : root.peachAccent
                    border.color: root.outlineColor; border.width: 2.0 * s

                    transform: Translate {
                        x: btnMa.pressed ? 3 * s : (btnMa.containsMouse ? -2 * s : 0)
                        y: btnMa.pressed ? 3 * s : (btnMa.containsMouse ? -2 * s : 0)
                        Behavior on x { NumberAnimation { duration: kit.dur(100) } }
                        Behavior on y { NumberAnimation { duration: kit.dur(100) } }
                    }

                    Row {
                        anchors.centerIn: parent; spacing: 8 * s
                        Text { text: "󰍁"; font.family: "JetBrainsMono Nerd Font"; font.pixelSize: 16 * s; font.weight: Font.Bold; color: root.mainText; anchors.verticalCenter: parent.verticalCenter }
                        Text { text: "LOGIN"; font.family: outfitFontFamily; font.pixelSize: 12 * s; font.weight: Font.Bold; font.letterSpacing: 2 * s; color: root.mainText; anchors.verticalCenter: parent.verticalCenter }
                    }

                    MouseArea { id: btnMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: doLogin() }
                }
            }

            Text { id: errText; width: parent.width; height: 16 * s; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter; text: ""; color: "#ff4444"; font.family: outfitFontFamily; font.pixelSize: 11 * s; font.weight: Font.Bold; font.letterSpacing: 1 * s }
        }
    }

    // SDDM Connections
    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() {
            errText.text = "ACCESS DENIED";
            passInput.text = "";
            passInput.forceActiveFocus();
            shake.start();
        }
    }

    function doLogin() {
        var u = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "");
        if (typeof sddm !== "undefined") sddm.login(u, passInput.text, root.sessionIndex);
    }

    function capitalizeFirst(str) { if (!str) return ""; return str.charAt(0).toUpperCase() + str.slice(1) }

    SequentialAnimation {
        id: shake
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; from: root.marginR; to: root.marginR + 10 * s; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: root.marginR - 10 * s; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: root.marginR; duration: kit.dur(50); easing.type: kit.ease(Easing.InOutSine) }
    }

    // Action Component
    component CwAction: Item {
        id: actItem
        width: actTxt.implicitWidth + 32 * s
        height: 36 * s
        property string label: ""
        property color bgColor: root.peachAccent
        signal clicked()

        Rectangle {
            anchors.fill: parent
            anchors.topMargin: 2.5 * s; anchors.leftMargin: 2.5 * s; anchors.bottomMargin: -2.5 * s; anchors.rightMargin: -2.5 * s
            radius: 8 * s; color: root.outlineColor
        }

        Rectangle {
            id: actPlate
            anchors.fill: parent
            radius: 8 * s
            color: actM.containsMouse ? root.peachAccent : bgColor
            border.color: root.outlineColor; border.width: 1.5 * s

            transform: Translate {
                x: actM.pressed ? 2.5 * s : (actM.containsMouse ? -1.5 * s : 0)
                y: actM.pressed ? 2.5 * s : (actM.containsMouse ? -1.5 * s : 0)
                Behavior on x { NumberAnimation { duration: kit.dur(80) } }
                Behavior on y { NumberAnimation { duration: kit.dur(80) } }
            }

            Text {
                id: actTxt; anchors.centerIn: parent
                text: label.toUpperCase(); color: root.mainText
                font.family: outfitFontFamily; font.pixelSize: 11 * s; font.weight: Font.Bold; font.letterSpacing: 2 * s
            }

            MouseArea { id: actM; anchors.fill: parent; hoverEnabled: true; onClicked: actItem.clicked(); cursorShape: Qt.PointingHandCursor }
        }
    }

    // One ring of the dial, rotated as a whole: its ticks don't change as it turns, so nothing runs per tick.
    component NeoRing: Item {
        id: ring
        property real angle: 0
        property real radius: 0
        property real numberInset: 0
        property real majorTick: 0
        property real minorTick: 0
        property real majorWidth: 0
        property real minorWidth: 0
        property real numberSize: 0
        x: clockContainer.cx; y: clockContainer.cy; z: 10
        rotation: ring.angle
        Repeater {
            model: 60
            delegate: Item {
                readonly property real base: index * 6
                readonly property real rad: base * Math.PI / 180
                readonly property bool isMajor: index % 5 == 0
                Rectangle {
                    width: isMajor ? ring.majorWidth : ring.minorWidth; height: isMajor ? ring.majorTick : ring.minorTick
                    x: ring.radius * Math.cos(rad) - width / 2; y: ring.radius * Math.sin(rad) - height / 2
                    color: root.dialColor
                    rotation: base + 90
                    antialiasing: true
                    transformOrigin: Item.Center
                }
                Text {
                    visible: isMajor; readonly property real nRad: ring.radius - ring.numberInset
                    x: nRad * Math.cos(rad) - width / 2; y: nRad * Math.sin(rad) - height / 2
                    text: String(index).padStart(2, '0'); font.family: root.clockFontFamily; font.pixelSize: ring.numberSize; font.weight: Font.Bold
                    color: root.numeralColor
                    rotation: base; transformOrigin: Item.Center
                    antialiasing: true
                }
            }
        }
    }
}
