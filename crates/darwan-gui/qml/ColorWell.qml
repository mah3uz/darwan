import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Shapes

// A colour square: the colour, a hatch for the theme's own, or a rainbow for one generated from the background.
AbstractButton {
    id: well

    property string colour: ""
    property bool generated: false
    readonly property bool valid: /^#[0-9a-fA-F]{3,8}$/.test(colour)

    implicitWidth: 44
    implicitHeight: 24
    hoverEnabled: true
    focusPolicy: Qt.TabFocus

    background: Rectangle {
        radius: Style.small
        color: well.hovered ? Style.controlHover : Style.control
        border.width: well.visualFocus ? 2 : 0
        border.color: Style.accent

        Rectangle {
            id: swatch
            anchors.fill: parent
            anchors.margins: 3
            radius: 3
            color: well.valid && !well.generated ? well.colour : "transparent"
            border.color: Qt.rgba(1, 1, 1, 0.14)
            gradient: well.generated ? rainbow : null
        }
        Shape {
            anchors.fill: swatch
            visible: !well.generated && !well.valid
            ShapePath {
                strokeColor: Qt.rgba(1, 1, 1, 0.25)
                strokeWidth: 1
                fillColor: "transparent"
                startX: 2; startY: swatch.height - 2
                PathLine { x: swatch.width - 2; y: 2 }
            }
        }
    }
    Gradient {
        id: rainbow
        orientation: Gradient.Horizontal
        GradientStop { position: 0; color: "#ff5f6d" }
        GradientStop { position: 0.3; color: "#ffc371" }
        GradientStop { position: 0.55; color: "#a6e3a1" }
        GradientStop { position: 0.8; color: "#64d2ff" }
        GradientStop { position: 1; color: "#b4befe" }
    }
}
