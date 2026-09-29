import QtQuick
import QtQuick.Controls.Basic

// A row in a pop-up menu: a label, a hint on the right, and a reason instead when it can't run.
AbstractButton {
    id: row

    property string hint: ""
    property string reason: ""
    readonly property bool available: reason === ""

    signal chosen()

    width: parent ? parent.width : implicitWidth
    implicitWidth: content.implicitWidth + 24
    height: 32
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    onClicked: if (available) chosen()

    ToolTip.visible: hovered && !available
    ToolTip.text: reason
    ToolTip.delay: 300

    background: Rectangle {
        radius: Style.small
        color: row.hovered && row.available ? Style.accent : "transparent"
    }
    contentItem: Item {}
    Row {
        id: content
        x: 10
        anchors.verticalCenter: parent.verticalCenter
        width: row.width - 20
        Label {
            width: parent.width - hintLabel.width
            text: row.text
            font.family: Style.family
            font.pixelSize: Style.body
            color: !row.available ? Style.muted : row.hovered ? Style.accentText : Style.text
            elide: Text.ElideRight
        }
        Label {
            id: hintLabel
            text: row.hint
            font.family: Style.family
            font.pixelSize: Style.caption
            color: row.hovered && row.available ? Qt.alpha(Style.accentText, 0.7) : Style.muted
        }
    }
}
