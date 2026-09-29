pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Your wallpaper folder, four levels deep, newest first; narrowed by name or by colour.
Item {
    id: lib

    required property Backend backend
    required property var library
    property string colour: ""
    property string query: ""

    signal open(var item, var list, int index)
    signal changeFolder()

    readonly property var hues: ({
        red: "#e5484d", orange: "#f76b15", yellow: "#ffc53d", green: "#46a758", teal: "#12a594",
        blue: "#0090ff", purple: "#8e4ec6", pink: "#d6409f", black: "#111111", grey: "#8b8d98", white: "#f0f0f0"
    })
    readonly property var shown: {
        if (!library)
            return []
        const words = query.toLowerCase().split(/\s+/).filter(w => w)
        return library.items.filter(i => (colour === "" || i.colour === colour)
                                    && words.every(w => (i.name + " " + i.folder).toLowerCase().includes(w)))
    }

    readonly property WallFeed feed: WallFeed { backend: lib.backend }
    onShownChanged: feed.set(shown)
    Component.onCompleted: feed.set(shown)

    WallGrid {
        id: grid
        anchors.fill: parent
        anchors.leftMargin: 46
        anchors.rightMargin: 46
        mode: "library"
        feed: lib.feed
        onOpen: (item, index) => lib.open(item, lib.feed.items, index)

        header: Column {
            width: grid.width
            spacing: 12
            bottomPadding: 22
            Item { width: 1; height: 30 }
            Label {
                text: "LIBRARY"
                font.family: Style.family
                font.pixelSize: 12
                font.weight: Font.DemiBold
                font.letterSpacing: 2.5
                color: Qt.rgba(1, 1, 1, 0.65)
            }
            Item {
                width: parent.width
                height: 48
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Your wallpapers"
                    font.family: Style.family
                    font.pixelSize: 36
                    font.weight: Font.Bold
                    color: "white"
                }
                Row {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 10
                    TextField {
                        width: 280
                        height: 36
                        leftPadding: 36
                        placeholderText: "Search your wallpapers"
                        placeholderTextColor: Qt.rgba(1, 1, 1, 0.5)
                        color: "white"
                        font.family: Style.family
                        font.pixelSize: Style.body
                        onTextChanged: lib.query = text
                        background: Rectangle {
                            radius: 18
                            color: Qt.rgba(1, 1, 1, 0.12)
                            border.color: Qt.rgba(1, 1, 1, 0.18)
                            Icon { x: 12; anchors.verticalCenter: parent.verticalCenter; name: "search"; size: 15; color: Qt.rgba(1, 1, 1, 0.7) }
                        }
                    }
                }
            }
            Label {
                text: lib.library ? lib.library.folder.replace(/^\/home\/[^/]+/, "~") + "  ·  " + lib.library.total + " wallpapers"
                      + (lib.library.prepared < lib.library.total ? "  ·  " + lib.library.prepared + " ready" : "") : ""
                font.family: Style.family
                font.pixelSize: Style.body
                color: Qt.rgba(1, 1, 1, 0.65)
            }
            Item { width: 1; height: 8 }
            Flow {
                width: parent.width
                spacing: 10
                Chip {
                    text: "All"
                    on: lib.colour === ""
                    onClicked: lib.colour = ""
                }
                Repeater {
                    model: lib.library ? lib.library.colours : []
                    delegate: Chip {
                        required property var modelData
                        text: modelData.label
                        count: modelData.count
                        swatch: lib.hues[modelData.value] || ""
                        on: lib.colour === modelData.value
                        onClicked: lib.colour = on ? "" : modelData.value
                    }
                }
            }
            Label {
                visible: lib.library !== null && lib.shown.length === 0
                topPadding: 40
                width: parent.width
                wrapMode: Text.WordWrap
                text: !lib.library ? "" : !lib.library.exists ? lib.library.folder + " doesn't exist yet. Change the folder with ⚙, or set a wallpaper from Explore and it's made for you."
                      : lib.library.total === 0 ? "No pictures or videos in " + lib.library.folder + " yet. Add some with +." : "Nothing matches."
                font.family: Style.family
                font.pixelSize: Style.body
                color: Qt.rgba(1, 1, 1, 0.6)
            }
        }
    }
}
