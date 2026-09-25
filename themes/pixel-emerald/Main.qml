import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import Qt.labs.folderlistmodel
import SddmComponents 2.0

// Pixel Emerald
Rectangle {
    MouseArea { anchors.fill: parent; cursorShape: Qt.ArrowCursor; z: -1 }

    readonly property real s: Screen.height / 768
    id: root; width: Screen.width; height: Screen.height; color: "#2ab8b8"

    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: userModel.lastIndex >= 0 ? userModel.lastIndex : 0
    property real ui: 0
    property bool sessionMenuOpen: false

    // Colors
    readonly property color emerald:     "#3ec878"
    readonly property color emeraldDark: "#1a6e3c"
    readonly property color gold:        "#f0c040"
    readonly property color teal:        "#38c8c8"
    readonly property color cardBg:      "#d8f5e8"
    readonly property color cardBg2:     "#c0ecdb"
    readonly property color inkDark:     "#1a2e1a"
    readonly property color inkMid:      "#2a6040"
    readonly property color accentRed:   "#d44040"
    readonly property color accentBlue:  "#4080d0"

    FolderListModel { id: fontFolder; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
    FontLoader { id: pf; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }

    ListView { id: sessionHelper; model: typeof sessionModel !== "undefined" ? sessionModel : null; currentIndex: root.sessionIndex; opacity: 0; width: 100; height: 100; z: -100; delegate: Item { property string sName: model.name || "" } }
    ListView { id: userHelper; model: typeof userModel !== "undefined" ? userModel : null; currentIndex: root.userIndex; opacity: 0; width: 100; height: 100; z: -100; delegate: Item { property string uName: model.realName || model.name || ""; property string uLogin: model.name || "" } }

    Timer { interval: 300; running: true; onTriggered: pwd.forceActiveFocus() }

    Component.onCompleted: { entryAnim.start(); keyboard.numLock = true }
    ParallelAnimation {
        id: entryAnim
        NumberAnimation { target: root; property: "ui"; from: 0; to: 1; duration: 1200; easing.type: Easing.OutCubic }
    }

    Loader { anchors.fill: parent; source: "BackgroundVideo.qml"; opacity: root.ui }

    // Clock
    Item {
        anchors.top: parent.top; anchors.left: parent.left
        anchors.topMargin: (22 + 18 * root.ui) * s; anchors.leftMargin: 44 * s
        opacity: root.ui

        Rectangle { width: clockCard.width + 4 * s; height: clockCard.height + 4 * s; x: 3 * s; y: 4 * s; radius: clockCard.radius + 1; color: "#60000000"; z: -1 }

        Rectangle {
            id: clockCard
            width: 250 * s; height: 90 * s
            color: root.cardBg; radius: 6 * s
            border.color: root.emeraldDark; border.width: 2 * s

            Rectangle {
                id: clockHeader
                width: parent.width; height: 18 * s
                color: root.emeraldDark; radius: 5 * s
                anchors.top: parent.top
                Rectangle { width: parent.width; height: 8 * s; color: root.emeraldDark; anchors.bottom: parent.bottom }

                Rectangle {
                    id: ledDot
                    width: 8 * s; height: 8 * s; radius: 4 * s
                    color: root.accentRed
                    anchors.left: parent.left; anchors.leftMargin: 10 * s
                    anchors.verticalCenter: parent.verticalCenter
                    SequentialAnimation {
                        loops: Animation.Infinite; running: true
                        NumberAnimation { target: ledDot; property: "opacity"; from: 1; to: 0.3; duration: 800 }
                        NumberAnimation { target: ledDot; property: "opacity"; from: 0.3; to: 1; duration: 800 }
                    }
                }

                Text { anchors.centerIn: parent; text: "CHRONO INTERFACE"; color: "#ccffee"; font.family: pf.name; font.pixelSize: 7 * s; font.letterSpacing: 2 * s }

                Row {
                    anchors.right: parent.right; anchors.rightMargin: 8 * s
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3 * s
                    Repeater { model: 3; Rectangle { width: 4 * s; height: 8 * s; radius: 1 * s; color: root.accentBlue; opacity: 0.8 } }
                }
            }

            Text {
                id: ct
                anchors.left: parent.left; anchors.leftMargin: 12 * s
                anchors.top: clockHeader.bottom; anchors.topMargin: 6 * s
                text: Qt.formatTime(new Date(), "HH:mm")
                color: root.inkDark; font.family: pf.name; font.pixelSize: 40 * s
                Timer { interval: 1000; running: true; repeat: true; onTriggered: ct.text = Qt.formatTime(new Date(), "HH:mm") }
            }

            Rectangle { anchors.left: parent.left; anchors.leftMargin: 12 * s; anchors.right: parent.right; anchors.rightMargin: 12 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 17 * s; height: 1 * s; color: root.emerald; opacity: 0.5 }

            Text { anchors.left: parent.left; anchors.leftMargin: 13 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 7 * s; text: Qt.formatDate(new Date(), "dddd, MMMM d").toUpperCase(); color: root.inkMid; font.family: pf.name; font.pixelSize: 8 * s; font.letterSpacing: 1.5 * s }

            Row {
                anchors.right: parent.right; anchors.rightMargin: 10 * s
                anchors.bottom: parent.bottom; anchors.bottomMargin: 9 * s
                spacing: 4 * s
                Repeater { model: [root.emerald, root.accentBlue, root.gold, root.emeraldDark]; Rectangle { width: 6 * s; height: 6 * s; radius: 1 * s; color: modelData; opacity: 0.85 } }
            }
        }
    }

    // Power
    Row {
        anchors.top: parent.top; anchors.right: parent.right
        anchors.topMargin: (22 + 18 * root.ui) * s; anchors.rightMargin: 44 * s
        spacing: 12 * s; opacity: root.ui

        Repeater {
            model: [{ l: "RESTART", a: 0 }, { l: "POWER OFF", a: 1 }]
            delegate: Rectangle {
                width: pwrTxt.implicitWidth + 28 * s; height: 30 * s
                color: pwrMouse.containsMouse ? root.emeraldDark : root.cardBg
                radius: 5 * s
                border.color: pwrMouse.containsMouse ? root.gold : root.emeraldDark
                border.width: 2 * s
                Behavior on color { ColorAnimation { duration: 180 } }

                Rectangle { width: parent.width; height: parent.height; x: 2 * s; y: 3 * s; radius: parent.radius; color: "#50000000"; z: -1 }

                Text {
                    id: pwrTxt; anchors.centerIn: parent; text: modelData.l
                    color: pwrMouse.containsMouse ? "#ccffee" : root.inkDark
                    font.family: pf.name; font.pixelSize: 8 * s; font.letterSpacing: 1.5 * s
                    Behavior on color { ColorAnimation { duration: 180 } }
                }

                MouseArea {
                    id: pwrMouse; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (modelData.a === 0) { if (typeof sddm !== "undefined") sddm.reboot() }
                        else if (modelData.a === 1) { if (typeof sddm !== "undefined") sddm.powerOff() }
                    }
                }
            }
        }
    }

    // Login
    Item {
        anchors.bottom: parent.bottom; anchors.right: parent.right
        anchors.bottomMargin: (24 + 18 * root.ui) * s; anchors.rightMargin: 44 * s
        width: 270 * s; height: 178 * s
        opacity: root.ui

        Rectangle { width: loginCard.width; height: loginCard.height; x: 3 * s; y: 4 * s; radius: loginCard.radius; color: "#60000000"; z: -1 }

        Rectangle {
            id: loginCard
            width: 270 * s; height: 178 * s
            color: root.cardBg; radius: 6 * s
            border.color: root.emeraldDark; border.width: 2 * s

            Rectangle {
                id: loginHeader
                width: parent.width; height: 22 * s
                color: root.emeraldDark; radius: 5 * s
                anchors.top: parent.top
                Rectangle { width: parent.width; height: 8 * s; color: root.emeraldDark; anchors.bottom: parent.bottom }

                Rectangle {
                    id: loginLed
                    width: 8 * s; height: 8 * s; radius: 4 * s
                    color: root.accentRed
                    anchors.left: parent.left; anchors.leftMargin: 10 * s
                    anchors.verticalCenter: parent.verticalCenter
                    SequentialAnimation {
                        loops: Animation.Infinite; running: true
                        NumberAnimation { target: loginLed; property: "opacity"; from: 1; to: 0.3; duration: 900 }
                        NumberAnimation { target: loginLed; property: "opacity"; from: 0.3; to: 1; duration: 900 }
                    }
                }

                Text { anchors.centerIn: parent; text: "SYSTEM AUTHENTICATION"; color: "#ccffee"; font.family: pf.name; font.pixelSize: 7 * s; font.letterSpacing: 2 * s }

                Row {
                    anchors.right: parent.right; anchors.rightMargin: 8 * s
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3 * s
                    Repeater { model: 3; Rectangle { width: 4 * s; height: 8 * s; radius: 1 * s; color: root.accentBlue; opacity: 0.8 } }
                }
            }

            Item {
                id: unRow
                anchors.top: loginHeader.bottom; anchors.topMargin: 10 * s
                anchors.left: parent.left; anchors.leftMargin: 14 * s
                anchors.right: parent.right; anchors.rightMargin: 14 * s
                height: 32 * s

                Rectangle { anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; width: trainerLabel.implicitWidth + 14 * s; height: 18 * s; radius: 3 * s; color: root.emerald; opacity: 0.3 }
                Text { id: trainerLabel; text: "TRAINER"; anchors.left: parent.left; anchors.leftMargin: 7 * s; anchors.verticalCenter: parent.verticalCenter; color: root.inkMid; font.family: pf.name; font.pixelSize: 8 * s; font.letterSpacing: 1 * s }
                Text { id: un; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : (userModel.lastUser || "Trainer")).toUpperCase(); color: root.inkDark; font.family: pf.name; font.pixelSize: 14 * s; font.letterSpacing: 1 * s }
                MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: { if (typeof userModel !== "undefined" && userModel.rowCount() > 0) root.userIndex = (root.userIndex + 1) % userModel.rowCount() } }
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1 * s; color: root.emerald; opacity: 0.4 }
            }

            Item {
                id: pwRow
                anchors.top: unRow.bottom; anchors.topMargin: 4 * s
                anchors.left: parent.left; anchors.leftMargin: 14 * s
                anchors.right: parent.right; anchors.rightMargin: 14 * s
                height: 40 * s

                Rectangle { anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; width: passkeyLabel.implicitWidth + 14 * s; height: 18 * s; radius: 3 * s; color: root.emerald; opacity: 0.3 }
                Text { id: passkeyLabel; text: "PASSKEY"; anchors.left: parent.left; anchors.leftMargin: 7 * s; anchors.verticalCenter: parent.verticalCenter; color: root.inkMid; font.family: pf.name; font.pixelSize: 8 * s; font.letterSpacing: 1 * s }

                TextInput {
                    id: pwd
                    anchors.right: parent.right; width: parent.width * 0.58; height: parent.height
                    color: root.inkDark; font.family: pf.name; font.pixelSize: 15 * s; font.letterSpacing: 4 * s
                    echoMode: TextInput.Password; passwordCharacter: "●"
                    onTextEdited: err.text = ""
                    focus: true; clip: true
                    horizontalAlignment: TextInput.AlignRight; verticalAlignment: TextInput.AlignVCenter
                    cursorVisible: false; cursorDelegate: Item { width: 0; height: 0 }
                    selectionColor: root.emerald
                    property bool wasClicked: false
                    Keys.onReturnPressed: doLogin()
                    Keys.onEnterPressed: doLogin()

                    Rectangle {
                        id: pixCursor
                        width: 2 * s; height: pwd.font.pixelSize * 1.1
                        color: root.gold; radius: 1 * s
                        x: pwd.cursorRectangle.x
                        anchors.verticalCenter: parent.verticalCenter
                        visible: pwd.activeFocus && (pwd.text.length > 0 || pwd.wasClicked)
                        SequentialAnimation {
                            loops: Animation.Infinite; running: pixCursor.visible
                            NumberAnimation { target: pixCursor; property: "opacity"; from: 1; to: 0.05; duration: 450 }
                            NumberAnimation { target: pixCursor; property: "opacity"; from: 0.05; to: 1; duration: 450 }
                        }
                    }

                    MouseArea { anchors.fill: parent; onClicked: { pwd.forceActiveFocus(); pwd.wasClicked = true } }
                }

                Text { anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter; text: "ENTER PIN"; color: root.inkDark; opacity: pwd.text.length === 0 ? 0.28 : 0; font.family: pf.name; font.pixelSize: 9 * s; font.letterSpacing: 1 * s; Behavior on opacity { NumberAnimation { duration: 150 } } }
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1.5 * s; color: root.gold; opacity: pwd.activeFocus ? 1 : 0.2; Behavior on opacity { NumberAnimation { duration: 200 } } }
            }

            Rectangle {
                width: parent.width; height: 44 * s; color: "transparent"
                anchors.bottom: parent.bottom

                Text { id: err; text: ""; anchors.left: parent.left; anchors.leftMargin: 14 * s; anchors.verticalCenter: parent.verticalCenter; color: "#d44040"; font.family: pf.name; font.pixelSize: 8 * s }

                Rectangle {
                    id: loginBtn
                    anchors.right: parent.right; anchors.rightMargin: 14 * s
                    anchors.verticalCenter: parent.verticalCenter
                    width: 84 * s; height: 26 * s; radius: 4 * s
                    color: lbm.containsMouse ? root.emeraldDark : "transparent"
                    border.color: lbm.containsMouse ? root.gold : root.emeraldDark
                    border.width: 2 * s
                    Behavior on color { ColorAnimation { duration: 180 } }

                    Rectangle { width: parent.width; height: parent.height; x: 2 * s; y: 2 * s; radius: parent.radius; color: "#40000000"; z: -1; visible: lbm.containsMouse }

                    Text { anchors.centerIn: parent; text: "ACCESS"; color: lbm.containsMouse ? "#ccffee" : root.inkDark; font.family: pf.name; font.pixelSize: 9 * s; font.letterSpacing: 2 * s; Behavior on color { ColorAnimation { duration: 180 } } }
                    MouseArea { id: lbm; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: doLogin() }
                }
            }
        }
    }

    // Session
    Item {
        id: sessionWidget
        anchors.bottom: parent.bottom; anchors.left: parent.left
        anchors.bottomMargin: (24 + 18 * root.ui) * s; anchors.leftMargin: 44 * s
        width: 240 * s
        height: sessionPill.height + sessionDropdown.height
        opacity: root.ui
        visible: !root.isQuickshell
        z: 10

        Item {
            id: sessionDropdown
            anchors.bottom: sessionPill.top; anchors.bottomMargin: 4 * s
            width: parent.width
            height: root.sessionMenuOpen
                    ? (36 * s * (typeof sessionModel !== "undefined" ? sessionModel.rowCount() : 0)) + 8 * s
                    : 0
            clip: true
            Behavior on height { NumberAnimation { duration: 340; easing.type: Easing.OutExpo } }

            Rectangle {
                anchors.fill: parent
                color: root.cardBg; radius: 6 * s
                border.color: root.emeraldDark; border.width: 2 * s
                Rectangle { width: parent.width; height: parent.height; x: 3 * s; y: 4 * s; radius: parent.radius; color: "#60000000"; z: -1 }
            }

            Rectangle { anchors.left: parent.left; anchors.leftMargin: 8 * s; anchors.top: parent.top; anchors.topMargin: 6 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 6 * s; width: 2 * s; color: root.emerald; opacity: 0.5; radius: 1 * s }

            Column {
                anchors.fill: parent
                anchors.leftMargin: 18 * s; anchors.rightMargin: 10 * s
                anchors.topMargin: 4 * s; anchors.bottomMargin: 4 * s
                spacing: 0

                Repeater {
                    model: typeof sessionModel !== "undefined" ? sessionModel : null
                    delegate: Item {
                        width: parent.width; height: 36 * s
                        property bool isActive: root.sessionIndex === index
                        property bool hovered: sItemMa.containsMouse

                        Rectangle { anchors.fill: parent; anchors.margins: 2 * s; radius: 4 * s; color: root.emerald; opacity: (isActive || hovered) ? 0.15 : 0; Behavior on opacity { NumberAnimation { duration: 180 } } }
                        Rectangle { width: 5 * s; height: 5 * s; radius: 2.5 * s; color: root.gold; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; opacity: isActive ? 1 : 0; Behavior on opacity { NumberAnimation { duration: 200 } } }

                        Text {
                            text: model.name.toUpperCase()
                            font.family: pf.name; font.pixelSize: 9 * s
                            font.letterSpacing: (isActive || hovered) ? 2 * s : 1 * s
                            color: isActive ? root.inkDark : root.inkMid
                            opacity: (isActive || hovered) ? 1.0 : 0.55
                            anchors.left: parent.left; anchors.leftMargin: 12 * s
                            anchors.verticalCenter: parent.verticalCenter
                            Behavior on opacity { NumberAnimation { duration: 180 } }
                            Behavior on font.letterSpacing { NumberAnimation { duration: 300; easing.type: Easing.OutQuart } }
                            Behavior on color { ColorAnimation { duration: 180 } }
                        }

                        MouseArea { id: sItemMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: { root.sessionIndex = index; root.sessionMenuOpen = false } }
                    }
                }
            }
        }

        Rectangle {
            id: sessionPill
            anchors.bottom: parent.bottom
            width: parent.width; height: 44 * s
            color: sessMa.containsMouse ? root.cardBg2 : root.cardBg
            radius: 6 * s
            border.color: (root.sessionMenuOpen || sessMa.containsMouse) ? root.gold : root.emeraldDark
            border.width: 2 * s
            Behavior on color { ColorAnimation { duration: 180 } }
            Behavior on border.color { ColorAnimation { duration: 180 } }

            Rectangle { width: parent.width; height: parent.height; x: 3 * s; y: 4 * s; radius: parent.radius; color: "#60000000"; z: -1 }

            Rectangle { anchors.left: parent.left; anchors.leftMargin: 10 * s; anchors.verticalCenter: parent.verticalCenter; width: sessLabel.implicitWidth + 14 * s; height: 18 * s; radius: 3 * s; color: root.emerald; opacity: 0.3 }
            Text { id: sessLabel; text: "SESSION"; anchors.left: parent.left; anchors.leftMargin: 17 * s; anchors.verticalCenter: parent.verticalCenter; color: root.inkMid; font.family: pf.name; font.pixelSize: 8 * s; font.letterSpacing: 1 * s }

            Row {
                anchors.right: parent.right; anchors.rightMargin: 10 * s
                anchors.verticalCenter: parent.verticalCenter
                spacing: 5 * s

                Text {
                    id: sessionText
                    text: (sessionHelper.currentItem && sessionHelper.currentItem.sName ? sessionHelper.currentItem.sName : "Default").toUpperCase()
                    color: (root.sessionMenuOpen || sessMa.containsMouse) ? root.emeraldDark : root.inkDark
                    font.family: pf.name; font.pixelSize: 11 * s
                    font.letterSpacing: (root.sessionMenuOpen || sessMa.containsMouse) ? 2 * s : 1 * s
                    anchors.verticalCenter: parent.verticalCenter
                    Behavior on color { ColorAnimation { duration: 180 } }
                    Behavior on font.letterSpacing { NumberAnimation { duration: 300; easing.type: Easing.OutQuart } }
                }

                Text {
                    text: "▲"; font.pixelSize: 7 * s; color: root.gold
                    opacity: (root.sessionMenuOpen || sessMa.containsMouse) ? 1 : 0.4
                    rotation: root.sessionMenuOpen ? 0 : 180
                    anchors.verticalCenter: parent.verticalCenter
                    Behavior on rotation { NumberAnimation { duration: 280; easing.type: Easing.OutCubic } }
                    Behavior on opacity { NumberAnimation { duration: 180 } }
                }
            }

            MouseArea { id: sessMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor; onClicked: root.sessionMenuOpen = !root.sessionMenuOpen }
        }
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() { err.text = "ACCESS DENIED"; pwd.text = ""; pwd.focus = true }
    }

    function doLogin() {
        var u = (userHelper.currentItem && userHelper.currentItem.uLogin)
                ? userHelper.currentItem.uLogin
                : (typeof userModel !== "undefined" ? userModel.lastUser : "")
        if (typeof sddm !== "undefined") sddm.login(u, pwd.text, root.sessionIndex)
    }
}
