import QtQuick
import QtQuick.Effects

// Clips what it holds to a rounded rectangle, anti-aliased.
Item {
    id: rounded

    property real radius: Style.radius
    default property alias content: holder.data

    Item {
        id: holder
        anchors.fill: parent
        visible: false
        layer.enabled: true
    }
    Rectangle {
        id: mask
        anchors.fill: parent
        radius: rounded.radius
        visible: false
        layer.enabled: true
    }
    MultiEffect {
        anchors.fill: parent
        source: holder
        maskEnabled: true
        maskSource: mask
        maskThresholdMin: 0.5
        maskSpreadAtMin: 1
    }
}
