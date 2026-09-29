import QtQuick
import QtQuick.Controls.Basic

// A small round badge on a card: an icon in a colour, with a tooltip saying what it means.
Rectangle {
    id: mark

    property string glyph: ""
    property color tint: Style.text
    property string tip: ""

    implicitWidth: 24
    implicitHeight: 24
    radius: 12
    color: Qt.rgba(0, 0, 0, 0.55)

    Icon {
        anchors.centerIn: parent
        name: mark.glyph
        size: 13
        color: mark.tint
    }
    HoverHandler { id: hover }
    ToolTip.visible: hover.hovered && tip !== ""
    ToolTip.text: tip
    ToolTip.delay: 400
}
