import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import QtMultimedia
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
    color: "#060a14"
    readonly property real s: (Screen.height / 768) * 0.75

    // Colors
    readonly property color srGold:        "#c8a96e"
    readonly property color srGoldLight:   "#e8cfa0"
    readonly property color srGoldDim:     "#8a7040"
    readonly property color srBlue:        "#7ec8e8"
    readonly property color srBlueDim:     "#4a8aaa"
    readonly property color srBlueGlow:    "#aaddff"
    readonly property color srPurple:      "#9b7dcc"
    readonly property color srPurpleDim:   "#6b5090"
    readonly property color srWhite:       "#eef2f8"
    readonly property color srGhost:       "#8899bb"
    readonly property color srPanel:       "#0d1420"
    readonly property color srPanelDark:   "#080d18"

    // Quickshell
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined

    // State
    property real uiOpacity: 0
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    property bool sessionPopupOpen: false

    TextConstants { id: textConstants }

    FolderListModel {
        id: fontFolder
        folder: Qt.resolvedUrl("font")
        nameFilters: ["*.ttf", "*.otf"]
    }
    FontLoader {
        id: mainFont
        source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : ""
    }

    // Helpers
    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null; currentIndex: root.sessionIndex
        opacity: 0; width: 100; height: 100; z: -100
        delegate: Item { property string sName: model.name || "" }
    }

    // Background
    Item {
        id: bgContainer; anchors.fill: parent; clip: true
        MediaPlayer {
            id: bgVideoPlayer; source: "bg.mp4"; loops: MediaPlayer.Infinite; autoPlay: true; videoOutput: bgVideoOutput
        }
        VideoOutput { id: bgVideoOutput; anchors.fill: parent; fillMode: VideoOutput.PreserveAspectCrop }

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#22060a14" }
                GradientStop { position: 0.6; color: "transparent" }
                GradientStop { position: 1.0; color: "#bb060a14" }
            }
        }

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0;  color: "#cc060a14" }
                GradientStop { position: 0.35; color: "#55060a14" }
                GradientStop { position: 0.60; color: "transparent" }
            }
        }

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.72; color: "transparent" }
                GradientStop { position: 0.88; color: "#66060a14" }
                GradientStop { position: 1.0;  color: "#cc060a14" }
            }
        }

        Repeater {
            model: 55
            Item {
                property real px: Math.random() * root.width
                property real py: Math.random() * root.height * 0.7
                property real sz: (0.8 + Math.random() * 2.2) * s
                x: px; y: py
                Rectangle {
                    width: sz; height: width; radius: width / 2
                    color: Math.random() > 0.5 ? root.srBlue : root.srWhite; opacity: 0
                    SequentialAnimation on opacity {
                        loops: Animation.Infinite
                        PauseAnimation { duration: Math.random() * 6000 }
                        NumberAnimation { from: 0; to: Math.random() * 0.55 + 0.1; duration: 2000 + Math.random() * 2000; easing.type: Easing.OutQuad }
                        NumberAnimation { from: 0.55; to: 0; duration: 2500 + Math.random() * 2000; easing.type: Easing.InQuad }
                    }
                }
            }
        }
    }

    // Interface
    Item {
        id: mainUI; anchors.fill: parent; opacity: root.uiOpacity
        Component.onCompleted: NumberAnimation { target: root; property: "uiOpacity"; from: 0; to: 1; duration: 1600; easing.type: Easing.OutCubic }

        Item {
            id: userProfile
            anchors.left: parent.left; anchors.leftMargin: 40 * s
            anchors.top: parent.top; anchors.topMargin: 40 * s
            width: 260 * s; height: 60 * s
            
            Rectangle { anchors.fill: parent; radius: 30 * s; color: "black"; opacity: 0.2; anchors.margins: -2 * s }
            Rectangle { anchors.fill: parent; radius: 30 * s; color: "#cc0d1420"; border.color: "#33ffffff"; border.width: 1.2 * s }
            Rectangle {
                width: parent.width * 0.4; height: 1.5 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 8 * s
                anchors.left: avatarFrame.right; anchors.leftMargin: 12 * s; color: root.srGold; opacity: 0.5
            }
            Rectangle {
                id: avatarFrame; width: 48 * s; height: 48 * s; radius: 24 * s; anchors.left: parent.left; anchors.leftMargin: 6 * s
                anchors.verticalCenter: parent.verticalCenter; color: "#15ffffff"; border.color: root.srGold; border.width: 1.5 * s
                Text { text: "✦"; anchors.centerIn: parent; font.family: mainFont.name; font.pixelSize: 22 * s; color: root.srGold; opacity: 0.9 }
            }

            Column {
                anchors.left: avatarFrame.right; anchors.leftMargin: 12 * s; anchors.verticalCenter: parent.verticalCenter; spacing: 1 * s
                Text {
                    text: {
                        var name = (typeof userModel !== "undefined") ? (userModel.data(userModel.index(root.userIndex, 0), Qt.UserRole + 1) || userModel.lastUser || "USER") : "USER"
                        return name.toUpperCase()
                    }
                    font.family: mainFont.name; font.pixelSize: 18 * s; font.bold: true; color: "white"; font.letterSpacing: 0.4 * s
                }
                Text { text: "LV. 80 • ASTRAL EXPRESS"; font.family: mainFont.name; font.pixelSize: 9 * s; color: root.srGold; opacity: 0.6; font.letterSpacing: 1.5 * s }
            }

            MouseArea {
                anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                onClicked: { if (typeof userModel !== "undefined" && userModel.count > 0) root.userIndex = (root.userIndex + 1) % userModel.count }
            }
        }

        Column {
            id: rightActionCol; anchors.right: parent.right; anchors.rightMargin: 36 * s; anchors.top: parent.top; anchors.topMargin: 50 * s; spacing: 24 * s

            Item {
                width: 60 * s; height: 62 * s
                Canvas {
                    anchors.centerIn: parent; width: 26 * s; height: 26 * s; anchors.verticalCenterOffset: -10 * s
                    onPaint: { var ctx = getContext("2d"); ctx.clearRect(0,0,width,height); ctx.strokeStyle = "white"; ctx.lineWidth = 1.6 * s; ctx.strokeRect(2*s, 5*s, 22*s, 16*s); ctx.beginPath(); ctx.moveTo(6*s, 10*s); ctx.lineTo(20*s, 10*s); ctx.stroke(); ctx.beginPath(); ctx.moveTo(6*s, 14*s); ctx.lineTo(16*s, 14*s); ctx.stroke(); }
                }
                Text { text: "Notices"; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; font.family: mainFont.name; font.pixelSize: 10 * s; color: "white"; opacity: 0.8 }
            }

            Item {
                width: 60 * s; height: 62 * s
                Canvas {
                    anchors.centerIn: parent; width: 26 * s; height: 26 * s; anchors.verticalCenterOffset: -10 * s
                    onPaint: { var ctx = getContext("2d"); ctx.clearRect(0,0,width,height); ctx.strokeStyle = "white"; ctx.lineWidth = 1.6 * s; ctx.beginPath(); ctx.arc(width/2, height/2, 9*s, -Math.PI*0.8, Math.PI*0.8); ctx.stroke(); ctx.fillStyle = "white"; ctx.beginPath(); ctx.moveTo(5*s, 6*s); ctx.lineTo(11*s, 4*s); ctx.lineTo(9*s, 11*s); ctx.fill(); }
                }
                Text { text: "Update"; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; font.family: mainFont.name; font.pixelSize: 10 * s; color: "white"; opacity: 0.8 }
            }

            Item {
                width: 60 * s; height: 62 * s
                scale: rstMouse.containsMouse ? 1.05 : 1.0; Behavior on scale { NumberAnimation { duration: 250; easing.type: Easing.OutBack } }
                Item {
                    anchors.fill: parent; opacity: rstMouse.containsMouse ? 1 : 0; Behavior on opacity { NumberAnimation { duration: 200 } }
                    Rectangle { width: 1.5*s; height: 28*s; color: root.srGold; anchors.left: parent.left; anchors.leftMargin: -2*s; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: -10*s }
                    Rectangle { width: 1.5*s; height: 28*s; color: root.srGold; anchors.right: parent.right; anchors.rightMargin: -2*s; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: -10*s }
                }
                Canvas {
                    id: rstCanvas; anchors.centerIn: parent; width: 26 * s; height: 26 * s; anchors.verticalCenterOffset: -10 * s
                    onPaint: { var ctx = getContext("2d"); ctx.clearRect(0,0,width,height); ctx.strokeStyle = rstMouse.containsMouse ? root.srGoldLight : "white"; ctx.lineWidth = 1.6 * s; ctx.lineCap = "round"; ctx.beginPath(); ctx.arc(width/2, height/2, 9*s, -Math.PI*0.7, Math.PI*0.8); ctx.stroke(); ctx.fillStyle = ctx.strokeStyle; ctx.beginPath(); ctx.moveTo(width*0.2, height*0.2); ctx.lineTo(width*0.4, height*0.1); ctx.lineTo(width*0.35, height*0.35); ctx.closePath(); ctx.fill(); }
                    Connections { target: rstMouse; function onContainsMouseChanged() { rstCanvas.requestPaint() } }
                }
                Text { text: "Restart"; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; font.family: mainFont.name; font.pixelSize: 10 * s; color: rstMouse.containsMouse ? root.srGoldLight : "white"; opacity: 0.8 }
                MouseArea { id: rstMouse; anchors.fill: parent; hoverEnabled: true; onClicked: { if (typeof sddm !== "undefined") sddm.reboot() } }
            }

            Item {
                width: 60 * s; height: 62 * s
                scale: shtMouse.containsMouse ? 1.05 : 1.0; Behavior on scale { NumberAnimation { duration: 250; easing.type: Easing.OutBack } }
                Item {
                    anchors.fill: parent; opacity: shtMouse.containsMouse ? 1 : 0; Behavior on opacity { NumberAnimation { duration: 200 } }
                    Rectangle { width: 1.5*s; height: 28*s; color: root.srGold; anchors.left: parent.left; anchors.leftMargin: -2*s; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: -10*s }
                    Rectangle { width: 1.5*s; height: 28*s; color: root.srGold; anchors.right: parent.right; anchors.rightMargin: -2*s; anchors.verticalCenter: parent.verticalCenter; anchors.verticalCenterOffset: -10*s }
                }
                Canvas {
                    id: shtCanvas; anchors.centerIn: parent; width: 26 * s; height: 26 * s; anchors.verticalCenterOffset: -10 * s
                    onPaint: { var ctx = getContext("2d"); ctx.clearRect(0,0,width,height); ctx.strokeStyle = shtMouse.containsMouse ? root.srGoldLight : "white"; ctx.lineWidth = 1.6 * s; ctx.lineCap = "round"; ctx.beginPath(); ctx.moveTo(width/2, 6*s); ctx.lineTo(width/2, 14*s); ctx.stroke(); ctx.beginPath(); ctx.arc(width/2, height/2, 9*s, -Math.PI*0.6, -Math.PI*0.4, true); ctx.stroke(); }
                    Connections { target: shtMouse; function onContainsMouseChanged() { shtCanvas.requestPaint() } }
                }
                Text { text: "Power Off"; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter; font.family: mainFont.name; font.pixelSize: 10 * s; color: shtMouse.containsMouse ? root.srGoldLight : "white"; opacity: 0.8 }
                MouseArea { id: shtMouse; anchors.fill: parent; hoverEnabled: true; onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() } }
            }
        }

        Item {
            id: loginPanel; width: 440 * s; anchors.horizontalCenter: parent.horizontalCenter; anchors.bottom: footerBar.top; anchors.bottomMargin: 140 * s
            Column {
                anchors.centerIn: parent; width: parent.width; spacing: 16 * s
                Item {
                    id: passInContainer; width: 280 * s; height: 40 * s; anchors.horizontalCenter: parent.horizontalCenter
                    Rectangle { id: passLine; width: parent.width; height: 1.2 * s; anchors.bottom: parent.bottom; color: passIn.activeFocus ? root.srGold : "#44ffffff"; Behavior on color { ColorAnimation { duration: 200 } } }
                    TextInput {
                        id: passIn; anchors.fill: parent; anchors.bottomMargin: 4 * s; font.family: mainFont.name; font.pixelSize: 18 * s; color: "white"; echoMode: TextInput.Password; passwordCharacter: "✦"
                        verticalAlignment: TextInput.AlignBottom; horizontalAlignment: TextInput.AlignHCenter; cursorVisible: false; cursorDelegate: Item { width: 0; height: 0 }
                        selectionColor: root.srGold; property bool wasClicked: false; onActiveFocusChanged: if (!activeFocus && text.length === 0) wasClicked = false
                        onTextEdited: { errText.text = ""; digitAnim.restart(); jitterAnim.restart() }
                        onAccepted: doLogin()
                        Text { text: "ENTER PASSWORD"; font.family: mainFont.name; font.pixelSize: 12 * s; font.letterSpacing: 2 * s; color: "#66ffffff"; anchors.centerIn: parent; anchors.verticalCenterOffset: 4 * s; opacity: passIn.text.length === 0 ? 1.0 : 0; Behavior on opacity { NumberAnimation { duration: 400; easing.type: Easing.InOutSine } } }
                        Rectangle {
                            id: customCursor; width: 2 * s; height: 20 * s; color: root.srGold; anchors.verticalCenter: parent.verticalCenter; x: passIn.cursorRectangle.x; visible: passIn.focus && (passIn.text.length > 0 || passIn.wasClicked)
                            SequentialAnimation { loops: Animation.Infinite; running: customCursor.visible; NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0.05; duration: 450 } NumberAnimation { target: customCursor; property: "opacity"; from: 0.05; to: 1; duration: 450 } }
                        }
                        MouseArea { anchors.fill: parent; onClicked: { passIn.forceActiveFocus(); passIn.wasClicked = true } }
                    }
                    Rectangle { id: passPulse; width: parent.width; height: 2 * s; anchors.bottom: parent.bottom; color: root.srGoldLight; opacity: 0; SequentialAnimation { id: jitterAnim; NumberAnimation { target: passPulse; property: "opacity"; from: 0.8; to: 0; duration: 450 } } }
                    Rectangle { id: digitPulse; anchors.fill: parent; color: root.srGold; opacity: 0; SequentialAnimation { id: digitAnim; NumberAnimation { target: digitPulse; property: "opacity"; from: 0.3; to: 0; duration: 250 } } }
                }
                Text { id: errText; height: 14 * s; anchors.horizontalCenter: parent.horizontalCenter; text: ""; color: "#ff4444"; font.family: mainFont.name; font.pixelSize: 12 * s; font.letterSpacing: 1 * s }
                Item {
                    width: 300 * s; height: 44 * s; anchors.horizontalCenter: parent.horizontalCenter; visible: !root.isQuickshell
                    Rectangle { anchors.fill: parent; radius: 22 * s; color: sesMouse.containsMouse ? "#aa000000" : "#88000000"; border.color: sesMouse.containsMouse ? "#ccffffff" : "#44ffffff"; border.width: 1 * s; Behavior on color { ColorAnimation { duration: 150 } } }
                    Row {
                        anchors.centerIn: parent; spacing: 12 * s
                        Rectangle {
                            width: 22 * s; height: 22 * s; radius: 11 * s; color: "transparent"; border.color: root.srGold; border.width: 1.5 * s; anchors.verticalCenter: parent.verticalCenter
                            Canvas { anchors.fill: parent; onPaint: { var ctx = getContext("2d"); ctx.clearRect(0,0,width,height); ctx.strokeStyle = root.srGold; ctx.lineWidth = 1.5 * s; ctx.beginPath(); ctx.moveTo(width*0.3, height*0.5); ctx.lineTo(width*0.45, height*0.65); ctx.lineTo(width*0.7, height*0.35); ctx.stroke(); } }
                        }
                        Text { text: (typeof sessionModel !== "undefined" && sessionModel.count > root.sessionIndex && root.sessionIndex >= 0) ? sessionHelper.currentItem.sName : "Select Session"; font.family: mainFont.name; font.pixelSize: 17 * s; color: "white"; anchors.verticalCenter: parent.verticalCenter }
                    }
                    MouseArea { id: sesMouse; anchors.fill: parent; hoverEnabled: true; onClicked: root.sessionPopupOpen = true }
                }
            }
        }

        Item {
            id: footerBar; width: parent.width; height: 60 * s; anchors.bottom: parent.bottom
            Rectangle { anchors.fill: parent; gradient: Gradient { GradientStop { position: 0.0; color: "transparent" } GradientStop { position: 1.0; color: "#44000000" } } }
            Text {
                anchors.left: parent.left; anchors.leftMargin: 24 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 14 * s
                text: "OSPRODWin1.0.5_D7281944_A3819401_L1920844"; font.family: mainFont.name; font.pixelSize: 10 * s; color: "white"; opacity: 0.25; font.letterSpacing: 0.2 * s
            }
            Text {
                id: promptText; anchors.centerIn: parent; text: "Click to Start"; font.family: mainFont.name; font.pixelSize: 15 * s; font.letterSpacing: 0.8 * s; color: "white"
                SequentialAnimation on opacity { loops: Animation.Infinite; NumberAnimation { from: 0.4; to: 0.9; duration: 2500; easing.type: Easing.InOutSine } NumberAnimation { from: 0.9; to: 0.4; duration: 2500; easing.type: Easing.InOutSine } }
                MouseArea { anchors.fill: parent; anchors.margins: -10 * s; onClicked: doLogin() }
            }
            Row {
                anchors.right: parent.right; anchors.rightMargin: 24 * s; anchors.bottom: parent.bottom; anchors.bottomMargin: 14 * s; spacing: 12 * s
                Text { id: srDate; font.family: mainFont.name; font.pixelSize: 11 * s; color: "white"; opacity: 0.4; font.letterSpacing: 1.5 * s; anchors.verticalCenter: parent.verticalCenter }
                Rectangle { width: 2 * s; height: 10 * s; color: root.srGoldLight; opacity: 0.5; anchors.verticalCenter: parent.verticalCenter }
                Text { id: srTime; font.family: mainFont.name; font.pixelSize: 15 * s; font.bold: true; color: root.srGoldLight; opacity: 0.9; font.letterSpacing: 1.5 * s; anchors.verticalCenter: parent.verticalCenter }
                Timer { interval: 1000; running: true; repeat: true; onTriggered: { var d = new Date(); srTime.text = Qt.formatTime(d, "HH:mm"); srDate.text = Qt.formatDate(d, "yyyy / MM / dd") } Component.onCompleted: triggered() }
            }
        }
    }

    Item {
        id: popupOverlay; anchors.fill: parent; visible: root.sessionPopupOpen
        Rectangle { anchors.fill: parent; color: "#aa000000"; opacity: root.sessionPopupOpen ? 1 : 0; Behavior on opacity { NumberAnimation { duration: 300 } } MouseArea { anchors.fill: parent; onClicked: root.sessionPopupOpen = false } }
        Item {
            anchors.centerIn: parent; width: 440 * s; height: 500 * s; scale: root.sessionPopupOpen ? 1.0 : 0.9; opacity: root.sessionPopupOpen ? 1 : 0
            Behavior on scale { NumberAnimation { duration: 350; easing.type: Easing.OutBack } } Behavior on opacity { NumberAnimation { duration: 250 } }
            Text { text: "SELECT DATA CENTER"; anchors.top: parent.top; anchors.horizontalCenter: parent.horizontalCenter; font.family: mainFont.name; font.pixelSize: 14 * s; font.bold: true; font.letterSpacing: 6 * s; color: root.srGold; opacity: 0.8 }
            Rectangle { width: parent.width; height: 1.5 * s; color: root.srGold; opacity: 0.3; anchors.top: parent.top; anchors.topMargin: 36 * s }
            ListView {
                anchors.top: parent.top; anchors.topMargin: 60 * s; width: parent.width; height: parent.height - 80 * s
                model: typeof sessionModel !== "undefined" ? sessionModel : null; clip: true; spacing: 10 * s
                delegate: Item {
                    width: ListView.view.width; height: 60 * s
                    Rectangle {
                        id: itemRect; anchors.fill: parent; radius: 4 * s; color: (index === root.sessionIndex) ? "#15ffffff" : (sesItemMouse.containsMouse ? "#08ffffff" : "transparent")
                        border.color: (index === root.sessionIndex) ? root.srGold : (sesItemMouse.containsMouse ? "#44ffffff" : "#22ffffff"); border.width: 1.5 * s
                        Item {
                            anchors.fill: parent; anchors.leftMargin: 20 * s
                            Rectangle {
                                id: selIndicator; width: 32 * s; height: 32 * s; radius: 16 * s; anchors.verticalCenter: parent.verticalCenter; color: (index === root.sessionIndex) ? root.srGold : "transparent"; border.color: root.srGold; border.width: 1.5 * s
                                Text { text: "✓"; visible: index === root.sessionIndex; anchors.centerIn: parent; color: "black"; font.bold: true }
                            }
                            Text { text: model.name.toUpperCase(); anchors.left: selIndicator.right; anchors.leftMargin: 16 * s; anchors.verticalCenter: parent.verticalCenter; font.family: mainFont.name; font.pixelSize: 18 * s; color: (index === root.sessionIndex || sesItemMouse.containsMouse) ? "white" : root.srGhost; font.letterSpacing: 1.5 * s }
                        }
                        MouseArea { id: sesItemMouse; anchors.fill: parent; hoverEnabled: true; onClicked: { root.sessionIndex = index; root.sessionPopupOpen = false } }
                    }
                }
            }
            Rectangle { width: 60 * s; height: 2 * s; color: root.srGold; anchors.bottom: parent.bottom; anchors.horizontalCenter: parent.horizontalCenter }
        }
    }

    // Action
    function doLogin() {
        if (passIn.text === "") { passIn.forceActiveFocus() }
        else {
            var uname = ""
            if (typeof userModel !== "undefined") { uname = userModel.data(userModel.index(root.userIndex, 0), Qt.UserRole + 1) }
            if (typeof sddm !== "undefined") sddm.login(uname, passIn.text, root.sessionIndex)
        }
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() { errText.text = "ACCESS DENIED"; passIn.text = ""; passIn.forceActiveFocus(); passFailAnim.start(); passLine.color = "#ff3355" }
    }

    SequentialAnimation {
        id: passFailAnim
        PauseAnimation { duration: 1000 }
        ColorAnimation { target: passLine; property: "color"; to: "#44ffffff"; duration: 400 }
    }

    Timer { interval: 300; running: true; onTriggered: passIn.forceActiveFocus() }
    Component.onCompleted: keyboard.numLock = true
}
