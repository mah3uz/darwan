import QtQuick
import QtQuick.Controls.Basic

// A small label over the preview, with a spinner while something loads.
Rectangle {
    id: pill

    property string text: ""
    property bool busy: false
    property bool warn: false
    property bool shown: true

    width: row.width + 20
    height: 26
    radius: 13
    color: Qt.rgba(0, 0, 0, 0.55)
    opacity: shown ? 1 : 0
    visible: opacity > 0
    Behavior on opacity { NumberAnimation { duration: Style.medium } }

    Row {
        id: row
        x: 10
        anchors.verticalCenter: parent.verticalCenter
        spacing: 8
        Spinner {
            visible: pill.busy
            anchors.verticalCenter: parent.verticalCenter
            size: 12
        }
        Label {
            text: pill.text
            font.family: Style.family
            font.pixelSize: Style.caption
            color: pill.warn ? Style.warn : "white"
        }
    }
}
