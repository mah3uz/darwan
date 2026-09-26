import QtQuick
import QtQuick.Controls.Basic

// Unavailable actions stay visible and say why when hovered or clicked, as the TUI does.
Button {
    id: button

    property string reason: ""
    property bool primary: false
    readonly property bool available: reason === ""

    signal activated()
    signal refused(string reason)

    hoverEnabled: true
    onClicked: available ? activated() : refused(reason)

    ToolTip.visible: hovered && !available
    ToolTip.text: reason
    ToolTip.delay: 300

    contentItem: Label {
        text: button.text
        color: !button.available ? Style.muted : button.primary ? Style.crust : Style.text
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    background: Rectangle {
        implicitHeight: 32
        implicitWidth: 64
        radius: 6
        color: !button.available ? Style.mantle
             : button.primary ? (button.down ? Qt.darker(Style.accent, 1.2) : Style.accent)
             : (button.down ? Style.border : button.hovered ? Qt.lighter(Style.surface, 1.2) : Style.surface)
        border.color: button.visualFocus ? Style.accent : button.available ? "transparent" : Style.surface
        border.width: button.visualFocus ? 2 : 1
    }
}
