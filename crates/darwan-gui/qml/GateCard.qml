import QtQuick
import QtQuick.Controls.Basic

// One of the two gates on the Wall: the theme on it, its unlock animation while hovered, and what you can do there.
Item {
    id: gate

    // A card from model::wall, or null when nothing is set for this gate.
    property var theme: null
    property string label: ""
    property string glyph: ""
    property string actionText: ""
    property string actionReason: ""

    signal act()
    signal open(string id, Item from)

    HoverHandler { id: hover }

    Rounded {
        id: frame
        anchors.fill: parent
        radius: 16
        Rectangle { anchors.fill: parent; color: Style.bgDeep }
        Image {
            anchors.fill: parent
            visible: gate.theme !== null
            source: gate.theme ? "file://" + gate.theme.still : ""
            sourceSize.width: 1280
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
        Loader {
            anchors.fill: parent
            active: hover.hovered && gate.theme !== null && gate.theme.loop !== ""
            sourceComponent: AnimatedImage {
                source: "file://" + gate.theme.loop
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                opacity: status === Image.Ready ? 1 : 0
                Behavior on opacity { NumberAnimation { duration: 600 } }
            }
        }
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.35; color: "transparent" }
                GradientStop { position: 1; color: Qt.rgba(0, 0, 0, 0.78) }
            }
        }
    }

    Row {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 20
        spacing: 12
        Column {
            width: parent.width - actions.width - 12
            spacing: 4
            Row {
                spacing: 6
                Icon { anchors.verticalCenter: parent.verticalCenter; name: gate.glyph; size: 13; color: Qt.rgba(1, 1, 1, 0.75) }
                Label {
                    text: gate.label.toUpperCase()
                    font.family: Style.family
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                    font.letterSpacing: 0.9
                    color: Qt.rgba(1, 1, 1, 0.75)
                }
            }
            Label {
                width: parent.width
                text: gate.theme ? gate.theme.title : "Not set"
                font.family: Style.family
                font.pixelSize: 24
                font.weight: Font.Bold
                color: "white"
                elide: Text.ElideRight
            }
        }
        Row {
            id: actions
            anchors.bottom: parent.bottom
            spacing: 8
            ActionButton {
                visible: gate.theme !== null
                text: gate.actionText
                reason: gate.actionReason
                pill: true
                onActivated: gate.act()
            }
            ActionButton {
                visible: gate.theme !== null
                text: "Customise"
                primary: true
                pill: true
                onActivated: gate.open(gate.theme.id, frame)
            }
        }
    }
}
