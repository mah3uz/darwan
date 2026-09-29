import QtQuick
import QtQuick.Controls.Basic

// A switch whose state comes from outside; clicking asks for the other state instead of flipping itself.
AbstractButton {
    id: toggle

    property bool on: false
    signal flipped(bool on)

    implicitWidth: 38
    implicitHeight: 22
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    onClicked: flipped(!on)

    background: Rectangle {
        radius: height / 2
        color: toggle.on ? Style.accent : Style.switchOff
        border.width: toggle.visualFocus ? 2 : 0
        border.color: Qt.lighter(Style.accent, 1.3)
        Behavior on color { ColorAnimation { duration: Style.fast } }
        Rectangle {
            y: 2
            x: toggle.on ? parent.width - width - 2 : 2
            width: 18
            height: 18
            radius: 9
            color: "white"
            Behavior on x { NumberAnimation { duration: Style.fast; easing.type: Style.ease } }
        }
    }
}
