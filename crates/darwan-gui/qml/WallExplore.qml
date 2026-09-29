pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Explore, as wallspace's: a title over the blurred picture, chips to narrow it, then every source mixed in one grid
// that grows as you scroll. A topic opens its own page.
Item {
    id: explore

    required property Backend backend
    // Empty: every source.
    property var sources: []
    property string ratio: ""
    property string sort: "popular"
    property string query: ""

    signal open(var item, var list, int index)
    signal topic(string label, string query)
    signal filter(Item from)

    readonly property var meta: backend.wallRevision >= 0 ? JSON.parse(backend.wallExplore()) : null
    readonly property WallFeed feed: WallFeed { backend: explore.backend; channel: "explore" }

    function ask() {
        backend.wallSearch("explore", JSON.stringify({ sources: sources, text: query, sort: sort, ratio: ratio, first: 50 }))
    }
    Component.onCompleted: ask()

    WallGrid {
        id: grid
        anchors.fill: parent
        anchors.leftMargin: 46
        anchors.rightMargin: 46
        feed: explore.feed
        onOpen: (item, index) => explore.open(item, explore.feed.items, index)
        onWantMore: explore.backend.wallMore("explore")

        header: Column {
            width: grid.width
            spacing: 18
            bottomPadding: 18

            Item { width: 1; height: 70 }
            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "Explore"
                font.family: Style.family
                font.pixelSize: 44
                font.weight: Font.Bold
                color: "white"
            }
            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                width: Math.min(560, parent.width)
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                text: "Free wallpapers from Wallhaven, Bing, NASA and Wikimedia Commons. Nothing sexual, ever; the rest is up to your Filter."
                font.family: Style.family
                font.pixelSize: 15
                color: Qt.rgba(1, 1, 1, 0.78)
            }
            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: 10
                TextField {
                    id: search
                    width: 360
                    height: 38
                    leftPadding: 38
                    placeholderText: "Search Wallhaven"
                    placeholderTextColor: Qt.rgba(1, 1, 1, 0.5)
                    color: "white"
                    font.family: Style.family
                    font.pixelSize: Style.body
                    onAccepted: {
                        explore.query = text
                        explore.ask()
                    }
                    Keys.onEscapePressed: {
                        text = ""
                        explore.query = ""
                        explore.ask()
                    }
                    background: Rectangle {
                        radius: 19
                        color: Qt.rgba(1, 1, 1, 0.12)
                        border.color: search.activeFocus ? Style.accent : Qt.rgba(1, 1, 1, 0.18)
                        Icon { x: 13; anchors.verticalCenter: parent.verticalCenter; name: "search"; size: 15; color: Qt.rgba(1, 1, 1, 0.7) }
                    }
                }
                AbstractButton {
                    id: filterButton
                    height: 38
                    hoverEnabled: true
                    onClicked: explore.filter(filterButton)
                    contentItem: Row {
                        leftPadding: 16
                        rightPadding: 18
                        spacing: 8
                        Icon { anchors.verticalCenter: parent.verticalCenter; name: "filter"; size: 16; color: "white" }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            text: "Filter"
                            font.family: Style.family
                            font.pixelSize: Style.body
                            font.weight: Font.Medium
                            color: "white"
                        }
                    }
                    background: Rectangle {
                        radius: 19
                        color: filterButton.hovered ? Qt.rgba(1, 1, 1, 0.22) : Qt.rgba(1, 1, 1, 0.12)
                        border.color: Qt.rgba(1, 1, 1, 0.18)
                    }
                }
            }
            Item { width: 1; height: 24 }

            // Every source until the user narrows it.
            Flow {
                width: parent.width
                spacing: 10
                Chip {
                    text: "All sources"
                    on: explore.sources.length === 0
                    onClicked: {
                        explore.sources = []
                        explore.ask()
                    }
                }
                Repeater {
                    model: explore.meta ? explore.meta.sources : []
                    delegate: Chip {
                        required property var modelData
                        text: modelData.label
                        on: explore.sources.length === 1 && explore.sources[0] === modelData.value
                        onClicked: {
                            explore.sources = on ? [] : [modelData.value]
                            explore.ask()
                        }
                    }
                }
            }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: [{ value: "21x9", label: "Ultrawide (21:9)" }, { value: "16x9", label: "Landscape (16:9)" }, { value: "16x10", label: "16:10" }]
                    delegate: Chip {
                        required property var modelData
                        text: modelData.label
                        on: explore.ratio === modelData.value
                        onClicked: {
                            explore.ratio = on ? "" : modelData.value
                            explore.ask()
                        }
                    }
                }
            }
            Flow {
                width: parent.width
                spacing: 10
                Repeater {
                    model: explore.meta ? explore.meta.topics : []
                    delegate: Chip {
                        required property var modelData
                        text: modelData.label
                        onClicked: explore.topic(modelData.label, modelData.query)
                    }
                }
            }
            Item { width: 1; height: 10 }
            Item {
                width: parent.width
                height: 40
                Column {
                    spacing: 3
                    Label {
                        text: explore.query ? "Results for “" + explore.query + "”"
                              : ({ popular: "Popular Wallpapers", latest: "Latest Wallpapers", random: "Something Different" })[explore.sort]
                        font.family: Style.family
                        font.pixelSize: 22
                        font.weight: Font.DemiBold
                        color: "white"
                    }
                    Label {
                        text: explore.feed.info.refused ? "That search has a word Darwan doesn't search for (" + explore.feed.info.refused + ")."
                              : explore.feed.info.error ? "Couldn't reach a source: " + explore.feed.info.error
                              : explore.sources.length === 0 ? "From every source, mixed" : "From " + explore.sources.join(", ")
                        font.family: Style.family
                        font.pixelSize: Style.body
                        color: Qt.rgba(1, 1, 1, 0.6)
                    }
                }
                Segmented {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    options: [{ value: "popular", label: "Popular" }, { value: "latest", label: "Latest" }, { value: "random", label: "Random" }]
                    current: explore.sort
                    onPicked: v => {
                        explore.sort = v
                        explore.ask()
                    }
                }
            }
        }
    }
}
