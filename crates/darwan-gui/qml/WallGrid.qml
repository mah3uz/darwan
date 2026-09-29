pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Wallpapers in big cards, three or four across, asking for more near the end.
GridView {
    id: grid

    property var items: []
    property string mode: "online"
    property bool loading: false
    property bool more: false
    readonly property int columns: Math.max(2, Math.floor((width + 20) / (400 + 20)))

    signal open(var item, int index)
    signal wantMore()

    cellWidth: width / columns
    cellHeight: cellWidth * 9 / 16 + 20
    model: items.length
    boundsBehavior: Flickable.StopAtBounds
    cacheBuffer: 800
    clip: true
    ScrollBar.vertical: ScrollBar {}
    // Near the end, the next 25.
    onContentYChanged: if (more && !loading && contentHeight > 0 && contentY + height > contentHeight - cellHeight * 2) wantMore()

    delegate: Item {
        id: cell
        required property int index
        width: grid.cellWidth
        height: grid.cellHeight
        WallCard {
            x: 10
            width: cell.width - 20
            height: width * 9 / 16
            item: grid.items[cell.index]
            mode: grid.mode
            onOpen: grid.open(grid.items[cell.index], cell.index)
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
