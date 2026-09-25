import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import Qt.labs.folderlistmodel
import SddmComponents 2.0

Rectangle {
    id: root
    width: Screen.width
    height: Screen.height
    color: "#0b151f"

    readonly property real s: Screen.height / 768
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property real ui: 0

    // Colors
    readonly property color cleanWhite: "#ffffff"
    readonly property color lightCyan: "#b2f0f4"
    readonly property color deepIceBlue: "#7bc3d4"
    readonly property color darkTealLine: "#1c5b6e"
    readonly property color watermarkTeal: "#3892a8"

    FolderListModel {
        id: fontFolder
        folder: Qt.resolvedUrl("font")
        nameFilters: ["*.ttf", "*.otf"]
    }

    FontLoader {
        id: pf
        source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : ""
    }

    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null
        currentIndex: root.sessionIndex
        opacity: 0
        width: 100
        height: 100
        z: -100
        delegate: Item {
            property string sName: model.name || ""
        }
    }

    ListView {
        id: userHelper
        model: typeof userModel !== "undefined" ? userModel : null
        currentIndex: root.userIndex
        opacity: 0
        width: 100
        height: 100
        z: -100
        delegate: Item {
            property string uName: model.realName || model.name || ""
            property string uLogin: model.name || ""
        }
    }

    // Autofocus
    Timer {
        interval: 300
        running: true
        onTriggered: pwd.forceActiveFocus()
    }

    Component.onCompleted: {
        fadeAnim.start();
        keyboard.numLock = true;
    }

    NumberAnimation {
        id: fadeAnim
        target: root
        property: "ui"
        from: 0
        to: 1
        duration: 800
        easing.type: Easing.OutCubic
    }

    // Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }

    Loader {
        anchors.fill: parent
        source: "BackgroundVideo.qml"
    }

    // Overlays
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 180 * s
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#e507111a" }
            GradientStop { position: 1.0; color: "transparent" }
        }
    }

    Rectangle {
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 300 * s
        gradient: Gradient {
            GradientStop { position: 0.0; color: "transparent" }
            GradientStop { position: 1.0; color: "#f4050d14" }
        }
    }

    // Clock
    Column {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.margins: 55 * s
        spacing: 6 * s
        opacity: root.ui

        Row {
            spacing: 8 * s
            Rectangle {
                width: 5 * s
                height: 5 * s
                color: root.lightCyan
                anchors.verticalCenter: parent.verticalCenter
            }
            Text {
                text: Qt.formatDate(new Date(), "dddd, MMMM d").toUpperCase()
                color: root.cleanWhite
                font.family: pf.name
                font.pixelSize: 13 * s
                font.letterSpacing: 2 * s
                font.bold: true
                anchors.verticalCenter: parent.verticalCenter
            }
        }

        Text {
            id: clockText
            text: Qt.formatTime(new Date(), "HH:mm")
            color: root.cleanWhite
            font.family: pf.name
            font.pixelSize: 76 * s
            font.bold: true

            Timer {
                interval: 1000
                running: true
                repeat: true
                onTriggered: clockText.text = Qt.formatTime(new Date(), "HH:mm")
            }
        }
    }

    // Login
    Item {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 55 * s
        opacity: root.ui
        width: 320 * s
        height: loginCol.implicitHeight

        Column {
            id: loginCol
            width: parent.width
            spacing: 12 * s

            // Username
            Text {
                text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : (userModel.lastUser || "User")).toUpperCase()
                color: userMouse.containsMouse ? root.lightCyan : root.cleanWhite
                font.family: pf.name
                font.pixelSize: 22 * s
                font.letterSpacing: 4 * s
                font.bold: true
                anchors.right: parent.right

                MouseArea {
                    id: userMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (typeof userModel !== "undefined" && userModel.rowCount() > 0)
                            root.userIndex = (root.userIndex + 1) % userModel.rowCount()
                    }
                }

                Behavior on color { ColorAnimation { duration: 120 } }
            }

            // Password
            Item {
                width: parent.width
                height: 28 * s
                anchors.right: parent.right

                TextInput {
                    id: pwd
                    anchors.fill: parent
                    color: root.lightCyan
                    font.family: pf.name
                    font.pixelSize: 18 * s
                    font.letterSpacing: 4 * s
                    echoMode: TextInput.Password
                    passwordCharacter: "■"
                    onTextEdited: errText.text = ""
                    focus: true
                    clip: true
                    horizontalAlignment: TextInput.AlignRight
                    verticalAlignment: TextInput.AlignVCenter
                    cursorVisible: false
                    cursorDelegate: Item { width: 0; height: 0 }
                    selectionColor: root.watermarkTeal

                    property bool wasClicked: false
                    onActiveFocusChanged: if (!activeFocus && text.length === 0) wasClicked = false

                    Keys.onReturnPressed: doLogin()
                    Keys.onEnterPressed: doLogin()
                }

                Text {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "ENTER PASSWORD"
                    color: root.deepIceBlue
                    font.family: pf.name
                    font.pixelSize: 12 * s
                    font.letterSpacing: 2 * s
                    opacity: pwd.text.length === 0 ? 0.75 : 0
                    Behavior on opacity { NumberAnimation { duration: 180 } }
                }

                // Cursor
                Rectangle {
                    id: customCursor
                    width: 2 * s
                    height: 16 * s
                    color: root.lightCyan
                    anchors.verticalCenter: parent.verticalCenter
                    x: pwd.cursorRectangle.x
                    visible: pwd.focus && (pwd.text.length > 0 || pwd.wasClicked)

                    SequentialAnimation {
                        loops: Animation.Infinite
                        running: customCursor.visible
                        NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0.1; duration: 400 }
                        NumberAnimation { target: customCursor; property: "opacity"; from: 0.1; to: 1; duration: 400 }
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        pwd.forceActiveFocus()
                        pwd.wasClicked = true
                    }
                }
            }

            // Login Button
            Item {
                anchors.right: parent.right
                width: 120 * s
                height: 24 * s

                Text {
                    id: btnText
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "➔ LOGIN"
                    color: loginMouse.containsMouse ? root.lightCyan : root.deepIceBlue
                    font.family: pf.name
                    font.pixelSize: 12 * s
                    font.letterSpacing: 2 * s
                    font.bold: true
                    Behavior on color { ColorAnimation { duration: 120 } }
                }

                MouseArea {
                    id: loginMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: doLogin()
                }
            }

            Text {
                id: errText
                text: ""
                height: 12 * s
                verticalAlignment: Text.AlignBottom
                color: "#ff5555"
                anchors.right: parent.right
                font.family: pf.name
                font.pixelSize: 11 * s
                font.bold: true
                font.letterSpacing: 1 * s
            }
        }
    }

    // Actions
    Row {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: 40 * s
        spacing: 16 * s
        opacity: root.ui

        Repeater {
            model: [
                { l: (sessionHelper.currentItem && sessionHelper.currentItem.sName ? sessionHelper.currentItem.sName : "Session").toUpperCase(), a: 2 },
                { l: "REBOOT", a: 0 },
                { l: "POWER OFF", a: 1 }
            ]

            delegate: Item {
                visible: modelData.a === 2 ? !root.isQuickshell : true
                width: pmt.implicitWidth + 28 * s
                height: 30 * s

                Rectangle {
                    anchors.fill: parent
                    color: "transparent"
                    border.color: root.darkTealLine
                    border.width: 1.5 * s
                    opacity: pm.containsMouse ? 1.0 : 0.7
                    Behavior on opacity { NumberAnimation { duration: 120 } }

                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 1.5 * s
                        color: modelData.a === 2 ? root.lightCyan : root.darkTealLine
                        opacity: pm.containsMouse ? 0.35 : 0
                        Behavior on opacity { NumberAnimation { duration: 120 } }
                    }
                }

                Text {
                    id: pmt
                    anchors.centerIn: parent
                    text: modelData.l
                    color: root.cleanWhite
                    font.family: pf.name
                    font.pixelSize: 10 * s
                    font.letterSpacing: 2 * s
                    font.bold: true
                }

                MouseArea {
                    id: pm
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (modelData.a === 0) {
                            if (typeof sddm !== "undefined") sddm.reboot()
                        } else if (modelData.a === 1) {
                            if (typeof sddm !== "undefined") sddm.powerOff()
                        } else if (typeof sessionModel !== "undefined" && sessionModel.rowCount() > 0) {
                            root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount()
                        }
                    }
                }
            }
        }
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() {
            errText.text = "ACCESS DENIED";
            pwd.text = "";
            pwd.focus = true;
        }
    }

    function doLogin() {
        var u = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "");
        if (typeof sddm !== "undefined")
            sddm.login(u, pwd.text, root.sessionIndex)
    }
}
