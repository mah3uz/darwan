import QtQuick

Rectangle {
    id: gbRoot
    property string label: "Button"
    signal clicked()
    property string fontFam: ""
    property real sc: 1.0

    width: 120 * sc; height: 36 * sc; radius: 2 * sc
    color: gbMouse.containsMouse ? "#c8ab6e" : "#2a3a5a"
    border.color: "#c8ab6e"; border.width: 1 * sc
    Behavior on color { ColorAnimation { duration: 150 } }
    scale: gbMouse.pressed ? 0.94 : 1.0
    Behavior on scale { NumberAnimation { duration: 100 } }

    Text {
        anchors.centerIn: parent
        text: gbRoot.label
        font.family: gbRoot.fontFam; font.pixelSize: 14 * gbRoot.sc
        color: gbMouse.containsMouse ? "#1a2340" : "#c8ab6e"
        font.letterSpacing: 1
        Behavior on color { ColorAnimation { duration: 150 } }
    }

    MouseArea {
        id: gbMouse; anchors.fill: parent; hoverEnabled: true
        onClicked: gbRoot.clicked()
    }
}
