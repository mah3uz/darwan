import QtQuick
import QtQuick.Controls.Basic

Rectangle {
    id: tag

    property alias text: label.text
    property color tint: Style.accent

    implicitWidth: label.implicitWidth + 12
    implicitHeight: label.implicitHeight + 4
    radius: height / 2
    color: Qt.alpha(tint, 0.15)
    border.color: Qt.alpha(tint, 0.5)

    Label {
        id: label
        anchors.centerIn: parent
        color: tag.tint
        font.pixelSize: 10
        font.bold: true
        font.letterSpacing: 0.5
    }
}
