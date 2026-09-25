import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects as Gfx
import Qt.labs.folderlistmodel
import SddmComponents 2.0

Rectangle {
    // Wayland Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }
    id: root
    width: Screen.width; height: Screen.height
    readonly property real s: height / 768
    color: "#3a6370"

    // Palette
    readonly property color cAmber: "#e8a040"   // Amber
    readonly property color cSteel: "#d8eef8"   // Steel
    readonly property color cWhite: "#f0ede8"   // White

    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex:    (typeof userModel    !== "undefined" && userModel.lastIndex    >= 0) ? userModel.lastIndex    : 0
    property real ui: 0

    FolderListModel { id: fontFolder; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
    FontLoader      { id: mainFont;   source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }

    ListView { id: sessionHelper; model: typeof sessionModel !== "undefined" ? sessionModel : null; currentIndex: root.sessionIndex; opacity: 0; width: 1; height: 1; delegate: Item { property string sName: model.name || "" } }
    ListView { id: userHelper;    model: typeof userModel    !== "undefined" ? userModel    : null; currentIndex: root.userIndex;    opacity: 0; width: 1; height: 1; delegate: Item { property string uName: model.realName || model.name || ""; property string uLogin: model.name || "" } }

    Timer { interval: 300; running: true; onTriggered: pwd.forceActiveFocus() }
    Component.onCompleted: { fadeAnim.start(); keyboard.numLock = true }
    NumberAnimation { id: fadeAnim; target: root; property: "ui"; from: 0; to: 1; duration: 1400; easing.type: Easing.OutCubic }

    // Background
    Image {
        anchors.fill: parent; source: "bg.png"
        fillMode: Image.PreserveAspectCrop; asynchronous: true; opacity: root.ui
    }
    Rectangle {
        anchors.fill: parent; visible: root.ui < 1.0
        opacity: 1.0 - root.ui; color: "#3a6370"; z: 100
    }

    // Clock
    Column {
        anchors { left: parent.left; top: parent.top; leftMargin: 70 * s; topMargin: 60 * s }
        opacity: root.ui
        spacing: 8 * s

        Text {
            id: clockText
            text: Qt.formatTime(new Date(), "HH:mm")
            color: cWhite
            font.family: mainFont.name; font.pixelSize: 96 * s
            font.weight: Font.Light; font.letterSpacing: -1 * s
            layer.enabled: true
            layer.effect: Gfx.DropShadow { color: "#30000000"; radius: 16; samples: 25 }
            Timer { interval: 1000; running: true; repeat: true; onTriggered: clockText.text = Qt.formatTime(new Date(), "HH:mm") }
        }

        Text {
            text: Qt.formatDate(new Date(), "dddd · MMMM d").toUpperCase()
            color: cSteel
            font.family: mainFont.name; font.pixelSize: 12 * s
            font.letterSpacing: 3 * s; font.weight: Font.Light
            layer.enabled: true
            layer.effect: Gfx.DropShadow { color: "#20000000"; radius: 6; samples: 11 }
        }
    }

    // Login
    Column {
        anchors { right: parent.right; bottom: parent.bottom; rightMargin: 70 * s; bottomMargin: 190 * s }
        opacity: root.ui
        spacing: 0
        width: 200 * s

        // Username
        Text {
            id: userDisp
            anchors.right: parent.right
            text: ((userHelper.currentItem && userHelper.currentItem.uName)
                   ? userHelper.currentItem.uName
                   : (typeof userModel !== "undefined" ? userModel.lastUser : "user")).toLowerCase()
            color: userMa.containsMouse ? cAmber : cWhite
            font.family: mainFont.name; font.pixelSize: 22 * s
            font.weight: Font.Light; font.letterSpacing: 3 * s
            horizontalAlignment: Text.AlignRight
            layer.enabled: true
            layer.effect: Gfx.DropShadow { color: "#25000000"; radius: 8; samples: 15 }
            Behavior on color { ColorAnimation { duration: 250 } }
            MouseArea {
                id: userMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onClicked: { if (typeof userModel !== "undefined") root.userIndex = (root.userIndex + 1) % userModel.rowCount() }
            }
        }

        Item { width: 1; height: 22 * s }

        // Password
        Item {
            anchors.right: parent.right
            width: parent.width; height: 46 * s

            Rectangle {
                anchors { bottom: parent.bottom; right: parent.right }
                width: parent.width; height: pwd.focus ? 2 * s : 1 * s
                color: pwd.focus ? cAmber : "#35f0ede8"
                Behavior on color  { ColorAnimation  { duration: 300 } }
                Behavior on height { NumberAnimation { duration: 200 } }
            }

            TextInput {
                id: pwd
                anchors.fill: parent; anchors.bottomMargin: 4 * s
                horizontalAlignment: TextInput.AlignRight
                verticalAlignment:   TextInput.AlignVCenter
                echoMode: TextInput.Password; passwordCharacter: "·"
                color: cWhite
                font.family: mainFont.name; font.pixelSize: 20 * s; font.letterSpacing: 10 * s
                focus: true; clip: true; cursorVisible: false
                cursorDelegate: Item { width: 0; height: 0 }
                onAccepted: doLogin()

                Text {
                    anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                    text: "password"; color: cWhite
                    opacity: pwd.text.length === 0 ? 0.55 : 0
                    font.family: mainFont.name; font.pixelSize: 13 * s
                    font.letterSpacing: 2 * s; font.weight: Font.Light
                    layer.enabled: true
                    layer.effect: Gfx.DropShadow { color: "#30000000"; radius: 6; samples: 11 }
                    Behavior on opacity { NumberAnimation { duration: 200 } }
                }

                Rectangle {
                    id: cursor
                    width: 2 * s; height: 20 * s; color: cAmber
                    anchors { verticalCenter: parent.verticalCenter; right: parent.right }
                    visible: pwd.focus && pwd.text.length > 0
                    SequentialAnimation {
                        loops: Animation.Infinite; running: cursor.visible
                        NumberAnimation { target: cursor; property: "opacity"; from: 1.0; to: 0.1; duration: 450 }
                        NumberAnimation { target: cursor; property: "opacity"; from: 0.1; to: 1.0; duration: 450 }
                    }
                }
            }
            MouseArea { anchors.fill: parent; cursorShape: Qt.IBeamCursor; onClicked: pwd.forceActiveFocus() }
        }

        Item { width: 1; height: 20 * s }

        // Actions
        Row {
            anchors.right: parent.right; spacing: 20 * s
            Repeater {
                model: [
                    { l: "session", a: 0 },
                    { l: "reboot",  a: 1 },
                    { l: "power",   a: 2 }
                ]
                delegate: Text {
                    text: (modelData.a === 0 && sessionHelper.currentItem)
                          ? sessionHelper.currentItem.sName.toLowerCase() : modelData.l
                    color: pm.containsMouse ? cAmber : cWhite
                    font.family: mainFont.name; font.pixelSize: 11 * s
                    font.letterSpacing: 2 * s; font.weight: Font.Light
                    opacity: pm.containsMouse ? 1.0 : 0.7
                    layer.enabled: true
                    layer.effect: Gfx.DropShadow { color: "#35000000"; radius: 6; samples: 11 }
                    Behavior on opacity { NumberAnimation { duration: 200 } }
                    Behavior on color   { ColorAnimation  { duration: 200 } }
                    MouseArea {
                        id: pm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if      (modelData.a === 0) { if (typeof sessionModel !== "undefined") root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount() }
                            else if (modelData.a === 1) { if (typeof sddm !== "undefined") sddm.reboot() }
                            else if (modelData.a === 2) { if (typeof sddm !== "undefined") sddm.powerOff() }
                        }
                    }
                }
            }
        }

        // Error
        Text {
            id: errorMsg
            anchors.right: parent.right
            text: ""; color: "#e06060"; visible: text !== ""
            font.family: mainFont.name; font.pixelSize: 11 * s; font.letterSpacing: 1.5 * s
        }
    }

    // Wiring
    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() { errorMsg.text = "wrong password"; pwd.text = ""; pwd.focus = true }
    }

    function doLogin() {
        var u = (userHelper.currentItem && userHelper.currentItem.uLogin)
                ? userHelper.currentItem.uLogin
                : (typeof userModel !== "undefined" ? userModel.lastUser : "")
        if (typeof sddm !== "undefined") sddm.login(u, pwd.text, root.sessionIndex)
    }
}
