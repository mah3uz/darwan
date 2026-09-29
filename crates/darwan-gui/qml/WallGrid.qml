pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Wallpapers in big cards, three or four across, asking for more near the end. The model changes in place, so the
// grid keeps its place while pictures arrive.
GridView {
    id: grid

    required property WallFeed feed
    property string mode: "online"
    readonly property bool loading: feed.info.loading === true
    readonly property bool more: feed.info.more === true
    readonly property int columns: Math.max(2, Math.floor((width + 20) / (400 + 20)))

    signal open(var item, int index)
    signal wantMore()

    cellWidth: width / columns
    cellHeight: cellWidth * 9 / 16 + 20
    model: feed.model
    boundsBehavior: Flickable.StopAtBounds
    // Rows made a screen ahead, so their pictures are decoding before they scroll in.
    cacheBuffer: height
    reuseItems: true
    clip: true
    SmoothWheel { view: grid; step: grid.cellHeight }
    // New results rise into place rather than appearing.
    add: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 320; easing.type: Easing.OutCubic }
        NumberAnimation { property: "y"; from: ViewTransition.destination.y + 24; duration: 320; easing.type: Easing.OutCubic }
    }
    // Only while scrolling, as wallspace shows it.
    ScrollIndicator.vertical: ScrollIndicator {}
    // Near the end, the next 25.
    onContentYChanged: if (more && !loading && contentHeight > 0 && contentY + height > contentHeight - cellHeight * 2) wantMore()

    delegate: Item {
        id: cell
        required property int index
        required property string json
        width: grid.cellWidth
        height: grid.cellHeight
        WallCard {
            x: 10
            width: cell.width - 20
            height: width * 9 / 16
            item: JSON.parse(cell.json)
            mode: grid.mode
            backend: grid.feed.backend
            onOpen: grid.open(grid.feed.items[cell.index], cell.index)
        }
    }
    footer: Item {
        width: grid.width
        height: 80
        Row {
            anchors.centerIn: parent
            spacing: 8
            visible: grid.loading
            Spinner { anchors.verticalCenter: parent.verticalCenter; color: Qt.rgba(1, 1, 1, 0.7) }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Loading…"
                font.family: Style.family
                font.pixelSize: Style.body
                color: Qt.rgba(1, 1, 1, 0.7)
            }
        }
    }
}
