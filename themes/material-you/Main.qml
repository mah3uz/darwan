import QtQuick
import QtQuick.Window
import Qt5Compat.GraphicalEffects
import SddmComponents 2.0
import "darwan"

Rectangle {
    readonly property bool clock12: config.clockFormat === "12h"
    function clockHour(d) { return clock12 ? String(d.getHours() % 12 || 12).padStart(2, "0") : Qt.formatTime(d, "HH") }
    function withAmPm(d, text) { return clock12 && config.clockShowAmPm === "true" ? text + " · " + (d.getHours() < 12 ? "AM" : "PM") : text }
    id: root
    width: Screen.width
    height: Screen.height
    color: root.isDark ? "#131218" : "white"

    // One palette per variant, exactly as designed; generated Material colours replace it role by role.
    readonly property bool isDark: config.colorScheme === "dark"
    Custom { id: kit }
    QtObject {
        id: pal
        function pick(lightRole, darkRole, light, dark) {
            const generated = config["material_" + (root.isDark ? darkRole : lightRole)]
            return generated ? generated : (root.isDark ? dark : light)
        }
        readonly property color ink: kit.color("text", pick("on_secondary_container", "on_surface", "#0f3c2c", "#e6e1e5"))
        readonly property color inkSoft: pick("on_surface_variant", "on_surface_variant", "#1e4f3e", "#cbc2db")
        readonly property color chip: pick("primary_container", "surface_variant", "#bee8c7", "#3a3247")
        readonly property color outline: pick("outline", "outline", "#8ca090", "#958da5")
        readonly property color tilePressed: pick("on_primary_container", "surface_container_low", "#0a281d", "#1c1924")
        readonly property color tileHover: pick("on_secondary_container", "surface_variant", "#0f3c2c", "#322a3e")
        readonly property color tile: pick("surface_container", "surface_container", "#e9f3eb", "#25232a")
        readonly property color tileIconHover: pick("primary_container", "on_surface", "#bee8c7", "#e6e1e5")
        readonly property color tileIcon: pick("on_secondary_container", "on_surface_variant", "#0f3c2c", "#cbc2db")
        readonly property color tileSubHover: pick("surface_container", "on_surface_variant", "#e9f3eb", "#cbc2db")
        readonly property color tileSub: pick("on_surface_variant", "outline", "#1e4f3e", "#958da5")
        readonly property color card: pick("surface_container", "surface_container_low", "#e9f3eb", "#1c1b20")
        readonly property color field: pick("surface_container_high", "surface_container_high", "#d0eadb", "#2b2930")
        readonly property color error: pick("error", "error", "#ea1821", "#ffb4ab")
        readonly property color focus: kit.color("accent", pick("on_secondary_container", "primary", "#0f3c2c", "#d0bcff"))
        readonly property color fieldText: kit.color("text", pick("on_surface", "on_surface", "#1d3c34", "#e6e1e5"))
        readonly property color selection: pick("secondary_container", "secondary_container", "#c2ebd4", "#4f4461")
        readonly property color cursor: kit.color("accent", pick("on_surface", "primary", "#1d3c34", "#d0bcff"))
        readonly property color userPressed: pick("surface_container_highest", "surface_container", "#cbe8cc", "#201e25")
        readonly property color userHover: pick("surface_container_high", "surface_container_highest", "#d2ebd4", "#36343b")
        readonly property color user: pick("surface_container_low", "surface_container_high", "#eef6f0", "#2b2930")
        readonly property color userText: kit.color("text", pick("on_surface", "on_surface_variant", "#1d3c34", "#cbc2db"))
        readonly property color loginPressed: pick("on_primary_container", "surface_container_high", "#0a281d", "#2b2238")
        readonly property color loginHover: pick("on_surface_variant", "secondary_container", "#1e4f3e", "#4f4461")
        readonly property color login: pick("on_secondary_container", "surface_variant", "#0f3c2c", "#3a3247")
    }

    // Background
    Background { id: userBg; anchors.fill: parent }
    Image {
        anchors.fill: parent
        source: root.isDark ? "bg-dark.png" : "bg.png"
        fillMode: Image.PreserveAspectCrop
        visible: !userBg.active
    }

    readonly property real s: Screen.height / 768
    property bool isQuickshell: typeof sddm === "undefined" || sddm.hostName === undefined
    property int sessionIndex: (typeof sessionModel !== "undefined" && sessionModel.lastIndex >= 0) ? sessionModel.lastIndex : 0
    property int userIndex: (typeof userModel !== "undefined" && userModel.lastIndex >= 0) ? userModel.lastIndex : 0
    
    // UI States
    property real ui1: 0
    property real ui2: 0
    property string errorMessage: ""

    // Fonts
    FontLoader {
        id: customFont
        source: "font/GoogleSans-VariableFont_GRAD,opsz,wght.ttf"
    }
    
    readonly property string sansFont: kit.font("text", customFont.name !== "" ? customFont.name : "Roboto, Inter, sans-serif")
    readonly property string clockFont: kit.font("clock", customFont.name !== "" ? customFont.name : "Roboto, Inter, sans-serif")

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

    Timer {
        id: focusTimer
        interval: 300
        running: true
        onTriggered: pwd.forceActiveFocus()
    }

    Connections {
        target: typeof sddm !== "undefined" ? sddm : null
        function onLoginFailed() {
            root.errorMessage = "ACCESS DENIED";
            pwd.text = "";
            shakeAnim.start();
            errTimer.start();
        }
    }

    Timer {
        id: errTimer
        interval: 3000
        onTriggered: root.errorMessage = ""
    }

    Component.onCompleted: {
        fadeAnim.start();
        if (typeof keyboard !== "undefined") keyboard.numLock = true;
    }

    SequentialAnimation {
        id: fadeAnim
        PauseAnimation { duration: kit.dur(500) }
        ParallelAnimation {
            NumberAnimation { target: root; property: "ui1"; from: 0; to: 1; duration: kit.dur(900); easing.type: kit.ease(Easing.OutCubic) }
            NumberAnimation { target: root; property: "ui2"; from: 0; to: 1; duration: kit.dur(900); easing.type: kit.ease(Easing.OutCubic) }
        }
    }

    SequentialAnimation {
        id: shakeAnim
        NumberAnimation { target: shakeTranslate; property: "x"; to: 15*s; duration: kit.dur(50) }
        NumberAnimation { target: shakeTranslate; property: "x"; to: -15*s; duration: kit.dur(50) }
        NumberAnimation { target: shakeTranslate; property: "x"; to: 15*s; duration: kit.dur(50) }
        NumberAnimation { target: shakeTranslate; property: "x"; to: -15*s; duration: kit.dur(50) }
        NumberAnimation { target: shakeTranslate; property: "x"; to: 0; duration: kit.dur(50) }
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.ArrowCursor
        z: -1
        onClicked: pwd.forceActiveFocus()
    }

    // Layout Row
    Row {
        id: mainLayout
        anchors.centerIn: parent
        spacing: 96 * s
        opacity: root.ui1
        scale: 0.96 + (0.04 * root.ui1)
        transform: Translate { y: (1 - root.ui1) * 30 * s }

        // Left Section
        Column {
            spacing: 24 * s
            anchors.verticalCenter: parent.verticalCenter
            
            Timer {
                interval: 1000
                running: true
                repeat: true
                onTriggered: {
                    let d = new Date();
                    hText.text = clockHour(d);
                    mText.text = Qt.formatTime(d, "mm");
                    dateChipText.text = withAmPm(d, Qt.formatDate(d, config.dateFormat || "dddd, MMM d").toUpperCase());
                }
            }

            // Clock
            Column {
                spacing: -24 * s
                
                Text {
                    id: hText
                    text: clockHour(new Date())
                    font.family: root.clockFont
                    font.pixelSize: 140 * s
                    font.weight: Font.Bold
                    color: pal.ink
                }
                
                Text {
                    id: mText
                    text: Qt.formatTime(new Date(), "mm")
                    font.family: root.clockFont
                    font.pixelSize: 140 * s
                    font.weight: Font.Bold
                    color: pal.inkSoft
                }
            }

            // Date Pill
            Rectangle {
                width: dateChipText.implicitWidth + 32 * s
                height: 44 * s
                radius: 22 * s
                color: pal.chip
                
                Text {
                    id: dateChipText
                    anchors.centerIn: parent
                    text: withAmPm(new Date(), Qt.formatDate(new Date(), config.dateFormat || "dddd, MMM d").toUpperCase())
                    font.family: root.sansFont
                    font.pixelSize: 11 * s
                    font.bold: true
                    font.letterSpacing: 1 * s
                    color: pal.ink
                }
            }
        }

        // Right Section
        Column {
            spacing: 24 * s
            anchors.verticalCenter: parent.verticalCenter

            // Settings Title
            Text {
                text: "QUICK SETTINGS"
                font.family: root.sansFont
                font.pixelSize: 11 * s
                font.bold: true
                font.letterSpacing: 1.5 * s
                color: pal.outline
            }

            // Settings Grid
            Grid {
                columns: 2
                spacing: 16 * s
                
                // Power
                Rectangle {
                    id: powerTile
                    width: 180 * s; height: 76 * s; radius: 38 * s
                    color: powerMouse.pressed ? pal.tilePressed : (powerMouse.containsMouse ? pal.tileHover : pal.tile)
                    scale: powerMouse.pressed ? 0.95 : (powerMouse.containsMouse ? 1.03 : 1.0)
                    Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                    Behavior on scale { NumberAnimation { duration: kit.dur(200); easing.type: kit.ease(Easing.OutBack) } }
                    
                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 16 * s
                        anchors.rightMargin: 16 * s
                        spacing: 12 * s
                        
                        Rectangle {
                            width: 48 * s; height: 48 * s; radius: 24 * s
                            color: pal.chip
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Image {
                                id: powerIcon
                                source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'><path d='M18.36 6.64a9 9 0 1 1-12.73 0'></path><line x1='12' y1='2' x2='12' y2='12'></line></svg>"
                                anchors.centerIn: parent
                                width: 20 * s
                                height: 20 * s
                                sourceSize.width: 40 * s
                                sourceSize.height: 40 * s
                                visible: false
                            }
                            ColorOverlay {
                                anchors.fill: powerIcon
                                source: powerIcon
                                color: pal.ink
                            }
                        }
                        
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 2 * s
                            
                            Text {
                                text: "POWER"
                                font.family: root.sansFont
                                font.pixelSize: 12 * s
                                font.bold: true
                                color: powerMouse.containsMouse ? pal.tileIconHover : pal.tileIcon
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                            Text {
                                text: "SHUT DOWN"
                                font.family: root.sansFont
                                font.pixelSize: 9 * s
                                color: powerMouse.containsMouse ? pal.tileSubHover : pal.tileSub
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                        }
                    }
                    
                    MouseArea {
                        id: powerMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (!root.isQuickshell) sddm.powerOff();
                    }
                }
                
                // Session
                Rectangle {
                    id: sessionTile
                    width: 180 * s; height: 76 * s; radius: 38 * s
                    color: sessionMouse.pressed ? pal.tilePressed : (sessionMouse.containsMouse ? pal.tileHover : pal.tile)
                    scale: sessionMouse.pressed ? 0.95 : (sessionMouse.containsMouse ? 1.03 : 1.0)
                    Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                    Behavior on scale { NumberAnimation { duration: kit.dur(200); easing.type: kit.ease(Easing.OutBack) } }
                    
                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 16 * s
                        anchors.rightMargin: 16 * s
                        spacing: 12 * s
                        
                        Rectangle {
                            width: 48 * s; height: 48 * s; radius: 24 * s
                            color: pal.chip
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Image {
                                id: sessionIcon
                                source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'><circle cx='12' cy='12' r='3'></circle><path d='M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z'></path></svg>"
                                anchors.centerIn: parent
                                width: 20 * s
                                height: 20 * s
                                sourceSize.width: 40 * s
                                sourceSize.height: 40 * s
                                visible: false
                            }
                            ColorOverlay {
                                anchors.fill: sessionIcon
                                source: sessionIcon
                                color: pal.ink
                            }
                        }
                        
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 2 * s
                            
                            Text {
                                text: "SESSION"
                                font.family: root.sansFont
                                font.pixelSize: 12 * s
                                font.bold: true
                                color: sessionMouse.containsMouse ? pal.tileIconHover : pal.tileIcon
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                            Text {
                                text: ((sessionHelper.currentItem && sessionHelper.currentItem.sName) ? sessionHelper.currentItem.sName : "PLASMA").toUpperCase()
                                font.family: root.sansFont
                                font.pixelSize: 9 * s
                                color: sessionMouse.containsMouse ? pal.tileSubHover : pal.tileSub
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                                elide: Text.ElideRight
                                width: 90 * s
                            }
                        }
                    }
                    
                    MouseArea {
                        id: sessionMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (!root.isQuickshell && typeof sessionModel !== "undefined" && sessionModel.rowCount() > 0) {
                                root.sessionIndex = (root.sessionIndex + 1) % sessionModel.rowCount();
                            }
                        }
                    }
                }

                // Reboot
                Rectangle {
                    id: rebootTile
                    width: 180 * s; height: 76 * s; radius: 38 * s
                    color: rebootMouse.pressed ? pal.tilePressed : (rebootMouse.containsMouse ? pal.tileHover : pal.tile)
                    scale: rebootMouse.pressed ? 0.95 : (rebootMouse.containsMouse ? 1.03 : 1.0)
                    Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                    Behavior on scale { NumberAnimation { duration: kit.dur(200); easing.type: kit.ease(Easing.OutBack) } }
                    
                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 16 * s
                        anchors.rightMargin: 16 * s
                        spacing: 12 * s
                        
                        Rectangle {
                            width: 48 * s; height: 48 * s; radius: 24 * s
                            color: pal.chip
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Image {
                                id: rebootIcon
                                source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'><polyline points='23 4 23 10 17 10'></polyline><path d='M20.49 15a9 9 0 1 1-2.12-9.36L23 10'></path></svg>"
                                anchors.centerIn: parent
                                width: 20 * s
                                height: 20 * s
                                sourceSize.width: 40 * s
                                sourceSize.height: 40 * s
                                visible: false
                            }
                            ColorOverlay {
                                anchors.fill: rebootIcon
                                source: rebootIcon
                                color: pal.ink
                            }
                        }
                        
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 2 * s
                            
                            Text {
                                text: "REBOOT"
                                font.family: root.sansFont
                                font.pixelSize: 12 * s
                                font.bold: true
                                color: rebootMouse.containsMouse ? pal.tileIconHover : pal.tileIcon
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                            Text {
                                text: "RESTART"
                                font.family: root.sansFont
                                font.pixelSize: 9 * s
                                color: rebootMouse.containsMouse ? pal.tileSubHover : pal.tileSub
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                        }
                    }
                    
                    MouseArea {
                        id: rebootMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (!root.isQuickshell) sddm.reboot();
                    }
                }

                // Sleep
                Rectangle {
                    id: suspendTile
                    width: 180 * s; height: 76 * s; radius: 38 * s
                    color: suspendMouse.pressed ? pal.tilePressed : (suspendMouse.containsMouse ? pal.tileHover : pal.tile)
                    scale: suspendMouse.pressed ? 0.95 : (suspendMouse.containsMouse ? 1.03 : 1.0)
                    Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                    Behavior on scale { NumberAnimation { duration: kit.dur(200); easing.type: kit.ease(Easing.OutBack) } }
                    
                    Row {
                        anchors.fill: parent
                        anchors.leftMargin: 16 * s
                        anchors.rightMargin: 16 * s
                        spacing: 12 * s
                        
                        Rectangle {
                            width: 48 * s; height: 48 * s; radius: 24 * s
                            color: pal.chip
                            anchors.verticalCenter: parent.verticalCenter
                            
                            Image {
                                id: suspendIcon
                                source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'><path d='M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z'></path></svg>"
                                anchors.centerIn: parent
                                width: 20 * s
                                height: 20 * s
                                sourceSize.width: 40 * s
                                sourceSize.height: 40 * s
                                visible: false
                            }
                            ColorOverlay {
                                anchors.fill: suspendIcon
                                source: suspendIcon
                                color: pal.ink
                            }
                        }
                        
                        Column {
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 2 * s
                            
                            Text {
                                text: "SLEEP"
                                font.family: root.sansFont
                                font.pixelSize: 12 * s
                                font.bold: true
                                color: suspendMouse.containsMouse ? pal.tileIconHover : pal.tileIcon
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                            Text {
                                text: "SUSPEND"
                                font.family: root.sansFont
                                font.pixelSize: 9 * s
                                color: suspendMouse.containsMouse ? pal.tileSubHover : pal.tileSub
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            }
                        }
                    }
                    
                    MouseArea {
                        id: suspendMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: if (!root.isQuickshell) sddm.suspend();
                    }
                }
            }

            // Login Card
            Rectangle {
                id: notificationCard
                width: 376 * s
                height: 180 * s
                radius: 32 * s
                color: pal.card
                transform: Translate { id: shakeTranslate }
                
                Column {
                    anchors.fill: parent
                    anchors.margins: 20 * s
                    spacing: 12 * s
                    
                    // Header
                    Row {
                        width: parent.width
                        spacing: 8 * s
                        
                        Item {
                            width: 12 * s
                            height: 12 * s
                            anchors.verticalCenter: parent.verticalCenter
                            Image {
                                id: lockIcon
                                source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'><rect x='3' y='11' width='18' height='11' rx='2' ry='2'></rect><path d='M7 11V7a5 5 0 0 1 10 0v4'></path></svg>"
                                anchors.fill: parent
                                sourceSize.width: 24 * s
                                sourceSize.height: 24 * s
                                visible: false
                            }
                            ColorOverlay {
                                anchors.fill: lockIcon
                                source: lockIcon
                                color: pal.outline
                            }
                        }
                        Text {
                            text: "SYSTEM UI"
                            font.family: root.sansFont
                            font.pixelSize: 10 * s
                            font.bold: true
                            font.letterSpacing: 1 * s
                            color: pal.outline
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Text {
                            text: "•  now"
                            font.family: root.sansFont
                            font.pixelSize: 10 * s
                            color: pal.outline
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }

                    // Password Box
                    Rectangle {
                        width: parent.width
                        height: 52 * s
                        radius: 26 * s
                        color: pal.field
                        border.color: root.errorMessage !== "" ? pal.error : (pwd.activeFocus ? pal.focus : "transparent")
                        border.width: pwd.activeFocus ? 2 * s : 0
                        Behavior on border.color { ColorAnimation { duration: kit.dur(150) } }
                        
                        TextInput {
                            id: pwd
                            anchors.fill: parent
                            anchors.leftMargin: 20 * s
                            anchors.rightMargin: 20 * s
                            font.family: root.sansFont
                            font.pixelSize: 18 * s
                            font.letterSpacing: 6 * s
                            color: pal.fieldText
                            echoMode: TextInput.Password
                            passwordCharacter: "•"
                            horizontalAlignment: TextInput.AlignHCenter
                            verticalAlignment: TextInput.AlignVCenter
                            clip: true
                            
                            cursorVisible: false
                            cursorDelegate: Item { width: 0; height: 0 }
                            selectionColor: pal.selection
                            
                            property bool wasClicked: false
                            onActiveFocusChanged: if (!activeFocus && text.length === 0) wasClicked = false
                            
                            Text {
                                anchors.centerIn: parent
                                text: root.errorMessage !== "" ? root.errorMessage : "PASSWORD REQUIRED"
                                font.family: root.sansFont
                                font.pixelSize: 11 * s
                                font.bold: true
                                font.letterSpacing: 1.5 * s
                                color: root.errorMessage !== "" ? pal.error : pal.outline
                                opacity: pwd.text === "" && (!pwd.activeFocus || (!pwd.wasClicked && pwd.text.length === 0)) ? 1 : 0
                                Behavior on opacity { NumberAnimation { duration: kit.dur(150) } }
                            }
                            
                            // Cursor
                            Rectangle {
                                id: customCursor
                                width: 2 * s
                                height: 18 * s
                                color: pal.cursor
                                anchors.verticalCenter: parent.verticalCenter
                                x: pwd.cursorRectangle.x
                                visible: pwd.activeFocus && (pwd.text.length > 0 || pwd.wasClicked) && root.errorMessage === ""
                                
                                SequentialAnimation {
                                    loops: Animation.Infinite
                                    running: customCursor.visible && !kit.reduceMotion
                                    NumberAnimation { target: customCursor; property: "opacity"; from: 1; to: 0; duration: kit.dur(400); easing.type: kit.ease(Easing.InOutQuad) }
                                    NumberAnimation { target: customCursor; property: "opacity"; from: 0; to: 1; duration: kit.dur(400); easing.type: kit.ease(Easing.InOutQuad) }
                                }
                            }
                            
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.IBeamCursor
                                onClicked: {
                                    pwd.wasClicked = true;
                                    pwd.forceActiveFocus();
                                }
                            }
                            
                            onAccepted: {
                                if (pwd.text !== "") {
                                    let currentUser = userHelper.currentItem ? userHelper.currentItem.uLogin : userModel.lastUser;
                                    sddm.login(currentUser, pwd.text, root.sessionIndex);
                                }
                            }
                        }
                    }

                    // Bottom Row
                    Row {
                        width: parent.width
                        spacing: 12 * s
                        
                        // User Switch
                        Rectangle {
                            width: userText.implicitWidth + 32 * s
                            height: 38 * s
                            radius: 19 * s
                            color: userMouse.pressed ? pal.userPressed : (userMouse.containsMouse ? pal.userHover : pal.user)
                            scale: userMouse.pressed ? 0.95 : (userMouse.containsMouse ? 1.02 : 1.0)
                            Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                            Behavior on scale { NumberAnimation { duration: kit.dur(150); easing.type: kit.ease(Easing.OutBack) } }
                            
                            Text {
                                id: userText
                                anchors.centerIn: parent
                                text: ((userHelper.currentItem && userHelper.currentItem.uName) ? userHelper.currentItem.uName : (userModel.lastUser || "USER")).toUpperCase()
                                font.family: root.sansFont
                                font.pixelSize: 10 * s
                                font.bold: true
                                font.letterSpacing: 1 * s
                                color: pal.userText
                            }
                            
                            MouseArea {
                                id: userMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (!root.isQuickshell && typeof userModel !== "undefined" && userModel.rowCount() > 0) {
                                        root.userIndex = (root.userIndex + 1) % userModel.rowCount();
                                    }
                                }
                            }
                        }

                        // Unlock Pill
                        Item {
                            width: parent.width - (userText.implicitWidth + 32 * s) - 12 * s
                            height: 38 * s
                            
                            Rectangle {
                                anchors.right: parent.right
                                width: parent.width
                                height: 38 * s
                                radius: 19 * s
                                color: loginMouse.pressed ? pal.loginPressed : (loginMouse.containsMouse ? pal.loginHover : pal.login)
                                scale: loginMouse.pressed ? 0.95 : (loginMouse.containsMouse ? 1.02 : 1.0)
                                Behavior on color { ColorAnimation { duration: kit.dur(150) } }
                                Behavior on scale { NumberAnimation { duration: kit.dur(150); easing.type: kit.ease(Easing.OutBack) } }
                                
                                Row {
                                    anchors.centerIn: parent
                                    spacing: 6 * s
                                    
                                    Text {
                                        text: "UNLOCK"
                                        font.family: root.sansFont
                                        font.pixelSize: 10 * s
                                        font.bold: true
                                        font.letterSpacing: 1.5 * s
                                        color: pal.tileIconHover
                                        anchors.verticalCenter: parent.verticalCenter
                                    }
                                    Text {
                                        text: "➔"
                                        font.family: root.sansFont
                                        font.pixelSize: 11 * s
                                        color: pal.tileIconHover
                                        anchors.verticalCenter: parent.verticalCenter
                                        transform: Translate {
                                            x: loginMouse.containsMouse ? 3 * s : 0
                                            Behavior on x { NumberAnimation { duration: kit.dur(150); easing.type: kit.ease(Easing.OutQuad) } }
                                        }
                                    }
                                }
                                
                                MouseArea {
                                    id: loginMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: pwd.accepted()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
