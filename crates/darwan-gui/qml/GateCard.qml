import QtQuick
import QtQuick.Controls.Basic

// One of the two gates in the hero's strip: its theme's still, cropped to the card, its unlock animation while hovered,
// a ring while it's the one featured. Picking it features it.
Item {
    id: gate

    // A card from model::wall, or null when nothing is set for this gate.
    property var theme: null
    property string label: ""
    property string glyph: ""
    property bool chosen: false
    property real frameHeight: width * 9 / 16
    property alias frame: frame

    signal picked()

    implicitWidth: 260
    implicitHeight: frame.height + 30

    HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: gate.picked() }

    Item {
        id: frame
        width: parent.width
        height: gate.frameHeight
        y: hover.hovered && !gate.chosen ? -2 : 0
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }

        Rectangle { anchors.fill: parent; radius: 14; color: Style.bgDeep }
        RoundedImage {
            anchors.fill: parent
            radius: 14
            visible: gate.theme !== null
            source: gate.theme ? "file://" + gate.theme.still : ""
            sourceSize.width: 1280
        }
        // Offscreen layers only while the animation plays.
        Loader {
            anchors.fill: parent
            active: hover.hovered && gate.theme !== null && gate.theme.loop !== ""
            sourceComponent: Rounded {
                radius: 14
                AnimatedImage {
                    anchors.fill: parent
                    source: "file://" + gate.theme.loop
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                    opacity: status === Image.Ready ? 1 : 0
                    Behavior on opacity { NumberAnimation { duration: 600 } }
                }
            }
        }
        Rectangle {
            anchors.fill: parent
            anchors.margins: -3
            radius: 17
            color: "transparent"
            border.width: 2
            border.color: "white"
            opacity: gate.chosen ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
        }
    }
    Row {
        anchors.top: frame.bottom
        anchors.topMargin: 10
        spacing: 6
        Icon { anchors.verticalCenter: parent.verticalCenter; name: gate.glyph; size: 13; color: Qt.rgba(1, 1, 1, 0.8) }
        Label {
            text: gate.label + (gate.theme ? "  ·  " + gate.theme.name : "  ·  Not set")
            font.family: Style.family
            font.pixelSize: Style.caption
            font.weight: Font.Medium
            color: Qt.rgba(1, 1, 1, gate.chosen ? 0.95 : 0.7)
        }
    }
}
