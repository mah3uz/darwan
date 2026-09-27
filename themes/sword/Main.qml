import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import Qt.labs.folderlistmodel
import SddmComponents 2.0
import "darwan"

Rectangle {
    Custom { id: kit }
    readonly property string clockFmt: config.clockFormat === "12h" ? (config.clockShowAmPm === "true" ? "h:mm AP" : "h:mm") : "HH:mm"
    // Wayland Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }
    id: root
    width: Screen.width; height: Screen.height
    color: "#050810"
    readonly property real s: height / 768
    readonly property color accent: kit.color("accent", "#6090b8")
    readonly property color accentLight: Qt.colorEqual(accent, "#6090b8") ? "#80b0d8" : Qt.lighter(accent, 1.25)
    readonly property color textColor: kit.color("text", "#ffffff")

    // Quickshell
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined

    // State
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property real ui: 0

    TextConstants { id: textConstants }

    FolderListModel { showDirs: false;
        id: fontFolder
        folder: Qt.resolvedUrl("font")
        nameFilters: ["*.ttf", "*.otf"]
    }

    FontLoader { id: shurikenFont; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }
    readonly property string shurikenFontFamily: kit.font("text", shurikenFont.status === FontLoader.Ready ? shurikenFont.name : "sans-serif")
    readonly property string clockFontFamily: kit.font("clock", shurikenFont.status === FontLoader.Ready ? shurikenFont.name : "sans-serif")

    // Helpers
    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null; currentIndex: root.sessionIndex
        visible: false; width: 0 * s; height: 0 * s
        delegate: Item { property string sName: model.name || "" }
    }

    ListView {
        id: userHelper
        model: typeof userModel !== "undefined" ? userModel : null; currentIndex: root.userIndex
        opacity: 0; width: 100 * s; height: 100 * s; z: -100
        delegate: Item { property string uName: model.realName || model.name || ""; property string uLogin: model.name || "" }
    }

    // Animation
    Component.onCompleted: { fadeAnim.start(); keyboard.numLock = true }

    Timer { interval: 300; running: true; onTriggered: passwordField.forceActiveFocus() }

    NumberAnimation { id: fadeAnim; target: root; property: "ui"; from: 0; to: 1; duration: kit.dur(1400); easing.type: kit.ease(Easing.OutCubic) }

    // Visuals
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#080c14" }
            GradientStop { position: 0.5; color: "#0e1420" }
            GradientStop { position: 1.0; color: "#050810" }
        }
    }

    Background { id: userBg; anchors.fill: parent }
    Loader { anchors.fill: parent; active: !userBg.active; source: "BackgroundVideo.qml" }

    RadialGradient {
        anchors.fill: parent; opacity: 0.75
        gradient: Gradient { GradientStop { position: 0.0; color: "transparent" } GradientStop { position: 1.0; color: "#bb000000" } }
    }

    Rectangle {
        anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; height: 200 * s; opacity: 0.65
        gradient: Gradient { GradientStop { position: 0.0; color: "transparent" } GradientStop { position: 1.0; color: "#dd000000" } }
    }

    // Clock
    Column {
        anchors.left: parent.left; anchors.top: parent.top; anchors.leftMargin: 70 * s; anchors.topMargin: 60 * s; spacing: 8 * s; opacity: root.ui
        Text {
            id: clockText; text: Qt.formatTime(new Date(), clockFmt); color: root.textColor; font.family: clockFontFamily; font.pixelSize: 88 * s; font.weight: Font.Thin
            Timer { interval: 1000; running: true; repeat: true; onTriggered: clockText.text = Qt.formatTime(new Date(), clockFmt) }
        }
        Row {
            spacing: 10 * s
            Rectangle { width: 22 * s; height: 1 * s; color: root.accent; anchors.verticalCenter: parent.verticalCenter }
            Text { text: Qt.formatDate(new Date(), config.dateFormat || "dddd · MMMM d").toUpperCase(); color: root.accent; font.family: shurikenFontFamily; font.pixelSize: 13 * s; font.letterSpacing: 3 * s }
        }
    }

    // Interface
    Column {
        id: loginPanel
        anchors.right: parent.right; anchors.bottom: parent.bottom; anchors.rightMargin: 40 * s; anchors.bottomMargin: 110 * s; width: 280 * s; spacing: 0 * s; opacity: root.ui

        Text {
            id: userDisplay; anchors.right: parent.right
            text: (userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : (typeof userModel !== "undefined" ? (userModel.lastUser || "User") : "User")
            color: root.textColor; font.family: shurikenFontFamily; font.pixelSize: 22 * s; font.letterSpacing: 2 * s
            scale: uMa.containsMouse ? 1.05 : 1.0; Behavior on scale { NumberAnimation { duration: kit.dur(250); easing.type: kit.ease(Easing.OutBack) } }
            transform: Translate { id: uTrans; x: 0 }
            MouseArea {
                id: uMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onClicked: { if (typeof userModel !== "undefined" && userModel.rowCount() > 0) uToggleAnim.start() }
            }
            SequentialAnimation {
                id: uToggleAnim; ParallelAnimation { NumberAnimation { target: userDisplay; property: "opacity"; to: 0; duration: kit.dur(120) } NumberAnimation { target: uTrans; property: "x"; to: 15 * s; duration: kit.dur(120) } }
                ScriptAction { script: root.userIndex = (root.userIndex + 1) % userModel.rowCount() }
                ParallelAnimation { NumberAnimation { target: userDisplay; property: "opacity"; to: 1; duration: kit.dur(180) } NumberAnimation { target: uTrans; property: "x"; to: 0; duration: kit.dur(180) } }
            }
        }

        Item { width: 1 * s; height: 22 * s }

        Item {
            width: parent.width; height: 36 * s
            TextInput {
                id: passwordField; anchors.left: parent.left; anchors.right: arrowHint.left; anchors.rightMargin: 12 * s; anchors.verticalCenter: parent.verticalCenter
                color: "transparent"; font.family: shurikenFontFamily; font.pixelSize: 14 * s; echoMode: TextInput.NoEcho; focus: true; clip: true; cursorVisible: false; cursorDelegate: Item { width: 0; height: 0 }
                selectionColor: root.accent; property bool wasClicked: false; onTextEdited: errorMessage.text = ""
                Keys.onReturnPressed: doLogin(); Keys.onEnterPressed: doLogin()
                Row {
                    anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; spacing: 8 * s
                    Repeater { model: passwordField.text.length; delegate: Text { text: "✦"; color: root.textColor; font: passwordField.font; verticalAlignment: Text.AlignVCenter } }
                    Text {
                        id: customCursor; text: "✦"; color: "white"; font: passwordField.font; verticalAlignment: Text.AlignVCenter; visible: passwordField.focus && (passwordField.text.length > 0 || passwordField.wasClicked)
                        layer.enabled: true; layer.effect: DropShadow { color: "white"; radius: 8; samples: 16 }
                        SequentialAnimation { loops: Animation.Infinite; running: (customCursor.visible) && !kit.reduceMotion; NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0.2; duration: kit.dur(600); easing.type: kit.ease(Easing.InOutSine) } NumberAnimation { target: customCursor; property: "opacity"; from: 0.2; to: 1; duration: kit.dur(600); easing.type: kit.ease(Easing.InOutSine) } }
                    }
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter; text: "Enter password"; color: root.textColor
                    opacity: (passwordField.text.length === 0 && !passwordField.wasClicked) ? 0.25 : 0
                    Behavior on opacity { NumberAnimation { duration: kit.dur(450); easing.type: kit.ease(Easing.InOutSine) } }
                    font.family: shurikenFontFamily; font.pixelSize: 14 * s; font.letterSpacing: 2 * s
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.IBeamCursor; onClicked: { passwordField.forceActiveFocus(); passwordField.wasClicked = true } }
            }
            Text {
                id: arrowHint; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: "→"; color: root.accent; font.pixelSize: 16 * s; opacity: passwordField.text.length > 0 ? 1.0 : 0.3
                Behavior on opacity { NumberAnimation { duration: kit.dur(200) } }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: doLogin() }
            }
        }

        Rectangle { width: parent.width; height: 1 * s; color: passwordField.activeFocus ? root.accentLight : "#28607888"; Behavior on color { ColorAnimation { duration: kit.dur(300) } } }

        Item { width: 1 * s; height: 10 * s }

        Text { id: errorMessage; anchors.right: parent.right; height: 15 * s; verticalAlignment: Text.AlignTop; font.family: shurikenFontFamily; font.pixelSize: 11 * s; font.letterSpacing: 1 * s; color: "#d06060"; text: "" }
    }

    Rectangle {
        anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; anchors.leftMargin: 40 * s; anchors.rightMargin: 40 * s; anchors.bottomMargin: 30 * s; height: 1 * s; color: "#15a0c8e0"; opacity: root.ui
    }

    Item {
        anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; anchors.margins: 40 * s; height: 40 * s; opacity: root.ui * 0.8

        Item {
            width: sessionSwitchRow.implicitWidth; height: sessionSwitchRow.implicitHeight; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; visible: !root.isQuickshell
            Row {
                id: sessionSwitchRow; spacing: 10 * s; opacity: sMa.containsMouse ? 1.0 : 0.85; scale: sMa.containsMouse ? 1.05 : 1.0
                transform: Translate { id: sTrans; x: 0 }
                Text { text: "◈"; color: "#405070"; font.pixelSize: 10 * s; anchors.verticalCenter: parent.verticalCenter }
                Text {
                    id: sessionLabel; text: (typeof sessionModel !== "undefined" && sessionModel.count > root.sessionIndex && root.sessionIndex >= 0 && sessionHelper.currentItem) ? sessionHelper.currentItem.sName : "Session"
                    color: root.textColor; opacity: 0.6; font.family: shurikenFontFamily; font.pixelSize: 12 * s; font.letterSpacing: 1 * s; anchors.verticalCenter: parent.verticalCenter
                }
            }
            MouseArea { id: sMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { if (typeof sessionModel !== "undefined" && sessionModel.rowCount() > 0) sToggleAnim.start() } }
            SequentialAnimation {
                id: sToggleAnim; ParallelAnimation { NumberAnimation { target: sessionLabel; property: "opacity"; to: 0; duration: kit.dur(120) } NumberAnimation { target: sTrans; property: "x"; to: 10 * s; duration: kit.dur(120) } }
                ScriptAction { script: root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount() }
                ParallelAnimation { NumberAnimation { target: sessionLabel; property: "opacity"; to: 0.6; duration: kit.dur(180) } NumberAnimation { target: sTrans; property: "x"; to: 0; duration: kit.dur(180) } }
            }
        }

        Row {
            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; spacing: 28 * s
            Text {
                text: "Restart"; color: root.textColor; opacity: 0.4; font.family: shurikenFontFamily; font.pixelSize: 12 * s; font.letterSpacing: 1 * s
                scale: rMa.containsMouse ? 1.1 : 1.0; Behavior on opacity { NumberAnimation { duration: kit.dur(150) } }
                MouseArea { id: rMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onEntered: parent.opacity = 0.9; onExited: parent.opacity = 0.4; onClicked: { if (typeof sddm !== "undefined") sddm.reboot() } }
            }
            Text {
                text: "Shut Down"; color: root.textColor; opacity: 0.4; font.family: shurikenFontFamily; font.pixelSize: 12 * s; font.letterSpacing: 1 * s
                scale: pMa.containsMouse ? 1.1 : 1.0; Behavior on opacity { NumberAnimation { duration: kit.dur(150) } }
                MouseArea { id: pMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onEntered: parent.opacity = 0.9; onExited: parent.opacity = 0.4; onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() } }
            }
        }
    }

    // Action
    function doLogin() {
        var uname = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "")
        if (typeof sddm !== "undefined") sddm.login(uname, passwordField.text, root.sessionIndex)
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() { errorMessage.text = "ACCESS DENIED"; passwordField.text = ""; passwordField.focus = true; shakeAnim.start() }
    }

    SequentialAnimation {
        id: shakeAnim
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: 50 * s; duration: kit.dur(50) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: 30 * s; duration: kit.dur(50) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: 45 * s; duration: kit.dur(50) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: 35 * s; duration: kit.dur(50) }
        NumberAnimation { target: loginPanel; property: "anchors.rightMargin"; to: 40 * s; duration: kit.dur(50) }
    }
}
