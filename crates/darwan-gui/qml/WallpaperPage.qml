pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Effects
import org.darwan

// Home, Explore and Library over one of your wallpapers at full size, as wallspace.app: sharp behind Home, blurred
// behind the others.
FocusScope {
    id: page

    required property Backend backend
    // "home", "explore" or "library"
    property string view: "home"
    property bool categoryShown: false

    signal switchPage(string page)
    signal openItem(var item, var list, int index)

    readonly property var library: backend.wallRevision >= 0 ? JSON.parse(backend.wallLibrary("", "")) : null
    readonly property var owner: backend.wallRevision >= 0 ? JSON.parse(backend.wallOwner()) : null
    readonly property var explore: backend.wallRevision >= 0 ? JSON.parse(backend.wallExplore()) : null
    property var featured: null
    property var strip: []
    property string notice: ""

    // A random picture from the Library features, with a strip of others; once, when the Library first arrives.
    function pick() {
        const pictures = library.items.filter(i => i.kind === "image" && i.thumb !== "")
        if (pictures.length === 0)
            return
        const shuffled = pictures.slice().sort(() => Math.random() - 0.5).slice(0, 14)
        strip = shuffled
        featured = shuffled[0]
    }
    onLibraryChanged: if (featured === null && library && library.prepared > 0) pick()
    Component.onCompleted: backend.wallScan()
    onViewChanged: categoryShown = false

    Rectangle { anchors.fill: parent; color: "#0c0c0e" }

    // The featured picture, its thumbnail first and then a screen-sized copy crossfading over it.
    property string featuredFull: ""
    function askFull() {
        if (featured)
            backend.wallFull(featured.path, Math.round(Math.max(page.width, 1280) * Screen.devicePixelRatio))
    }
    // Later, not now: the first pick happens while the page is still being made, and a copy already made is answered
    // at once, before anything here listens.
    onFeaturedChanged: {
        featuredFull = ""
        Qt.callLater(askFull)
    }
    Connections {
        target: page.backend
        function onWallFullReady(key, path) {
            if (page.featured && page.featured.path === key)
                page.featuredFull = path
        }
    }
    FadeImage {
        id: backdrop
        anchors.fill: parent
        duration: 600
        source: page.featuredFull !== "" ? page.featuredFull : page.featured ? page.featured.thumb : ""
        visible: false
    }
    MultiEffect {
        anchors.fill: parent
        source: backdrop
        blurEnabled: true
        blurMax: 64
        blur: page.view === "home" && !page.categoryShown ? 0 : 1
        saturation: page.view === "home" && !page.categoryShown ? 0 : -0.1
        Behavior on blur { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
        opacity: page.featured ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: 600 } }
    }
    // Dark where the text and rows sit: under the toolbar, and below the hero.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0; color: Qt.rgba(0, 0, 0, 0.45) }
            GradientStop { position: 0.14; color: Qt.rgba(0, 0, 0, 0.05) }
            GradientStop { position: 0.45; color: Qt.rgba(0, 0, 0, 0.12) }
            GradientStop { position: 0.8; color: Qt.rgba(0.05, 0.05, 0.06, 0.82) }
            GradientStop { position: 1; color: Qt.rgba(0.05, 0.05, 0.06, 0.94) }
        }
    }
    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0.05, 0.05, 0.06, 0.55)
        opacity: page.view === "home" && !page.categoryShown ? 0 : 1
        Behavior on opacity { NumberAnimation { duration: Style.slow } }
    }

    Loader {
        anchors.fill: parent
        anchors.topMargin: 84
        active: page.view === "home"
        visible: active && !page.categoryShown
        sourceComponent: WallHome {
            backend: page.backend
            library: page.library
            featured: page.featured
            strip: page.strip
            onFeaturedChanged: page.featured = featured
            onOpen: (item, list, index) => page.openItem(item, list, index)
            onSeeAll: (title, sources, subject) => {
                category.show("SOURCE", title, sources, "", subject)
                page.categoryShown = true
            }
        }
    }
    Loader {
        anchors.fill: parent
        anchors.topMargin: 84
        active: page.view === "explore" || item !== null
        visible: page.view === "explore" && !page.categoryShown
        sourceComponent: WallExplore {
            backend: page.backend
            onOpen: (item, list, index) => page.openItem(item, list, index)
            onTopic: (label, query) => {
                category.show("CATEGORY", label, [], query, "")
                page.categoryShown = true
            }
            onFilter: from => {
                filterMenu.parent = from
                filterMenu.x = (from.width - filterMenu.width) / 2
                filterMenu.y = from.height + 10
                filterMenu.open()
            }
        }
    }
    Loader {
        anchors.fill: parent
        anchors.topMargin: 84
        active: page.view === "library" || item !== null
        visible: page.view === "library" && !page.categoryShown
        sourceComponent: WallLibrary {
            backend: page.backend
            library: page.library
            onOpen: (item, list, index) => page.openItem(item, list, index)
        }
    }
    WallCategory {
        id: category
        anchors.fill: parent
        anchors.topMargin: 84
        visible: page.categoryShown
        backend: page.backend
        onBack: page.categoryShown = false
        onOpen: (item, list, index) => page.openItem(item, list, index)
    }

    // The toolbar floats over the picture.
    Item {
        id: toolbar
        width: parent.width
        height: 84
        Row {
            x: 56
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12
            Rectangle {
                width: 38
                height: 38
                radius: 11
                color: Qt.rgba(0, 0, 0, 0.35)
                anchors.verticalCenter: parent.verticalCenter
                Image {
                    anchors.centerIn: parent
                    width: 26
                    height: 26
                    source: "file://" + page.backend.iconPath
                    sourceSize: Qt.size(52, 52)
                    smooth: true
                }
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Darwan"
                font.family: Style.family
                font.pixelSize: 20
                font.weight: Font.DemiBold
                color: "white"
            }
        }
        PageSwitch {
            anchors.centerIn: parent
            current: page.view
            glass: true
            onPicked: v => page.switchPage(v)
        }
        Row {
            anchors.right: parent.right
            anchors.rightMargin: 40
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12
            Repeater {
                model: [{ glyph: "plus", tip: "Add pictures or videos to your Library" }, { glyph: "gear", tip: "Wallpaper folder and colours" }]
                delegate: AbstractButton {
                    id: round
                    required property var modelData
                    width: 44
                    height: 44
                    hoverEnabled: true
                    ToolTip.visible: hovered
                    ToolTip.text: modelData.tip
                    ToolTip.delay: 500
                    onClicked: {
                        if (modelData.glyph === "plus") {
                            addDialog.open()
                            return
                        }
                        settingsMenu.parent = round
                        settingsMenu.x = round.width - settingsMenu.width
                        settingsMenu.y = round.height + 10
                        settingsMenu.open()
                    }
                    contentItem: Item { Icon { anchors.centerIn: parent; name: round.modelData.glyph; size: 18; color: "white" } }
                    background: Rectangle {
                        radius: 22
                        color: round.hovered ? Qt.rgba(0, 0, 0, 0.55) : Qt.rgba(0, 0, 0, 0.38)
                        border.color: Qt.rgba(1, 1, 1, 0.12)
                    }
                }
            }
        }
    }

    FileDialog {
        id: addDialog
        title: "Add pictures or videos to your Library"
        fileMode: FileDialog.OpenFiles
        nameFilters: ["Pictures and videos (*.jpg *.jpeg *.png *.webp *.bmp *.gif *.mp4 *.mkv *.webm *.mov)"]
        onAccepted: {
            const files = selectedFiles.map(f => decodeURIComponent(f.toString()))
            page.notice = page.backend.wallAdd(JSON.stringify(files))
        }
    }
    FolderDialog {
        id: folderDialog
        title: "Choose your wallpaper folder"
        currentFolder: page.library ? "file://" + page.library.folder : ""
        onAccepted: page.backend.wallFolder(decodeURIComponent(selectedFolder.toString()))
    }

    // What the online sources may show; sexual content has no switch.
    Popover {
        id: filterMenu
        width: 320
        contentItem: Column {
            spacing: 2
            Label {
                width: parent.width
                leftPadding: 10
                topPadding: 8
                bottomPadding: 4
                text: "Filter"
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Repeater {
                model: page.explore ? page.explore.groups : []
                delegate: SettingRow {
                    id: groupRow
                    required property var modelData
                    required property int index
                    width: 320 - 12
                    first: groupRow.index === 0
                    label: "Show " + groupRow.modelData.label.toLowerCase()
                    Toggle {
                        on: groupRow.modelData.on
                        onFlipped: v => page.backend.wallAllow(groupRow.modelData.id, v)
                    }
                }
            }
            Label {
                width: 320 - 12
                leftPadding: 10
                rightPadding: 10
                topPadding: 8
                bottomPadding: 8
                text: "Sexual content is never shown, whatever is on here."
                wrapMode: Text.WordWrap
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
            }
        }
    }

    // The folder, what draws the wallpaper, and colour generators to run after a change.
    Popover {
        id: settingsMenu
        width: 360
        contentItem: Column {
            spacing: 2
            LookRow {
                width: 360 - 12
                first: true
                backend: page.backend
            }
            SettingRow {
                width: 360 - 12
                label: "Wallpaper folder"
                sub: page.library ? page.library.folder.replace(/^\/home\/[^/]+/, "~") : ""
                ActionButton {
                    text: "Change"
                    onActivated: {
                        settingsMenu.close()
                        folderDialog.open()
                    }
                }
            }
            SettingRow {
                width: 360 - 12
                label: page.owner && page.owner.found ? "Set through " + page.owner.name : "No wallpaper tool found"
                sub: page.owner ? page.owner.note : ""
                ActionButton {
                    glyph: "refresh"
                    flat: true
                    tip: "Look again"
                    onActivated: page.backend.wallScan()
                }
            }
            Repeater {
                model: page.owner ? page.owner.generators : []
                delegate: SettingRow {
                    id: genRow
                    required property var modelData
                    width: 360 - 12
                    label: "Run " + genRow.modelData.id + " after each change"
                    sub: genRow.modelData.reason
                    off: genRow.modelData.reason !== ""
                    Toggle {
                        on: genRow.modelData.on && genRow.modelData.reason === ""
                        enabled: genRow.modelData.reason === ""
                        onFlipped: v => page.backend.wallColours(genRow.modelData.id, v)
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 28
        visible: page.notice !== ""
        width: noticeLabel.implicitWidth + 40
        height: 44
        radius: 22
        color: Style.panel
        border.color: Style.panelBorder
        GlassEdge {}
        Label {
            id: noticeLabel
            anchors.centerIn: parent
            text: page.notice
            font.family: Style.family
            font.pixelSize: Style.body
            color: Style.text
        }
        Timer {
            running: page.notice !== ""
            interval: 3000
            onTriggered: page.notice = ""
        }
    }
}
