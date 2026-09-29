pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// A titled row that scrolls sideways, with ‹ › at its end and "See all" on its title, as wallspace's picks.
Column {
    id: row

    property string title: ""
    property string subtitle: ""
    property var items: []
    property string mode: "online"
    property real cardWidth: 460
    property bool loading: false
    property bool canSeeAll: true

    signal open(var item, int index)
    signal seeAll()

    spacing: 14
    visible: items.length > 0 || loading

    Item {
        width: row.width
        height: heading.height
        Column {
            id: heading
            spacing: 3
            Row {
                spacing: 6
                Label {
                    text: row.title
                    font.family: Style.family
                    font.pixelSize: 22
                    font.weight: Font.DemiBold
                    color: "white"
                }
                AbstractButton {
                    id: seeAllButton
                    visible: row.canSeeAll
                    anchors.verticalCenter: parent.verticalCenter
                    width: 22
                    height: 22
                    hoverEnabled: true
                    onClicked: row.seeAll()
                    contentItem: Item { Icon { anchors.centerIn: parent; name: "right"; size: 18; color: seeAllButton.hovered ? "white" : Qt.rgba(1, 1, 1, 0.7) } }
                    background: null
                }
            }
            Label {
                text: row.subtitle
                font.family: Style.family
                font.pixelSize: Style.body
                color: Qt.rgba(1, 1, 1, 0.6)
            }
        }
        Row {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            spacing: 6
            Repeater {
                model: [-1, 1]
                delegate: AbstractButton {
                    id: arrow
                    required property int modelData
                    width: 32
                    height: 32
                    hoverEnabled: true
                    enabled: arrow.modelData < 0 ? !list.atXBeginning : !list.atXEnd
                    opacity: enabled ? 1 : 0.35
                    onClicked: list.contentX = Math.max(0, Math.min(list.contentWidth - list.width, list.contentX + arrow.modelData * (row.cardWidth + 18) * 2))
                    contentItem: Item { Icon { anchors.centerIn: parent; name: arrow.modelData < 0 ? "left" : "right"; size: 18; color: "white" } }
                    background: Rectangle { radius: 16; color: arrow.hovered ? Qt.rgba(1, 1, 1, 0.16) : "transparent" }
                }
            }
        }
    }

    ListView {
        id: list
        width: row.width
        height: row.cardWidth * 9 / 16 + 12
        orientation: ListView.Horizontal
        spacing: 18
        clip: false
        model: row.items.length
        boundsBehavior: Flickable.StopAtBounds
        Behavior on contentX { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
        delegate: WallCard {
            required property int index
            width: row.cardWidth
            height: row.cardWidth * 9 / 16
            item: row.items[index]
            mode: row.mode
            onOpen: row.open(row.items[index], index)
        }
        Spinner {
            anchors.verticalCenter: parent.verticalCenter
            x: 10
            visible: row.loading && row.items.length === 0
            color: Qt.rgba(1, 1, 1, 0.7)
            size: 22
        }
    }
}
