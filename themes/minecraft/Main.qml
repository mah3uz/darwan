import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
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
    readonly property real s: Screen.height / 768
    width: Screen.width; height: Screen.height
    color: "#1e1e1e"

    // Quickshell
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined

    // State
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: 0
    property bool sessionPopupOpen: false
    property real uiOpacity: 0
    
    // Colors
    readonly property color mcBtnFace:      "#8b8b8b"
    readonly property color mcBtnHighlight: "#bcbcbc"
    readonly property color mcBtnShadow:    "#373737"
    readonly property color mcBtnHover:     "#9090c0" 
    readonly property color mcBtnPress:     "#585858"
    
    readonly property color mcTextWhite:    "#ffffff"
    readonly property color mcTextShadow:   "#3f3f3f"
    readonly property color mcTextYellow:   "#ffff55"
    readonly property color mcTextGray:     "#aaaaaa"
    readonly property color mcTextRed:      "#ff5555"
    readonly property color mcTextGreen:    "#55ff55"
    
    readonly property color mcFldBg:        "#000000"
    readonly property color mcFldBorder:    "#a0a0a0"

    // Fonts
    FolderListModel { id: fontFolder; folder: Qt.resolvedUrl("font"); nameFilters: ["*.ttf", "*.otf"] }
    FontLoader { id: mcFont; source: fontFolder.count > 0 ? "font/" + fontFolder.get(0, "fileName") : "" }
    TextConstants { id: textConstants }

    // Background
    Item {
        anchors.fill: parent; z: 0
        Image {
            anchors.fill: parent
            source: "background.png"; fillMode: Image.PreserveAspectCrop
            horizontalAlignment: Image.AlignHCenter; verticalAlignment: Image.AlignVCenter
            scale: 2.0; transformOrigin: Item.Center
        }
    }

    // Effect
    RadialGradient {
        anchors.fill: parent; z: 1
        gradient: Gradient {
            GradientStop { position: 0.0; color: "transparent" }
            GradientStop { position: 1.2; color: "#aa000000" }
        }
    }

    // Helpers
    ListView {
        id: sessionHelper
        model: typeof sessionModel !== "undefined" ? sessionModel : null; currentIndex: root.sessionIndex
        opacity: 0; width: 100; height: 100; z: -100; visible: true
        delegate: Item { property string sName: model.name || ""; property string sComment: model.comment || "" }
    }
    ListView {
        id: userHelper
        model: typeof userModel !== "undefined" ? userModel : null; currentIndex: root.userIndex
        opacity: 0; width: 100; height: 100; z: -100; visible: true
        delegate: Item { property string uName: model.realName || model.name || ""; property string uLogin: model.name || "" }
    }

    // Interface
    Column {
        id: mainStack
        anchors.centerIn: parent
        width: 420 * s; spacing: 20 * s; opacity: root.uiOpacity

        // Logo
        Item {
            width: parent.width; height: 200 * s
            Image {
                id: mainLogo; source: "title.png"
                width: 850 * s; height: 180 * s
                fillMode: Image.PreserveAspectFit
                anchors.horizontalCenter: parent.horizontalCenter
                layer.enabled: true
                layer.effect: DropShadow {
                    transparentBorder: true
                    horizontalOffset: 4 * s; verticalOffset: 4 * s
                    radius: 8 * s; samples: 16
                    color: "#cc000000"
                }
            }

            Text {
                id: splashText
                anchors.horizontalCenter: mainLogo.horizontalCenter; anchors.horizontalCenterOffset: 255 * s
                anchors.top: mainLogo.top; anchors.topMargin: 35 * s
                
                text: "GNU/Linux!"; font.family: mcFont.name; font.pixelSize: 24 * s
                color: root.mcTextYellow; rotation: -20; style: Text.Outline; styleColor: "black"
                SequentialAnimation on scale {
                    loops: Animation.Infinite
                    NumberAnimation { from: 1.0; to: 1.18; duration: 600; easing.type: Easing.InOutQuad }
                    NumberAnimation { from: 1.18; to: 1.0; duration: 600; easing.type: Easing.InOutQuad }
                }
                Component.onCompleted: {
                    var splashes = ["I use Arch btw", "|||RTFM!|||", "sudo rm -rf /", "Kernel Panic!", "Btw I use NixOS!", "Pacman -Syu", "chmod 777", "Segmentation Fault"];
                    text = splashes[Math.floor(Math.random() * splashes.length)];
                }
            }
        }

        Item { width: 1; height: 12 * s } 

        // Target
        Column {
            width: parent.width; spacing: 10 * s
            McText { label: "Logged in as:"; pixelSize: 12 * s; textColor: root.mcTextGray }
            McTextField {
                id: userBox; width: parent.width; height: 44 * s; inputRef: userMouse
                McText {
                    anchors.left: parent.left; anchors.leftMargin: 12 * s
                    anchors.verticalCenter: parent.verticalCenter
                    label: capitalizeFirst((userHelper.currentItem && userHelper.currentItem.uLogin !== "") ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "User"))
                    pixelSize: 18 * s; textColor: root.mcTextWhite
                }
                MouseArea {
                    id: userMouse; anchors.fill: parent; cursorShape: Qt.PointingHandCursor; hoverEnabled: true
                    onClicked: { if (typeof userModel !== "undefined" && userModel.rowCount() > 0) root.userIndex = (root.userIndex + 1) % userModel.rowCount() }
                    property bool isHighlighted: containsMouse || pressed
                }
            }
        }

        // Credentials
        Column {
            width: parent.width; spacing: 10 * s
            McText { label: "Enter Password:"; pixelSize: 12 * s; textColor: root.mcTextGray }
            McTextField {
                id: passField; width: parent.width; height: 42 * s; inputRef: passInput
                TextInput {
                    id: passInput; anchors.fill: parent; anchors.leftMargin: 12 * s; anchors.rightMargin: 12 * s
                    echoMode: TextInput.Password; passwordCharacter: "*"; color: "white"
                    font.family: mcFont.name; font.pixelSize: 18 * s; font.letterSpacing: 4 * s
                    verticalAlignment: TextInput.AlignVCenter; clip: true; focus: true
                    cursorVisible: false; cursorDelegate: Item { width: 0; height: 0 }
                    selectionColor: root.mcBtnHover
                    property bool wasClicked: false
                    onTextEdited: errText.text = ""
                    KeyNavigation.tab: loginBtn; KeyNavigation.backtab: rebootBtn
                    Keys.onReturnPressed: doLogin()
                    
                    Text {
                        anchors.fill: parent; verticalAlignment: Text.AlignVCenter; anchors.leftMargin: 2 * s
                        text: "Enter password..."; color: "#555555"; font.family: mcFont.name; font.pixelSize: 14 * s
                        opacity: passInput.text.length === 0 ? 1.0 : 0
                        Behavior on opacity { NumberAnimation { duration: 400; easing.type: Easing.InOutSine } }
                    }
                    Rectangle {
                        id: customCursor
                        width: 2 * s; height: 22 * s
                        color: root.mcTextWhite
                        anchors.verticalCenter: parent.verticalCenter
                        x: passInput.cursorRectangle.x
                        visible: passInput.focus && (passInput.text.length > 0 || passInput.wasClicked)
                        SequentialAnimation {
                            loops: Animation.Infinite; running: customCursor.visible
                            NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0.05; duration: 450 }
                            NumberAnimation { target: customCursor; property: "opacity"; from: 0.05; to: 1; duration: 450 }
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            passInput.forceActiveFocus()
                            passInput.wasClicked = true
                        }
                    }
                }
            }
        }

        // Error
        Text {
            id: errText; width: parent.width; horizontalAlignment: Text.AlignHCenter
            height: 15 * s; verticalAlignment: Text.AlignBottom
            text: ""; color: root.mcTextRed; font.family: mcFont.name; font.pixelSize: 14 * s
        }

        Item { width: 1; height: 10 * s }

        // Actions
        McButton { 
            id: loginBtn; width: parent.width; height: 48 * s; label: "Login"
            onClicked: doLogin(); KeyNavigation.tab: sessionBtn; KeyNavigation.backtab: passInput
        }

        McButton {
            id: sessionBtn; width: parent.width; height: 48 * s
            visible: !root.isQuickshell
            label: (typeof sessionModel !== "undefined" && sessionHelper.currentItem && sessionHelper.currentItem.sName ? sessionHelper.currentItem.sName : "SESSION")
            onClicked: root.sessionPopupOpen = true; KeyNavigation.tab: shutdownBtn; KeyNavigation.backtab: loginBtn
        }

        Row {
            width: parent.width; spacing: 12 * s
            McButton { id: shutdownBtn; width: (parent.width - 12 * s) / 2; height: 48 * s; label: "Shutdown"; onClicked: { if (typeof sddm !== "undefined") sddm.powerOff() } KeyNavigation.tab: rebootBtn }
            McButton { id: rebootBtn; width: (parent.width - 12 * s) / 2; height: 48 * s; label: "Reboot"; onClicked: { if (typeof sddm !== "undefined") sddm.reboot() } KeyNavigation.tab: passInput }
        }
    }

    // Modal
    Item {
        id: sessionOverlay
        visible: root.sessionPopupOpen && !root.isQuickshell
        anchors.fill: parent; z: 5000
        
        Rectangle { 
            anchors.fill: parent; color: "black"; opacity: 0.9 
            Image { anchors.fill: parent; source: "background.png"; fillMode: Image.Tile; opacity: 0.12; visible: sessionOverlay.visible }
        }

        Column {
            id: sessionContent
            anchors.centerIn: parent; width: 440 * s; spacing: 16 * s
            opacity: sessionOverlay.visible ? 1 : 0
            scale: sessionOverlay.visible ? 1 : 0.9
            Behavior on opacity { NumberAnimation { duration: 200 } }
            Behavior on scale { NumberAnimation { duration: 250; easing.type: Easing.OutBack } }
            
            McText {
                label: "SELECT SESSION"; pixelSize: 26 * s; textColor: root.mcTextYellow
                anchors.horizontalCenter: parent.horizontalCenter; horizontalAlignment: Text.AlignHCenter
            }
            
            Item { width: 1; height: 10 * s }

            Column {
                width: parent.width; spacing: 8 * s
                Repeater {
                    model: typeof sessionModel !== "undefined" ? sessionModel : null
                    McButton {
                        width: 440 * s; height: 50 * s
                        label: model.name || "Default"
                        isActive: index === root.sessionIndex
                        onClicked: { root.sessionIndex = index; root.sessionPopupOpen = false }
                    }
                }
            }
            
            Item { width: 1; height: 12 * s }

            McButton {
                width: 200 * s; height: 44 * s; label: "Cancel"
                anchors.horizontalCenter: parent.horizontalCenter
                onClicked: root.sessionPopupOpen = false
            }
        }

        Keys.onEscapePressed: root.sessionPopupOpen = false
        focus: visible
    }

    // Corners
    Item {
        anchors.fill: parent; opacity: root.uiOpacity
        
        McText {
            anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.margins: 6 * s
            id: versionText; label: "Current Time: " + Qt.formatTime(new Date(), "HH:mm")
            pixelSize: 14 * s; textColor: root.mcTextWhite
            Timer { interval: 1000; running: true; repeat: true; onTriggered: versionText.label = "Current Time: " + Qt.formatTime(new Date(), "HH:mm") }
        }

        McText {
            anchors.bottom: parent.bottom; anchors.right: parent.right; anchors.margins: 6 * s
            label: Qt.formatDate(new Date(), "dddd, MMMM d")
            pixelSize: 14 * s; textColor: root.mcTextWhite
        }
    }

    // Boot
    NumberAnimation { id: fadeIn; target: root; property: "uiOpacity"; to: 1; duration: 1000; easing.type: Easing.OutCubic }
    Component.onCompleted: { fadeIn.start(); keyboard.numLock = true }

    function doLogin() { 
        var lName = (userHelper.currentItem && userHelper.currentItem.uLogin !== "") ? userHelper.currentItem.uLogin : (typeof userModel !== "undefined" ? userModel.lastUser : "")
        if (typeof sddm !== "undefined") sddm.login(lName, passInput.text, root.sessionIndex) 
    }
    
    function capitalizeFirst(str) {
        if (!str) return "";
        return str.charAt(0).toUpperCase() + str.slice(1);
    }

    Connections { target: typeof sddm !== "undefined" ? sddm : null; function onLoginFailed() { errText.text = "ACCESS DENIED"; passInput.text = ""; passInput.forceActiveFocus() } }

    // Widgets
    component McText: Item {
        property string label: ""; property int pixelSize: 16 * s; property color textColor: root.mcTextWhite
        property int horizontalAlignment: Text.AlignLeft
        property int shadowOffset: 2 * s
        implicitWidth: fore.implicitWidth + 8 * s; implicitHeight: fore.implicitHeight + 2 * s
        Text { x: shadowOffset; y: shadowOffset; width: parent.width; text: label; color: root.mcTextShadow; font.family: mcFont.name; font.pixelSize: pixelSize; horizontalAlignment: parent.horizontalAlignment }
        Text { id: fore; width: parent.width; text: label; color: textColor; font.family: mcFont.name; font.pixelSize: pixelSize; horizontalAlignment: parent.horizontalAlignment }
    }
    
    component McTextField: Item {
        property var inputRef
        Rectangle {
            anchors.fill: parent; color: "black"
            border.color: "#808080"; border.width: 2 * s
            Rectangle {
                anchors.fill: parent; anchors.margins: 2 * s; color: "#0a0a0a"
                Rectangle {
                    visible: inputRef && (inputRef.activeFocus || (inputRef.isHighlighted === true))
                    anchors.fill: parent; color: "transparent"; border.color: "#ffffff"; border.width: 1 * s
                }
            }
        }
    }
    
    component McButton: Item {
        id: mcBtn; property string label: ""; signal clicked()
        property bool isActive: false
        implicitWidth: 200; implicitHeight: 48
        
        Rectangle {
            anchors.fill: parent; color: "black"
            Rectangle {
                anchors.fill: parent; anchors.margins: 2 * s
                color: mcMouse.pressed ? root.mcBtnPress : (mcMouse.containsMouse ? "#686868" : (mcBtn.isActive ? "#454545" : root.mcBtnFace))
                
                Rectangle { anchors.top: parent.top; anchors.left: parent.left; anchors.right: parent.right; height: 3 * s; color: "white"; opacity: mcMouse.pressed ? 0.1 : (mcMouse.containsMouse ? 0.4 : 0.2) }
                Rectangle { anchors.top: parent.top; anchors.left: parent.left; anchors.bottom: parent.bottom; width: 3 * s; color: "white"; opacity: mcMouse.pressed ? 0.1 : (mcMouse.containsMouse ? 0.4 : 0.2) }
                Rectangle { anchors.bottom: parent.bottom; anchors.left: parent.left; anchors.right: parent.right; height: 3 * s; color: "black"; opacity: 0.4 }
                Rectangle { anchors.top: parent.top; anchors.right: parent.right; anchors.bottom: parent.bottom; width: 3 * s; color: "black"; opacity: 0.4 }
                
                McText { 
                    anchors.centerIn: parent
                    label: mcBtn.label; textColor: mcMouse.containsMouse ? root.mcTextYellow : root.mcTextWhite; pixelSize: 18 * s 
                    shadowOffset: mcMouse.pressed ? 3 * s : 2 * s
                }
            }
        }
        MouseArea { id: mcMouse; anchors.fill: parent; hoverEnabled: true; onClicked: mcBtn.clicked() }
    }
}
