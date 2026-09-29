import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts

// One setting: its name on the left, its control on the right. A changed setting gets a dot and, on hover, a reset.
Item {
    id: row

    property string label: ""
    property string sub: ""
    property bool changed: false
    property bool off: false
    property bool first: false
    property bool stacked: false
    default property alias control: slot.data

    signal reset()

    implicitHeight: Math.max(40, layout.implicitHeight + 14)
    opacity: off ? 0.4 : 1
    Behavior on opacity { NumberAnimation { duration: Style.medium } }

    HoverHandler { id: hover }

    Rectangle {
        visible: !row.first
        x: 12
        width: parent.width - 12
        height: 1
        color: Style.sep
    }

    GridLayout {
        id: layout
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: 12
        anchors.rightMargin: 10
        columns: row.stacked ? 1 : 3
        columnSpacing: 8
        rowSpacing: 8

        Column {
            id: title
            Layout.fillWidth: true
            spacing: 1
            Label {
                width: parent.width
                text: row.label
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.text
                wrapMode: Text.WordWrap
                Rectangle {
                    visible: row.changed
                    x: -8
                    y: parent.height / 2 - 2
                    width: 4
                    height: 4
                    radius: 2
                    color: Style.accent
                }
            }
            Label {
                width: parent.width
                visible: row.sub !== ""
                text: row.sub
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
                wrapMode: Text.WordWrap
            }
        }
        ActionButton {
            visible: !row.stacked
            glyph: "reset"
            flat: true
            compact: true
            tip: "Back to the default"
            opacity: row.changed && (hover.hovered || activeFocus) ? 1 : 0
            enabled: row.changed && !row.off
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
            onActivated: row.reset()
        }
        Item {
            id: slot
            Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
            Layout.fillWidth: row.stacked
            implicitWidth: childrenRect.width
            implicitHeight: childrenRect.height
        }
    }
}
