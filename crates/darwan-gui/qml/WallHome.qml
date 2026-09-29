pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Home, as wallspace's: the featured wallpaper fills the window, a strip of yours under it, then rows to browse.
Flickable {
    id: home

    required property Backend backend
    required property var library
    // The featured one, from the Library; picking another in the strip features it.
    property var featured: null
    property var strip: []

    signal open(var item, var list, int index)
    signal seeAll(string title, var sources, string subject)

    readonly property WallFeed bing: WallFeed { backend: home.backend; channel: "home-bing" }
    readonly property WallFeed wallhaven: WallFeed { backend: home.backend; channel: "home-wallhaven" }
    readonly property WallFeed apod: WallFeed { backend: home.backend; channel: "home-apod" }
    readonly property WallFeed commons: WallFeed { backend: home.backend; channel: "home-commons" }
    readonly property WallFeed recent: WallFeed { backend: home.backend }
    onLibraryChanged: if (library) recent.set(library.items.slice(0, 16))

    contentHeight: body.height + 60
    boundsBehavior: Flickable.StopAtBounds
    ScrollIndicator.vertical: ScrollIndicator {}

    Component.onCompleted: {
        backend.wallSearch("home-bing", JSON.stringify({ sources: ["bing"], first: 16 }))
        backend.wallSearch("home-wallhaven", JSON.stringify({ sources: ["wallhaven"], sort: "popular", first: 24 }))
        backend.wallSearch("home-apod", JSON.stringify({ sources: ["apod"], first: 20 }))
        backend.wallSearch("home-commons", JSON.stringify({ sources: ["commons"], subject: "nature", first: 20 }))
    }

    Column {
        id: body
        x: 56
        width: home.width - 112
        spacing: 44

        // The hero: the text sits low on the picture, the strip just under it.
        Item {
            width: body.width
            height: Math.max(420, home.height * 0.62)
            Column {
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                spacing: 12
                visible: home.featured !== null
                Label {
                    text: "FEATURED"
                    font.family: Style.family
                    font.pixelSize: 13
                    font.weight: Font.DemiBold
                    font.letterSpacing: 3
                    color: Qt.rgba(1, 1, 1, 0.75)
                }
                Label {
                    text: home.featured ? home.featured.name : ""
                    font.family: Style.family
                    font.pixelSize: 44
                    font.weight: Font.Bold
                    color: "white"
                    width: Math.min(implicitWidth, body.width * 0.6)
                    elide: Text.ElideRight
                }
                Label {
                    text: home.featured ? [home.featured.folder || "Your Library", home.featured.size, home.featured.bytes].filter(s => s).join("    ") : ""
                    font.family: Style.family
                    font.pixelSize: 15
                    color: Qt.rgba(1, 1, 1, 0.8)
                }
                Item { width: 1; height: 8 }
                AbstractButton {
                    id: view
                    height: 44
                    hoverEnabled: true
                    onClicked: home.open(home.featured, home.strip, home.strip.indexOf(home.featured))
                    contentItem: Row {
                        leftPadding: 22
                        rightPadding: 20
                        spacing: 10
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            text: "View Wallpaper"
                            font.family: Style.family
                            font.pixelSize: 15
                            font.weight: Font.Medium
                            color: "white"
                        }
                        Icon { anchors.verticalCenter: parent.verticalCenter; name: "external"; size: 16; color: "white" }
                    }
                    background: Rectangle {
                        radius: 22
                        color: view.hovered ? Qt.rgba(1, 1, 1, 0.34) : Qt.rgba(1, 1, 1, 0.22)
                        border.color: Qt.rgba(1, 1, 1, 0.28)
                    }
                }
            }
        }

        ListView {
            id: stripView
            width: body.width + 56
            height: 150
            orientation: ListView.Horizontal
            spacing: 18
            clip: false
            model: home.strip.length
            boundsBehavior: Flickable.StopAtBounds
            delegate: Item {
                id: slot
                required property int index
                readonly property var item: home.strip[index]
                readonly property bool chosen: home.featured !== null && item.path === home.featured.path
                width: 260
                height: 146
                WallCard {
                    anchors.fill: parent
                    item: slot.item
                    mode: "library"
                    quiet: true
                    onOpen: home.featured = slot.item
                }
                Rectangle {
                    anchors.fill: parent
                    anchors.margins: -3
                    radius: Style.radius + 5
                    color: "transparent"
                    border.width: 2
                    border.color: "white"
                    visible: slot.chosen
                }
            }
        }

        WallRow {
            width: body.width
            title: "Today on Bing"
            subtitle: "Microsoft's images of the day, for use as wallpapers"
            feed: home.bing
            onOpen: (item, index) => home.open(item, home.bing.items, index)
            onSeeAll: home.seeAll("Bing", ["bing"], "")
        }
        WallRow {
            width: body.width
            title: "Popular on Wallhaven"
            subtitle: "This year's favourites, each checked before it shows"
            feed: home.wallhaven
            onOpen: (item, index) => home.open(item, home.wallhaven.items, index)
            onSeeAll: home.seeAll("Wallhaven", ["wallhaven"], "")
        }
        WallRow {
            width: body.width
            title: "Astronomy Picture of the Day"
            subtitle: "From NASA, day by day"
            feed: home.apod
            onOpen: (item, index) => home.open(item, home.apod.items, index)
            onSeeAll: home.seeAll("NASA APOD", ["apod"], "")
        }
        WallRow {
            width: body.width
            title: "Featured on Wikimedia Commons"
            subtitle: "Freely licensed nature photography"
            feed: home.commons
            onOpen: (item, index) => home.open(item, home.commons.items, index)
            onSeeAll: home.seeAll("Wikimedia Commons", ["commons"], "nature")
        }
        WallRow {
            width: body.width
            title: "Recently added"
            subtitle: "The newest in your Library"
            mode: "library"
            canSeeAll: false
            feed: home.recent
            onOpen: (item, index) => home.open(item, home.recent.items, index)
        }
    }
}
