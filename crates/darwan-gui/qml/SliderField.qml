import QtQuick
import QtQuick.Controls.Basic

// A slider with its value beside it; the value is sent on release, so dragging doesn't reload the preview each step.
Row {
    id: field

    property real from: 0
    property real to: 1
    property real step: 0
    property real value: 0
    property string unit: ""
    signal committed(real value)

    spacing: 10

    Slider {
        id: slider
        anchors.verticalCenter: parent.verticalCenter
        width: 150
        from: field.from
        to: field.to
        stepSize: field.step
        snapMode: field.step > 0 ? Slider.SnapAlways : Slider.NoSnap
        value: field.value
        focusPolicy: Qt.TabFocus
        onPressedChanged: if (!pressed) field.committed(Math.round(value * 100) / 100)
        onMoved: if (!pressed) field.committed(Math.round(value * 100) / 100)
        background: Rectangle {
            x: slider.leftPadding
            y: slider.topPadding + slider.availableHeight / 2 - height / 2
            width: slider.availableWidth
            height: 4
            radius: 2
            color: Style.switchOff
            Rectangle {
                width: slider.visualPosition * parent.width
                height: parent.height
                radius: 2
                color: Style.accent
            }
        }
        handle: Rectangle {
            x: slider.leftPadding + slider.visualPosition * (slider.availableWidth - width)
            y: slider.topPadding + slider.availableHeight / 2 - height / 2
            width: 18
            height: 18
            radius: 9
            color: "white"
            border.width: slider.visualFocus ? 2 : 0
            border.color: Style.accent
            scale: slider.pressed ? 1.1 : 1
            Behavior on scale { NumberAnimation { duration: 100 } }
        }
    }
    Label {
        anchors.verticalCenter: parent.verticalCenter
        width: 40
        horizontalAlignment: Text.AlignRight
        text: (Math.round(slider.value * 100) / 100) + field.unit
        font.family: Style.family
        font.pixelSize: Style.caption
        font.features: { "tnum": 1 }
        color: Style.sub
    }
}
