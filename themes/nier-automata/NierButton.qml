import QtQuick
import QtQuick.Window

// NieR: Automata style button
// Usage: NierButton { text: "Label"; fontFamily: someFont.name; onClicked: ... }
Item {
    readonly property real s: Screen.height / 768
    id: btn
    property string text:       ""
    property string fontFamily: ""
    property int    fontPixelSize: 12
    signal clicked()

    width:  Math.max(160 * s, lbl.implicitWidth + 36 * s)
    height: 28 * s

    scale: ma.pressed ? 0.97 : 1.0
    Behavior on scale { NumberAnimation { duration: 80; easing.type: Easing.OutBack } }

    Rectangle {
        anchors.fill: parent
        color:        ma.pressed ? "#4a4840" : ma.containsMouse ? "#3a3830" : "#2c2a24"
        border.color: ma.containsMouse ? "#d0cca8" : "#706c58"
        border.width: 1
        Behavior on color { ColorAnimation { duration: 80 } }

        // Left accent bar
        Rectangle {
            width: 3 * s; height: parent.height
            color: ma.containsMouse ? "#d0cca8" : "#706c58"
            Behavior on color { ColorAnimation { duration: 80 } }
        }

        Text {
            id: lbl
            text: "◆ " + btn.text
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left; anchors.leftMargin: 11 * s
            font.family:      btn.fontFamily
            font.pixelSize:   btn.fontPixelSize * s
            font.letterSpacing: 0.8
            color: ma.containsMouse ? "#d0cca8" : "#b0ac94"
            Behavior on color { ColorAnimation { duration: 80 } }
        }
    }

    MouseArea {
        id: ma; anchors.fill: parent; hoverEnabled: true
        onClicked: btn.clicked()
    }
}
