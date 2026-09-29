pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// One topic or source on its own page, as wallspace's category page: back, its name, Latest / Popular / Random.
Item {
    id: category

    required property Backend backend
    property string kicker: "CATEGORY"
    property string title: ""
    property var sources: []
    property string topic: ""
    property string subject: ""
    property string sort: "popular"

    signal back()
    signal open(var item, var list, int index)

    readonly property WallFeed feed: WallFeed { backend: category.backend; channel: "category" }

    function ask() {
        backend.wallSearch("category", JSON.stringify({ sources: sources, topic: topic, subject: subject, sort: sort, first: 50 }))
    }
    function show(k, t, srcs, top, subj) {
        kicker = k
        title = t
        sources = srcs
        topic = top
        subject = subj
        sort = "popular"
        ask()
    }

    WallGrid {
        id: grid
        anchors.fill: parent
        anchors.leftMargin: 46
        anchors.rightMargin: 46
        feed: category.feed
        onOpen: (item, index) => category.open(item, category.feed.items, index)
        onWantMore: category.backend.wallMore("category")

        header: Column {
            width: grid.width
            spacing: 10
            bottomPadding: 22
            Item { width: 1; height: 20 }
            AbstractButton {
                id: backButton
                width: 34
                height: 34
                hoverEnabled: true
                onClicked: category.back()
                contentItem: Item { Icon { anchors.centerIn: parent; name: "left"; size: 18; color: "white" } }
                background: Rectangle { radius: 17; color: backButton.hovered ? Qt.rgba(1, 1, 1, 0.18) : Qt.rgba(1, 1, 1, 0.08) }
            }
            Item { width: 1; height: 8 }
            Label {
                text: category.kicker
                font.family: Style.family
                font.pixelSize: 12
                font.weight: Font.DemiBold
                font.letterSpacing: 2.5
                color: Qt.rgba(1, 1, 1, 0.65)
            }
            Label {
                text: category.title
                font.family: Style.family
                font.pixelSize: 36
                font.weight: Font.Bold
                color: "white"
            }
            Segmented {
                options: [{ value: "latest", label: "Latest" }, { value: "popular", label: "Popular" }, { value: "random", label: "Random" }]
                current: category.sort
                onPicked: v => {
                    category.sort = v
                    category.ask()
                }
            }
            Label {
                visible: category.feed.model.count === 0 && !category.feed.info.loading
                topPadding: 30
                text: category.feed.info.error ? "Couldn't reach it: " + category.feed.info.error : "Nothing here with your Filter."
                font.family: Style.family
                font.pixelSize: Style.body
                color: Qt.rgba(1, 1, 1, 0.6)
            }
        }
    }
}
