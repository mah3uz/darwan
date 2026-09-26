import QtQuick
import QtQuick.Controls.Basic

Rectangle {
    id: badge

    property string kind: "ok"
    property int size: 20
    readonly property color tint: kind === "ok" ? Style.ok : kind === "warn" ? Style.warn : Style.error

    implicitWidth: size
    implicitHeight: size
    radius: size / 2
    color: Qt.alpha(tint, 0.18)
    border.color: Qt.alpha(tint, 0.6)

    Label {
        anchors.centerIn: parent
        text: badge.kind === "ok" ? "✓" : badge.kind === "warn" ? "!" : "✕"
        color: badge.tint
        font.bold: true
        font.pixelSize: Math.round(badge.size * 0.55)
    }
}
