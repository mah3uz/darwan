import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// One wallpaper in a grid: its picture on a tile of its own colour, badges, and its name and size on hover. Kept
// light, since a grid makes many: the picture is one shader (no layers), and the words exist only while hovered.
FocusScope {
    id: card

    required property var item
    // "library" or "online"; online cards credit their author on hover.
    property string mode: "library"
    // The Home strip: the picture only, no words on hover.
    property bool quiet: false
    // Told when the picture can't be shown, so the card is left out rather than showing a hole.
    property var backend: null
    readonly property bool lit: hover.hovered || activeFocus

    signal open()

    activeFocusOnTab: true
    Keys.onReturnPressed: open()
    Keys.onSpacePressed: open()

    HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: card.open() }

    // Solid, so it reads over any picture.
    component Badge: Rectangle {
        property alias text: label.text
        property color tint: Style.accent
        implicitWidth: label.implicitWidth + 14
        implicitHeight: 20
        radius: 10
        color: tint
        Label {
            id: label
            anchors.centerIn: parent
            color: "white"
            font.family: Style.family
            font.pixelSize: 10
            font.weight: Font.Bold
            font.letterSpacing: 0.6
        }
    }

    Item {
        id: frame
        anchors.fill: parent
        y: card.lit ? -3 : 0
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }

        // The picture's own colour while it loads: a calm tile rather than a spinner per card.
        Rectangle {
            anchors.fill: parent
            radius: 16
            color: card.item.swatches && card.item.swatches.length ? card.item.swatches[0] : Qt.rgba(1, 1, 1, 0.06)
            opacity: card.item.swatches && card.item.swatches.length ? 0.45 : 1
        }
        RoundedImage {
            id: picture
            anchors.fill: parent
            radius: 16
            source: card.item.thumb
            sourceSize.width: 512
            onStatusChanged: if (status === Image.Error && card.backend) card.backend.wallHide(card.item.key || card.item.path)
        }
        Rectangle {
            anchors.fill: parent
            radius: 16
            visible: card.lit && !card.quiet
            gradient: Gradient {
                GradientStop { position: 0.45; color: "transparent" }
                GradientStop { position: 1; color: Qt.rgba(0, 0, 0, 0.72) }
            }
        }
        Rectangle {
            anchors.fill: parent
            anchors.margins: -2
            radius: 18
            color: "transparent"
            border.width: 2
            border.color: card.item.inUse ? Style.ok : Style.accent
            visible: card.lit || card.item.inUse === true
        }

        Row {
            x: 10
            y: 10
            spacing: 6
            Badge {
                visible: card.item.isNew === true
                text: "NEW"
            }
            Badge {
                visible: card.item.kind === "video" || card.item.kind === "animated"
                text: card.item.kind === "video" ? "VIDEO" : "ANIMATED"
                tint: Style.lock
            }
            Badge {
                visible: card.mode === "online" && card.item.downloaded === true
                text: "✓ DOWNLOADED"
                tint: Style.ok
            }
        }
        Badge {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 10
            visible: card.item.inUse === true
            text: "IN USE"
            tint: Style.ok
        }

        Loader {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 14
            active: card.lit && !card.quiet
            sourceComponent: Column {
                spacing: 2
                Label {
                    width: parent.width
                    text: card.mode === "online" ? card.item.credit.source : card.item.folder
                    visible: text !== ""
                    elide: Text.ElideRight
                    font.family: Style.family
                    font.pixelSize: 11
                    color: Qt.rgba(1, 1, 1, 0.7)
                }
                Label {
                    width: parent.width
                    text: card.mode === "online" ? card.item.title : card.item.name
                    elide: Text.ElideRight
                    font.family: Style.family
                    font.pixelSize: Style.body
                    font.weight: Font.DemiBold
                    color: "white"
                }
                Label {
                    width: parent.width
                    text: card.mode === "online"
                          ? [card.item.credit.author, card.item.size].filter(s => s).join(" · ")
                          : [card.item.size, card.item.bytes].filter(s => s).join(" · ")
                    visible: text !== ""
                    elide: Text.ElideRight
                    font.family: Style.family
                    font.pixelSize: 11
                    color: Qt.rgba(1, 1, 1, 0.7)
                }
            }
        }
    }
}
