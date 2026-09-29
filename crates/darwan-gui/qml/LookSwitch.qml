import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Darwan's own look or the system's Qt theme, on the window's left edge; saved the moment it's picked.
// A light knob sits under the look in use and slides to the other.
Rectangle {
    id: look

    required property Backend backend
    readonly property Item current: Style.look === "system" ? system : darwan

    width: 36
    height: column.height + 8
    radius: width / 2
    color: Style.panel
    border.color: Style.panelBorder
    GlassEdge {}

    Rectangle {
        x: 4
        y: look.current.y + 4
        width: 28
        height: 28
        radius: 14
        color: Qt.rgba(1, 1, 1, 0.92)
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
    }

    component Choice: AbstractButton {
        id: choice
        property string value: ""
        property string tipText: ""
        readonly property bool on: Style.look === value
        width: 28
        height: 28
        hoverEnabled: true
        focusPolicy: Qt.TabFocus
        opacity: on ? 1 : hovered ? 0.85 : 0.5
        Behavior on opacity { NumberAnimation { duration: Style.fast } }
        onClicked: if (!on) look.backend.setValueNow("gui.look", value)
        ToolTip.visible: hovered
        ToolTip.text: tipText + (on ? " · in use" : "")
        ToolTip.delay: 400
        background: Rectangle {
            radius: 14
            color: !choice.on && choice.hovered ? Style.controlHover : "transparent"
            border.width: choice.visualFocus ? 2 : 0
            border.color: Style.accent
        }
        contentItem: Item {}
    }

    Column {
        id: column
        x: 4
        y: 4
        spacing: 2
        Choice {
            id: darwan
            value: "darwan"
            tipText: "Darwan’s own look"
            Image {
                anchors.centerIn: parent
                width: 18
                height: 18
                source: "file://" + look.backend.iconPath
                sourceSize: Qt.size(36, 36)
                smooth: true
            }
        }
        Choice {
            id: system
            value: "system"
            tipText: "Your system’s Qt theme"
            Icon {
                anchors.centerIn: parent
                name: "display"
                size: 15
                color: system.on ? "#1c1c1e" : Style.text
            }
        }
    }
}
