import QtQuick
import QtQuick.Controls.Basic

// Joined buttons; the highlight slides to the chosen one.
Rectangle {
    id: seg

    // [{ value, label, badge }]
    property var options: []
    property string current: ""
    property bool small: false
    property bool stretch: false
    property Item chosen: null

    signal picked(string value)

    implicitHeight: small ? 24 : 30
    implicitWidth: row.implicitWidth + 4
    radius: small ? 7 : 9
    color: Style.control

    Rectangle {
        y: 2
        height: parent.height - 4
        x: seg.chosen ? seg.chosen.x + 2 : 2
        width: seg.chosen ? seg.chosen.width : 0
        visible: seg.chosen !== null
        radius: parent.radius - 2
        color: Style.own ? Qt.rgba(1, 1, 1, 0.22) : Style.accent
        Behavior on x { NumberAnimation { duration: Style.fast; easing.type: Style.ease } }
        Behavior on width { NumberAnimation { duration: Style.fast; easing.type: Style.ease } }
    }

    Row {
        id: row
        x: 2
        y: 2
        Repeater {
            model: seg.options
            delegate: AbstractButton {
                id: option
                required property var modelData
                readonly property bool on: modelData.value === seg.current
                width: seg.stretch ? (seg.width - 4) / seg.options.length : implicitWidth
                height: seg.height - 4
                implicitWidth: content.implicitWidth + (seg.small ? 20 : 26)
                hoverEnabled: true
                focusPolicy: Qt.TabFocus
                onClicked: if (!on) seg.picked(modelData.value)
                Binding { when: option.on; target: seg; property: "chosen"; value: option }
                contentItem: Item {}
                Row {
                    id: content
                    anchors.centerIn: parent
                    spacing: 6
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: option.modelData.label
                        font.family: Style.family
                        font.pixelSize: seg.small ? Style.caption : Style.body
                        font.weight: option.on ? Font.DemiBold : Font.Normal
                        color: option.on ? (Style.own ? "white" : Style.accentText) : option.hovered ? Style.text : Style.sub
                        Behavior on color { ColorAnimation { duration: Style.fast } }
                    }
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: (option.modelData.badge || 0) > 0
                        implicitWidth: Math.max(17, badge.implicitWidth + 10)
                        implicitHeight: 17
                        radius: 9
                        color: option.on ? (Style.own ? "white" : Style.accentText) : Style.accent
                        Label {
                            id: badge
                            anchors.centerIn: parent
                            text: option.modelData.badge || ""
                            font.pixelSize: 10
                            font.bold: true
                            color: option.on ? Style.accent : Style.accentText
                        }
                    }
                }
                Rectangle {
                    anchors.fill: parent
                    radius: seg.radius - 2
                    color: "transparent"
                    border.width: option.visualFocus ? 2 : 0
                    border.color: Style.accent
                }
            }
        }
    }
}
