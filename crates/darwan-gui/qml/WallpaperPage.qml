pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Effects
import org.darwan

// Your wallpaper folder and free wallpapers online, set through whatever draws your desktop.
FocusScope {
    id: page

    required property Backend backend
    property string tab: "library"
    property string colour: ""
    property alias search: search
    // Online choices; a change asks the source again.
    property string source: "bing"
    property string sort: "popular"
    property string ratio: ""
    property string topic: "nature"
    property int onlinePage: 1

    signal switchPage(string page)
    signal openItem(var item, string mode)

    // Parsed only when the text changes; the revision is read inside the expression so the compiled binding keeps it.
    readonly property var library: backend.wallRevision >= 0 ? JSON.parse(backend.wallLibrary(tab === "library" ? search.text : "", colour)) : null
    readonly property var online: backend.wallRevision >= 0 ? JSON.parse(backend.wallOnline()) : null
    readonly property var owner: backend.wallRevision >= 0 ? JSON.parse(backend.wallOwner()) : null
    readonly property var items: tab === "library" ? library.items : online.items
    readonly property int columns: Math.max(2, Math.floor((grid.width + 18) / (300 + 18)))

    function ask(more) {
        onlinePage = more ? onlinePage + 1 : 1
        backend.wallSearch(JSON.stringify({
            source: source, text: source === "wallhaven" ? search.text : "", sort: sort, ratio: ratio,
            topic: topic, page: onlinePage
        }))
    }

    Component.onCompleted: backend.wallScan()
    onTabChanged: if (tab === "online" && online && online.items.length === 0 && !online.loading) ask(false)

    Rectangle { anchors.fill: parent; color: Style.bg }
    // Darwan's own look sits on the wallpaper in use, blurred, as the Wall sits on a theme.
    Item {
        anchors.fill: parent
        visible: Style.own
        Image {
            id: backdrop
            anchors.fill: parent
            anchors.margins: -80
            visible: false
            source: {
                const used = page.library ? page.library.items.find(i => i.inUse) : null
                return used && used.thumb ? used.thumb : ""
            }
            sourceSize.width: 512
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
        MultiEffect {
            anchors.fill: backdrop
            source: backdrop
            blurEnabled: true
            blur: 1
            blurMax: 64
            saturation: 0.4
            opacity: backdrop.status === Image.Ready ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 600 } }
        }
        Rectangle { anchors.fill: parent; color: "black"; opacity: 0.72 }
    }

    FolderDialog {
        id: folderDialog
        title: "Choose your wallpaper folder"
        currentFolder: page.library ? "file://" + page.library.folder : ""
        onAccepted: page.backend.wallFolder(selectedFolder.toString())
    }

    // The groups the online tab hides until allowed; sexual content has no switch.
    Popover {
        id: groups
        width: 320
        contentItem: Column {
            spacing: 2
            Label {
                width: parent.width
                leftPadding: 10
                topPadding: 8
                bottomPadding: 4
                text: "Allowed online"
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Repeater {
                model: page.online ? page.online.groups : []
                delegate: SettingRow {
                    id: d1
                    required property var modelData
                    required property int index
                    width: 320 - 12
                    first: d1.index === 0
                    label: d1.modelData.label
                    Toggle {
                        on: d1.modelData.on
                        onFlipped: v => page.backend.wallAllow(d1.modelData.id, v)
                    }
                }
            }
            Label {
                width: 320 - 12
                leftPadding: 10
                rightPadding: 10
                topPadding: 8
                bottomPadding: 8
                text: "Sexual content in any form is never shown, from any source, whatever is allowed here."
                wrapMode: Text.WordWrap
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
            }
        }
    }

    // Colour generators set up on this machine, run after each change when the wallpaper tool doesn't make colours.
    Popover {
        id: colourMenu
        width: 340
        contentItem: Column {
            spacing: 2
            Label {
                width: parent.width
                leftPadding: 10
                topPadding: 8
                bottomPadding: 4
                text: "Colours from the wallpaper"
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Repeater {
                model: page.owner ? page.owner.generators : []
                delegate: SettingRow {
                    id: d2
                    required property var modelData
                    required property int index
                    width: 340 - 12
                    first: d2.index === 0
                    label: "Run " + d2.modelData.id + " after each change"
                    sub: d2.modelData.reason
                    off: d2.modelData.reason !== ""
                    Toggle {
                        on: d2.modelData.on && d2.modelData.reason === ""
                        enabled: d2.modelData.reason === ""
                        onFlipped: v => page.backend.wallColours(d2.modelData.id, v)
                    }
                }
            }
        }
    }

    component Chip: AbstractButton {
        id: chip
        property bool on: false
        property string swatch: ""
        property string count: ""
        height: 28
        hoverEnabled: true
        focusPolicy: Qt.TabFocus
        contentItem: Row {
            leftPadding: chip.swatch ? 8 : 12
            rightPadding: 12
            spacing: 6
            Rectangle {
                visible: chip.swatch !== ""
                anchors.verticalCenter: parent.verticalCenter
                width: 12
                height: 12
                radius: 6
                color: chip.swatch || "transparent"
                border.color: Qt.rgba(1, 1, 1, 0.3)
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: chip.text
                font.family: Style.family
                font.pixelSize: Style.body
                color: chip.on ? Style.bg : chip.hovered ? Style.text : Style.sub
            }
            Label {
                visible: chip.count !== ""
                anchors.verticalCenter: parent.verticalCenter
                text: chip.count
                font.family: Style.family
                font.pixelSize: Style.body
                color: chip.on ? Qt.alpha(Style.bg, 0.6) : Style.muted
            }
        }
        background: Rectangle {
            radius: height / 2
            color: chip.on ? Style.text : Style.own ? Style.group : "transparent"
            border.color: chip.on ? Style.text : chip.hovered ? Style.muted : Style.sep
            border.width: chip.visualFocus ? 2 : 1
            Behavior on color { ColorAnimation { duration: Style.fast } }
        }
    }

    readonly property var hues: ({
        red: "#e5484d", orange: "#f76b15", yellow: "#ffc53d", green: "#46a758", teal: "#12a594",
        blue: "#0090ff", purple: "#8e4ec6", pink: "#d6409f", black: "#111111", grey: "#8b8d98", white: "#f0f0f0"
    })

    GridView {
        id: grid
        x: 56
        y: header.y + header.height + 8
        width: parent.width - 84
        height: parent.height - y
        clip: true
        cellWidth: width / page.columns
        cellHeight: cellWidth * 9 / 16 + 18
        model: page.items ? page.items.length : 0
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar {}
        cacheBuffer: 600
        delegate: Item {
            id: d3
            required property int index
            width: grid.cellWidth
            height: grid.cellHeight
            WallCard {
                x: 9
                y: 4
                width: parent.width - 18
                height: width * 9 / 16
                item: page.items[d3.index]
                mode: page.tab
                onOpen: page.openItem(page.items[d3.index], page.tab)
            }
        }
        footer: Item {
            width: grid.width
            height: 90
            ActionButton {
                anchors.centerIn: parent
                visible: page.tab === "online" && page.online.more && !page.online.loading
                text: "Load more"
                pill: true
                onActivated: page.ask(true)
            }
            Row {
                anchors.centerIn: parent
                spacing: 8
                visible: page.tab === "online" && page.online.loading
                Spinner { anchors.verticalCenter: parent.verticalCenter; color: Style.sub }
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    text: page.source === "wallhaven" ? "Checking each picture before it shows…" : "Loading…"
                    font.family: Style.family
                    font.pixelSize: Style.body
                    color: Style.sub
                }
            }
        }
    }

    Label {
        anchors.centerIn: grid
        width: Math.min(460, grid.width)
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        visible: page.items && page.items.length === 0 && !(page.tab === "online" && page.online.loading)
        text: {
            if (page.tab === "library") {
                if (!page.library.exists)
                    return page.library.folder + " doesn't exist yet. Choose another folder, or download one online and it's made for you."
                if (page.library.total === 0)
                    return "No pictures or videos in " + page.library.folder + " yet."
                return "Nothing matches."
            }
            if (page.online.refused)
                return "That search has a word Darwan doesn't search for (" + page.online.refused + ")."
            if (page.online.error)
                return "Couldn't reach " + page.source + ": " + page.online.error
            return "Nothing here with what's allowed."
        }
        font.family: Style.family
        font.pixelSize: Style.body
        color: Style.muted
    }

    Column {
        id: header
        x: 56
        y: toolbar.height + 22
        width: parent.width - 84
        spacing: 14

        Item {
            width: parent.width
            height: 32
            Segmented {
                id: tabs
                options: [{ value: "library", label: "Library" }, { value: "online", label: "Online" }]
                current: page.tab
                onPicked: v => page.tab = v
            }
            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: 8
                visible: page.tab === "library"
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    text: page.library ? page.library.folder.replace(/^\/home\/[^/]+/, "~") + "  ·  " + page.library.total
                        + (page.library.prepared < page.library.total ? "  (" + page.library.prepared + " ready)" : "") : ""
                    font.family: Style.family
                    font.pixelSize: Style.caption
                    color: Style.sub
                }
                ActionButton {
                    text: "Change folder"
                    glyph: "folder"
                    onActivated: folderDialog.open()
                }
                ActionButton {
                    glyph: "refresh"
                    flat: true
                    tip: "Look through the folder again"
                    onActivated: page.backend.wallScan()
                }
            }
            Row {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: 8
                visible: page.tab === "online"
                Segmented {
                    options: page.online ? page.online.sources : []
                    current: page.source
                    onPicked: v => {
                        page.source = v
                        page.ask(false)
                    }
                }
                ActionButton {
                    id: allowedButton
                    glyph: "shield"
                    text: "Allowed"
                    tip: "Which kinds of pictures the online sources may show"
                    onActivated: {
                        groups.parent = allowedButton
                        groups.x = allowedButton.width - groups.width
                        groups.y = allowedButton.height + 8
                        groups.open()
                    }
                }
            }
        }

        Flow {
            width: parent.width
            spacing: 8
            visible: page.tab === "library"
            Chip {
                text: "All"
                on: page.colour === ""
                onClicked: page.colour = ""
            }
            Repeater {
                model: page.library ? page.library.colours : []
                delegate: Chip {
                    id: d4
                    required property var modelData
                    text: d4.modelData.label
                    count: d4.modelData.count
                    swatch: page.hues[d4.modelData.value] || ""
                    on: page.colour === d4.modelData.value
                    onClicked: page.colour = on ? "" : d4.modelData.value
                }
            }
        }

        Flow {
            width: parent.width
            spacing: 8
            visible: page.tab === "online" && page.source === "wallhaven"
            Segmented {
                small: true
                options: [{ value: "popular", label: "Popular" }, { value: "latest", label: "Latest" }, { value: "random", label: "Random" }]
                current: page.sort
                onPicked: v => {
                    page.sort = v
                    page.ask(false)
                }
            }
            Repeater {
                model: [{ value: "", label: "Any shape" }, { value: "16x9", label: "Landscape (16:9)" }, { value: "21x9", label: "Ultrawide (21:9)" }, { value: "16x10", label: "16:10" }]
                delegate: Chip {
                    id: d5
                    required property var modelData
                    text: d5.modelData.label
                    on: page.ratio === d5.modelData.value
                    onClicked: {
                        page.ratio = d5.modelData.value
                        page.ask(false)
                    }
                }
            }
        }

        Flow {
            width: parent.width
            spacing: 8
            visible: page.tab === "online" && page.source === "commons"
            Repeater {
                model: page.online ? page.online.topics : []
                delegate: Chip {
                    id: d6
                    required property var modelData
                    text: d6.modelData.label
                    on: page.topic === d6.modelData.value
                    onClicked: {
                        page.topic = d6.modelData.value
                        page.ask(false)
                    }
                }
            }
        }

        Label {
            width: parent.width
            visible: page.tab === "online"
            text: page.online ? page.online.note : ""
            wrapMode: Text.WordWrap
            font.family: Style.family
            font.pixelSize: Style.caption
            color: Style.muted
        }
    }

    Rectangle {
        id: toolbar
        width: parent.width
        height: 56
        color: Style.chrome
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Style.sep }

        Row {
            x: 56
            anchors.verticalCenter: parent.verticalCenter
            spacing: 9
            Image {
                width: 26
                height: 26
                anchors.verticalCenter: parent.verticalCenter
                source: "file://" + page.backend.iconPath
                sourceSize: Qt.size(52, 52)
                smooth: true
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Darwan"
                font.family: Style.family
                font.pixelSize: Style.title
                font.weight: Font.DemiBold
                color: Style.text
            }
        }

        PageSwitch {
            anchors.centerIn: parent
            current: "wallpapers"
            onPicked: v => page.switchPage(v)
        }

        Row {
            anchors.right: parent.right
            anchors.rightMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            spacing: 10
            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 8
                    height: 8
                    radius: 4
                    color: page.owner && page.owner.found ? Style.ok : Style.warn
                }
                Label {
                    anchors.verticalCenter: parent.verticalCenter
                    text: page.owner ? (page.owner.found ? "via " + page.owner.name : "No wallpaper tool") : ""
                    font.family: Style.family
                    font.pixelSize: Style.caption
                    color: Style.sub
                    HoverHandler { id: ownerHover }
                    ToolTip.visible: ownerHover.hovered && page.owner !== null
                    ToolTip.text: page.owner ? page.owner.note : ""
                    ToolTip.delay: 400
                }
            }
            ActionButton {
                id: colourButton
                anchors.verticalCenter: parent.verticalCenter
                visible: page.owner !== null && page.owner.generators !== undefined && page.owner.generators.length > 0
                glyph: "colour"
                flat: true
                tip: "Colour generators to run after a change"
                onActivated: {
                    colourMenu.parent = colourButton
                    colourMenu.x = colourButton.width - colourMenu.width
                    colourMenu.y = colourButton.height + 8
                    colourMenu.open()
                }
            }
            TextField {
                id: search
                visible: page.tab === "library" || page.source === "wallhaven"
                width: 260
                height: 30
                leftPadding: 32
                placeholderText: page.tab === "library" ? "Search your wallpapers" : "Search Wallhaven"
                placeholderTextColor: Style.muted
                color: Style.text
                font.family: Style.family
                font.pixelSize: Style.body
                Keys.onEscapePressed: text = ""
                onAccepted: if (page.tab === "online") page.ask(false)
                background: Rectangle {
                    radius: 8
                    color: Style.control
                    border.width: search.activeFocus ? 2 : 0
                    border.color: Style.accentSoft
                    Icon { x: 10; anchors.verticalCenter: parent.verticalCenter; name: "search"; size: 14; color: Style.sub }
                }
            }
        }
    }
}
