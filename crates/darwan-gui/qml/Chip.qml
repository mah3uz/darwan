import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// A filter chip: a word, an optional colour dot and count; filled while on.
AbstractButton {
    id: chip

    property bool on: false
    property string swatch: ""
    property string count: ""

    height: 30
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    contentItem: Row {
        leftPadding: chip.swatch ? 9 : 14
        rightPadding: 14
        spacing: 6
        Rectangle {
            visible: chip.swatch !== ""
            anchors.verticalCenter: parent.verticalCenter
            width: 12
            height: 12
            radius: 6
            color: chip.swatch || "transparent"
            border.color: Qt.rgba(1, 1, 1, 0.3)
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: chip.text
            font.family: Style.family
            font.pixelSize: Style.body
            color: chip.on ? "#111111" : chip.hovered ? "white" : Qt.rgba(1, 1, 1, 0.78)
        }
        Label {
            visible: chip.count !== ""
            anchors.verticalCenter: parent.verticalCenter
            text: chip.count
            font.family: Style.family
            font.pixelSize: Style.body
            color: chip.on ? Qt.rgba(0, 0, 0, 0.5) : Qt.rgba(1, 1, 1, 0.45)
        }
    }
    background: Rectangle {
        radius: height / 2
        color: chip.on ? Qt.rgba(1, 1, 1, 0.92) : chip.hovered ? Qt.rgba(1, 1, 1, 0.16) : Qt.rgba(1, 1, 1, 0.08)
        border.color: chip.visualFocus ? Style.accent : Qt.rgba(1, 1, 1, chip.on ? 0 : 0.12)
        border.width: chip.visualFocus ? 2 : 1
        Behavior on color { ColorAnimation { duration: Style.fast } }
    }
}
