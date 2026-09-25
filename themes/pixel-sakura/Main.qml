import Qt5Compat.GraphicalEffects
import QtQuick
import QtQuick.Window
import SddmComponents 2.0

Rectangle {
    id: root

    readonly property real s: Screen.height / 768
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property real ui: 0
    property bool sessionMenuOpen: false

    // Colors
    readonly property color sakuraPink: "#df7a8c"
    readonly property color slateDark: "#32354c"
    readonly property color slateMid: "#506275"
    readonly property color sunRed: "#e26b67"

    function doLogin() {
        var u = (userHelper.currentItem && userHelper.currentItem.uLogin) ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "");
        if (typeof sddm !== "undefined")
            sddm.login(u, pwdInput.text, root.sessionIndex);
    }

    width: Screen.width
    height: Screen.height
    color: "#ebf0f5"

    Component.onCompleted: {
        entryAnim.start();
        keyboard.numLock = true;
    }

    // Cursor Fix
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
    }

    FontLoader {
        id: pf
        source: "font/PixelifySans-Bold.ttf"
    }

    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null
        currentIndex: root.sessionIndex
        visible: false
        width: 100
        height: 100
        delegate: Item {
            property string sName: model.name || ""
        }
    }

    ListView {
        id: userHelper
        model: typeof userModel !== "undefined" ? userModel : null
        currentIndex: root.userIndex
        visible: false
        width: 100
        height: 100
        delegate: Item {
            property string uName: model.realName || model.name || ""
            property string uLogin: model.name || ""
        }
    }

    Timer {
        interval: 300
        running: true
        onTriggered: pwdInput.forceActiveFocus()
    }

    // Anim
    NumberAnimation {
        id: entryAnim
        target: root
        property: "ui"
        from: 0
        to: 1
        duration: 1600
        easing.type: Easing.OutCubic
    }

    // Background
    Rectangle {
        anchors.fill: parent
        color: "#ebf0f5"
        z: -3
    }

    Loader {
        anchors.fill: parent
        source: "BackgroundVideo.qml"
    }

    // Clock
    Item {
        id: clockZone
        width: 300 * s
        height: 120 * s
        anchors.left: parent.left
        anchors.leftMargin: 40 * s
        anchors.top: parent.top
        anchors.topMargin: 40 * s
        opacity: root.ui

        Text {
            id: clockText
            text: Qt.formatTime(new Date(), "HH:mm")
            color: root.slateDark
            font.family: pf.name
            font.pixelSize: 64 * s
            font.bold: true
            anchors.left: parent.left
            anchors.top: parent.top

            Timer {
                interval: 1000
                running: true
                repeat: true
                onTriggered: clockText.text = Qt.formatTime(new Date(), "HH:mm")
            }
        }

        Text {
            id: dateText
            text: Qt.formatDate(new Date(), "dddd, MMMM d").toUpperCase()
            color: root.sakuraPink
            font.family: pf.name
            font.pixelSize: 11 * s
            font.letterSpacing: 2 * s
            font.bold: true
            anchors.left: parent.left
            anchors.top: clockText.bottom
            anchors.topMargin: -4 * s
        }

        // Track
        Item {
            width: 120 * s
            height: 8 * s
            anchors.top: dateText.bottom
            anchors.topMargin: 12 * s
            anchors.left: parent.left

            Rectangle {
                width: parent.width
                height: 1.5 * s
                color: root.slateMid
                opacity: 0.25
                anchors.verticalCenter: parent.verticalCenter
            }

            Row {
                anchors.fill: parent
                spacing: 10 * s

                Repeater {
                    model: 11
                    Rectangle {
                        width: 2 * s
                        height: 6 * s
                        color: root.slateMid
                        opacity: 0.35
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }
        }
    }

    // Login
    Item {
        id: loginZone
        width: 240 * s
        height: 120 * s
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: 60 * s
        opacity: root.ui

        // Username
        Item {
            id: userPill
            width: parent.width
            height: 28 * s
            anchors.top: parent.top
            anchors.horizontalCenter: parent.horizontalCenter

            Text {
                text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : (userModel.lastUser || "Trainer")).toUpperCase()
                color: userMouse.containsMouse ? root.sakuraPink : root.slateDark
                font.family: pf.name
                font.pixelSize: 18 * s
                font.bold: true
                font.letterSpacing: 4 * s
                anchors.centerIn: parent

                Behavior on color {
                    ColorAnimation {
                        duration: 150
                    }
                }
            }

            MouseArea {
                id: userMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (typeof userModel !== "undefined" && userModel.rowCount() > 1)
                        root.userIndex = (root.userIndex + 1) % userModel.rowCount();
                }
            }
        }

        // Password
        Item {
            id: pwdContainer
            width: 180 * s
            height: 30 * s
            anchors.top: userPill.bottom
            anchors.topMargin: 8 * s
            anchors.horizontalCenter: parent.horizontalCenter

            // Underline
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1 * s
                color: pwdInput.activeFocus ? root.sakuraPink : root.slateMid
                opacity: pwdInput.activeFocus ? 0.8 : 0.35
                Behavior on color { ColorAnimation { duration: 150 } }
                Behavior on opacity { NumberAnimation { duration: 150 } }
            }

            TextInput {
                id: pwdInput
                property bool wasClicked: false
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                horizontalAlignment: TextInput.AlignHCenter
                verticalAlignment: TextInput.AlignVCenter
                color: root.slateDark
                font.family: pf.name
                font.pixelSize: 15 * s
                font.letterSpacing: 4 * s
                echoMode: TextInput.Password
                passwordCharacter: "■"
                onTextEdited: errText.text = ""
                focus: true
                clip: true
                cursorVisible: false
                selectionColor: root.sakuraPink
                Keys.onReturnPressed: doLogin()
                Keys.onEnterPressed: doLogin()

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        pwdInput.forceActiveFocus();
                        pwdInput.wasClicked = true;
                    }
                }

                cursorDelegate: Item {
                    width: 0
                    height: 0
                }
            }

            Text {
                anchors.centerIn: parent
                text: "PASSWORD"
                color: root.slateMid
                font.family: pf.name
                font.pixelSize: 10 * s
                font.letterSpacing: 2 * s
                opacity: pwdInput.text.length === 0 ? 0.6 : 0

                Behavior on opacity {
                    NumberAnimation {
                        duration: 150
                    }
                }
            }

            // Cursor
            Rectangle {
                id: customCursor
                width: 6 * s
                height: pwdInput.cursorRectangle.height
                color: root.sakuraPink
                y: pwdInput.y + pwdInput.cursorRectangle.y
                x: pwdInput.x + pwdInput.cursorRectangle.x
                visible: pwdInput.focus && (pwdInput.text.length > 0 || pwdInput.wasClicked)

                SequentialAnimation {
                    loops: Animation.Infinite
                    running: customCursor.visible
                    NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0.1; duration: 400 }
                    NumberAnimation { target: customCursor; property: "opacity"; from: 0.1; to: 1; duration: 400 }
                }
            }
        }

        // Error
        Text {
            id: errText
            text: ""
            color: root.sunRed
            font.family: pf.name
            font.pixelSize: 9 * s
            font.bold: true
            font.letterSpacing: 2 * s
            anchors.top: pwdContainer.bottom
            anchors.topMargin: 8 * s
            anchors.horizontalCenter: parent.horizontalCenter
        }

        // Actions
        Row {
            id: actionZone
            anchors.top: pwdContainer.bottom
            anchors.topMargin: 25 * s
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 16 * s

            // Session
            Item {
                width: 90 * s
                height: 24 * s

                Row {
                    spacing: 5 * s
                    anchors.centerIn: parent

                    Text {
                        text: "✦"
                        font.pixelSize: 8 * s
                        color: sessMouse.containsMouse ? root.sakuraPink : root.slateMid
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        id: sessText
                        text: (sessionHelper.currentItem && sessionHelper.currentItem.sName ? sessionHelper.currentItem.sName : "SESSION").toUpperCase()
                        color: sessMouse.containsMouse ? root.sakuraPink : root.slateMid
                        font.family: pf.name
                        font.pixelSize: 11 * s
                        font.bold: true
                        font.letterSpacing: 1.5 * s
                        anchors.verticalCenter: parent.verticalCenter

                        Behavior on color {
                            ColorAnimation {
                                duration: 150
                            }
                        }
                    }
                }

                MouseArea {
                    id: sessMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (typeof sessionModel !== "undefined" && sessionModel.rowCount() > 1) {
                            root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount()
                        }
                    }
                }
            }

            // Repeater
            Repeater {
                model: [{
                    "label": "RESTART",
                    "action": 0
                }, {
                    "label": "SHUT DOWN",
                    "action": 1
                }]

                delegate: Item {
                    width: modelData.label === "SHUT DOWN" ? 104 * s : 86 * s
                    height: 24 * s

                    Row {
                        spacing: 5 * s
                        anchors.centerIn: parent

                        Text {
                            text: "✦"
                            font.pixelSize: 8 * s
                            color: powerMouse.containsMouse ? root.sakuraPink : root.slateMid
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        Text {
                            text: modelData.label
                            color: powerMouse.containsMouse ? root.sakuraPink : root.slateMid
                            font.family: pf.name
                            font.pixelSize: 11 * s
                            font.bold: true
                            font.letterSpacing: 1.5 * s
                            anchors.verticalCenter: parent.verticalCenter

                            Behavior on color {
                                ColorAnimation {
                                    duration: 150
                                }
                            }
                        }
                    }

                    MouseArea {
                        id: powerMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (modelData.action === 0) {
                                if (typeof sddm !== "undefined")
                                    sddm.reboot();
                            } else {
                                if (typeof sddm !== "undefined")
                                    sddm.powerOff();
                            }
                        }
                    }
                }
            }
        }

        transform: Translate {
            id: shakeTranslate
            x: 0
        }
    }

    // Shake
    SequentialAnimation {
        id: shakeAnim

        NumberAnimation {
            target: shakeTranslate
            property: "x"
            to: 8 * s
            duration: 50
        }

        NumberAnimation {
            target: shakeTranslate
            property: "x"
            to: -6 * s
            duration: 50
        }

        NumberAnimation {
            target: shakeTranslate
            property: "x"
            to: 4 * s
            duration: 50
        }

        NumberAnimation {
            target: shakeTranslate
            property: "x"
            to: -2 * s
            duration: 50
        }

        NumberAnimation {
            target: shakeTranslate
            property: "x"
            to: 0
            duration: 50
        }
    }

    Connections {
        function onLoginFailed() {
            errText.text = "ACCESS DENIED";
            pwdInput.text = "";
            pwdInput.focus = true;
            shakeAnim.start();
        }

        target: typeof sddm !== "undefined" ? sddm : null
    }
}
