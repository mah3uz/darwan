import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects

// A pop-up panel that grows from where it was opened and stays inside the window.
Popup {
    id: pop

    padding: 6
    margins: 8
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.fast }
        NumberAnimation { property: "scale"; from: 0.95; to: 1; duration: Style.fast; easing.type: Style.ease }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: 110 }
        NumberAnimation { property: "scale"; to: 0.97; duration: 110 }
    }

    background: Item {
        RectangularShadow {
            anchors.fill: parent
            offset.y: 8
            blur: 28
            radius: Style.radius
            color: Qt.rgba(0, 0, 0, 0.5)
        }
        Rectangle {
            anchors.fill: parent
            radius: Style.radius
            color: Style.popover
            border.color: Style.panelBorder
            GlassEdge {}
        }
    }
}
