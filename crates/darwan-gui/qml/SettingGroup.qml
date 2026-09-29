import QtQuick
import QtQuick.Controls.Basic

// A titled box of settings that folds away, with its notes under it.
Column {
    id: group

    property string title: ""
    property int changed: 0
    property var notes: []
    property bool open: true
    property bool foldable: true
    default property alias rows: box.data

    spacing: 6

    AbstractButton {
        width: parent.width
        height: 24
        enabled: group.foldable
        focusPolicy: Qt.TabFocus
        onClicked: group.open = !group.open
        contentItem: Item {}
        Row {
            anchors.verticalCenter: parent.verticalCenter
            x: 4
            spacing: 8
            Label {
                text: group.title
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Label {
                visible: group.changed > 0
                text: group.changed + " changed"
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.accent
                anchors.baseline: parent.children[0].baseline
            }
        }
        Icon {
            visible: group.foldable
            anchors.right: parent.right
            anchors.rightMargin: 4
            anchors.verticalCenter: parent.verticalCenter
            name: "down"
            size: 13
            color: Style.muted
            rotation: group.open ? 0 : -90
            Behavior on rotation { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        }
    }

    Rectangle {
        width: parent.width
        height: group.open ? box.implicitHeight : 0
        radius: Style.radius
        color: Style.group
        clip: true
        opacity: group.open ? 1 : 0
        Behavior on height { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        Behavior on opacity { NumberAnimation { duration: Style.fast } }
        Column {
            id: box
            width: parent.width
        }
    }

    Repeater {
        model: group.open ? group.notes : []
        delegate: Label {
            required property string modelData
            x: 12
            width: group.width - 24
            text: modelData
            font.family: Style.family
            font.pixelSize: Style.caption
            color: Style.sub
            wrapMode: Text.WordWrap
        }
    }
}
