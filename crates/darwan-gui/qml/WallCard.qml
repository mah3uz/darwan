import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import org.darwan

// One wallpaper in a grid: its thumbnail, a NEW or In use badge, and its name and size on hover.
FocusScope {
    id: card

    required property var item
    // "library" or "online"; online cards credit their author on hover.
    property string mode: "library"
    // The Home strip: the picture only, no words on hover.
    property bool quiet: false
    readonly property bool lit: hover.hovered || activeFocus

    signal open()

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

    activeFocusOnTab: true
    Keys.onReturnPressed: open()
    Keys.onSpacePressed: open()

    HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: card.open() }

    Item {
        id: frame
        anchors.fill: parent
        y: card.lit ? -2 : 0
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }

        RectangularShadow {
            anchors.fill: parent
            offset.y: card.lit ? 10 : 5
            blur: card.lit ? 26 : 16
            radius: 16
            color: Qt.rgba(0, 0, 0, card.lit ? 0.5 : 0.35)
        }
        Rectangle {
            anchors.fill: parent
            anchors.margins: -2
            radius: 18
            color: "transparent"
            border.width: 2
            border.color: card.item.inUse ? Style.ok : Style.accent
            opacity: card.lit || card.item.inUse ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
        }
        Rounded {
            anchors.fill: parent
            radius: 16
            Rectangle {
                anchors.fill: parent
                color: card.item.swatches && card.item.swatches.length ? card.item.swatches[0] : Style.bgDeep
                opacity: 0.35
            }
            Spinner {
                anchors.centerIn: parent
                visible: card.item.thumb === "" || thumb.status !== Image.Ready
            }
            Image {
                id: thumb
                anchors.fill: parent
                source: card.item.thumb
                sourceSize.width: 512
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                opacity: status === Image.Ready ? 1 : 0
                Behavior on opacity { NumberAnimation { duration: Style.medium } }
            }
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: parent.height * 0.55
                opacity: card.lit && !card.quiet ? 1 : 0
                Behavior on opacity { NumberAnimation { duration: Style.fast } }
                gradient: Gradient {
                    GradientStop { position: 0; color: "transparent" }
                    GradientStop { position: 1; color: Qt.rgba(0, 0, 0, 0.72) }
                }
            }
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

        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 14
            spacing: 2
            visible: !card.quiet
            opacity: card.lit ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
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
