import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects

// A theme on the Wall: its still, its unlock animation while hovered or focused, and what it is used for.
FocusScope {
    id: card

    required property var theme
    // Focus lights a card only while the keyboard is in use; after a click the pointer decides.
    property bool keyboard: false
    readonly property bool lit: hover.hovered || (activeFocus && keyboard)
    property alias frame: frame

    signal open(string id, Item from)

    implicitHeight: frame.height + 32
    activeFocusOnTab: true
    Keys.onReturnPressed: open(theme.id, frame)
    Keys.onSpacePressed: open(theme.id, frame)

    HoverHandler { id: hover; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: card.open(card.theme.id, frame) }

    Item {
        id: frame
        width: parent.width
        height: width * 9 / 16
        y: card.lit ? -2 : 0
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }

        RectangularShadow {
            anchors.fill: parent
            offset.y: card.lit ? 10 : 6
            blur: card.lit ? 28 : 18
            radius: Style.radius
            color: Qt.rgba(0, 0, 0, card.lit ? 0.55 : 0.4)
            Behavior on offset.y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        }
        Rectangle {
            anchors.fill: parent
            anchors.margins: -2
            radius: Style.radius + 2
            color: "transparent"
            border.width: 2
            border.color: Style.accent
            opacity: card.lit ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
        }
        Rounded {
            anchors.fill: parent
            radius: Style.radius
            Rectangle { anchors.fill: parent; color: Style.bgDeep }
            Image {
                anchors.fill: parent
                source: card.theme.still ? "file://" + card.theme.still : ""
                sourceSize.width: 640
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
            Loader {
                anchors.fill: parent
                active: card.lit && card.theme.loop !== ""
                sourceComponent: AnimatedImage {
                    source: "file://" + card.theme.loop
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                    opacity: status === Image.Ready ? 1 : 0
                    Behavior on opacity { NumberAnimation { duration: 500 } }
                }
            }
        }
        Row {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: 8
            spacing: 4
            Mark { visible: card.theme.isLock; glyph: "lock"; tint: Style.lock; tip: "Your lockscreen" }
            Mark { visible: card.theme.isSddm; glyph: "login"; tint: Style.login; tip: "Your login screen" }
            Mark { visible: card.theme.missingFonts > 0; glyph: "warn"; tint: Style.warn; tip: "A font it needs is missing" }
        }
    }

    // The name and kind sit on whatever the Wall shows behind them, so a soft dark halo keeps them readable on any
    // colour; it has no edge, so it reads as shade, not as a box.
    Item {
        id: meta
        anchors.top: frame.bottom
        anchors.topMargin: 10
        width: parent.width
        height: name.height

        RectangularShadow {
            visible: Style.own
            x: -14
            y: -8
            width: Math.min(name.contentWidth, name.width) + 28
            height: name.height + 16
            radius: 16
            blur: 26
            color: Qt.rgba(0, 0, 0, 0.36)
        }
        RectangularShadow {
            visible: Style.own
            x: kind.x - 14
            y: -8
            width: kind.width + 28
            height: name.height + 16
            radius: 16
            blur: 26
            color: Qt.rgba(0, 0, 0, 0.36)
        }
        Label {
            id: name
            width: parent.width - kind.width - 10
            text: card.theme.name
            font.family: Style.family
            font.pixelSize: Style.body
            font.weight: Font.DemiBold
            color: Style.text
            elide: Text.ElideRight
        }
        Row {
            id: kind
            anchors.right: parent.right
            anchors.verticalCenter: name.verticalCenter
            spacing: 4
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                name: card.theme.background === "video" ? "film" : card.theme.background === "image" ? "image" : "colour"
                size: 12
                color: Style.sub
            }
            Label {
                text: card.theme.background === "video" ? "Video" : card.theme.background === "image" ? "Image" : "Colour"
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
            }
        }
    }
}
