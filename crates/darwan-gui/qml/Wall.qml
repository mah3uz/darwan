import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import org.darwan

// The first screen, as the wallpaper pages' Home: the featured gate's theme fills the window behind a hero, with a
// strip to feature the other gate; then every theme as a card.
FocusScope {
    id: wall

    required property Backend backend
    property string filter: "all"
    property int current: 0
    property alias search: search
    // Set by the arrow keys, cleared by a real pointer move: whether focus should show on the cards.
    property bool keyboard: false
    // Which gate the hero features: "lock" or "sddm".
    property string gate: "lock"
    readonly property var featured: model.gates[gate] || null
    // 0 with the hero in view, 1 once scrolled past it: the backdrop blurs and darkens, the toolbar turns solid.
    readonly property real depth: Math.min(1, Math.max(0, scroll.contentY / Math.max(1, hero.height - toolbar.height)))
    // Each card's frame by theme id, for the Stage to grow from and shrink back into.
    property var frames: ({})

    // Parsed only when the text changes: a draft edit bumps the revision without changing the Wall.
    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property string json: backend.revision >= 0 ? backend.wall(search.text, filter) : ""
    readonly property var model: JSON.parse(json)
    readonly property int columns: Math.max(2, Math.floor((body.width + 18) / (250 + 18)))
    readonly property int count: model.order.length
    readonly property var health: backend.revision >= 0 ? JSON.parse(backend.health()) : null

    signal openTheme(string id, Item from)
    signal lockNow(string id)
    signal testSddm(string id)
    signal settings(Item from)
    signal doctor()
    signal switchPage(string page)

    // Measured from where the scroll is heading, so keys pressed during a scroll still land the card in view.
    function reveal(item) {
        const y = item.mapToItem(body, 0, 0).y + body.y
        // The toolbar floats over the top of the scroll.
        const top = (glideStep.running ? glide.targetValue : scroll.contentY) + toolbar.height
        if (y < top + 8)
            scroll.contentY = Math.max(0, y - toolbar.height - 16)
        else if (y + item.height > top + scroll.height - 8)
            scroll.contentY = y + item.height - scroll.height + 16
    }
    // Brings a theme's card into view at once and returns its frame, or null when the chip or search hides it.
    function frameOf(id) {
        const f = frames[id]
        if (!f || !f.visible)
            return null
        glide.enabled = false
        reveal(f.parent)
        glide.enabled = Qt.binding(() => !scroll.moving)
        return f
    }
    function move(by) {
        wall.keyboard = true
        wall.current = Math.max(0, Math.min(wall.count - 1, wall.current + by))
    }

    onCountChanged: current = Math.min(current, Math.max(0, count - 1))
    Keys.onLeftPressed: move(-1)
    Keys.onRightPressed: move(1)
    Keys.onUpPressed: current < columns ? search.forceActiveFocus() : move(-columns)
    Keys.onDownPressed: move(columns)

    Rectangle { anchors.fill: parent; color: Style.bg }
    // The featured theme's still, blurred enough that its clock and password field read as colour, not as a lock to
    // type into; fully blurred once the cards scroll up. MultiEffect mixes blur levels it made once, so following the
    // scroll costs nothing.
    FadeImage {
        id: backdrop
        anchors.fill: parent
        anchors.margins: -60
        duration: 500
        source: wall.featured ? "file://" + wall.featured.still : ""
        visible: false
    }
    MultiEffect {
        anchors.fill: backdrop
        source: backdrop
        visible: wall.featured !== null
        blurEnabled: true
        blurMax: 64
        blur: 0.8 + 0.2 * wall.depth
        saturation: Style.own ? -0.1 - 0.3 * wall.depth : 0
    }
    // Dark under the toolbar and the hero's text, as the wallpaper pages have it.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0; color: Qt.rgba(0, 0, 0, 0.45) }
            GradientStop { position: 0.14; color: Qt.rgba(0, 0, 0, 0.05) }
            GradientStop { position: 0.4; color: Qt.rgba(0, 0, 0, 0.12) }
            GradientStop { position: 0.75; color: Qt.rgba(0.05, 0.05, 0.06, 0.8) }
            GradientStop { position: 1; color: Style.own ? Qt.rgba(0.05, 0.05, 0.06, 0.94) : Style.bg }
        }
    }
    // Behind the hero's text, whatever colour the theme is.
    Rectangle {
        width: parent.width * 0.65
        height: parent.height
        opacity: 1 - wall.depth
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: Qt.rgba(0, 0, 0, 0.55) }
            GradientStop { position: 1; color: "transparent" }
        }
    }
    Rectangle {
        anchors.fill: parent
        color: Style.own ? "black" : Style.bg
        opacity: wall.depth * (Style.own ? 0.7 : 1)
    }

    HoverHandler {
        property point last
        onPointChanged: {
            if (point.position !== last && last !== Qt.point(0, 0))
                wall.keyboard = false
            last = point.position
        }
    }

    Flickable {
        id: scroll
        anchors.fill: parent
        contentHeight: body.height + 120
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        ScrollBar.vertical: ScrollBar {}
        SmoothWheel { view: scroll; step: (body.width - 18 * (wall.columns - 1)) / wall.columns * 9 / 16 + 32 + 22 }
        Behavior on contentY {
            id: glide
            enabled: !scroll.moving
            NumberAnimation { id: glideStep; duration: Style.medium; easing.type: Style.ease }
        }

        Column {
            id: body
            x: 56
            width: scroll.width - 84
            spacing: 14

            // The hero: the featured gate's name and actions low on its picture, the strip of both gates under them.
            Item {
                id: hero
                width: body.width
                // A little of the picture above the text, then the text and the strip.
                height: toolbar.height + Math.max(24, wall.height * 0.06) + heroText.height + 30 + strip.height
                Column {
                    id: heroText
                    anchors.left: parent.left
                    anchors.bottom: strip.top
                    anchors.bottomMargin: 30
                    width: parent.width
                    spacing: 12
                    Row {
                        spacing: 8
                        Icon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: wall.gate === "lock" ? "lock" : "login"
                            size: 14
                            color: Qt.rgba(1, 1, 1, 0.75)
                        }
                        Label {
                            text: wall.gate === "lock" ? "YOUR LOCKSCREEN" : "YOUR LOGIN SCREEN"
                            font.family: Style.family
                            font.pixelSize: 13
                            font.weight: Font.DemiBold
                            font.letterSpacing: 3
                            color: Qt.rgba(1, 1, 1, 0.75)
                        }
                    }
                    Label {
                        width: Math.min(implicitWidth, body.width * 0.6)
                        text: wall.featured ? wall.featured.name : "Not set"
                        font.family: Style.family
                        font.pixelSize: 44
                        font.weight: Font.Bold
                        color: "white"
                        elide: Text.ElideRight
                    }
                    Label {
                        visible: wall.featured !== null
                        text: !wall.featured ? "" : [
                            ({ video: "Video background", image: "Image background" })[wall.featured.background] || "Colour background",
                            wall.featured.missingFonts > 0 ? "A font it needs is missing" : ""
                        ].filter(s => s).join("    ")
                        font.family: Style.family
                        font.pixelSize: 15
                        color: Qt.rgba(1, 1, 1, 0.8)
                    }
                    Item { width: 1; height: 6 }
                    Row {
                        visible: wall.featured !== null
                        spacing: 10
                        ActionButton {
                            text: "Customise"
                            primary: true
                            pill: true
                            onActivated: wall.openTheme(wall.featured.id, wall.gate === "lock" ? lockGate.frame : sddmGate.frame)
                        }
                        ActionButton {
                            text: wall.gate === "lock" ? "Lock now" : "Test"
                            pill: true
                            onActivated: wall.gate === "lock" ? wall.lockNow(wall.featured.id) : wall.testSddm(wall.featured.id)
                        }
                    }
                }
                Row {
                    id: strip
                    // Both gates side by side, half the width each, the same height.
                    readonly property real cardHeight: Math.max(260, Math.min(420, body.width * 0.31))
                    anchors.bottom: parent.bottom
                    spacing: 18
                    GateCard {
                        id: lockGate
                        width: (body.width - strip.spacing) / 2
                        frameHeight: strip.cardHeight
                        theme: wall.model.gates.lock
                        label: "Lockscreen"
                        glyph: "lock"
                        chosen: wall.gate === "lock"
                        onPicked: wall.gate = "lock"
                    }
                    GateCard {
                        id: sddmGate
                        width: (body.width - strip.spacing) / 2
                        frameHeight: strip.cardHeight
                        theme: wall.model.gates.sddm
                        label: "Login screen"
                        glyph: "login"
                        chosen: wall.gate === "sddm"
                        onPicked: wall.gate = "sddm"
                    }
                }
            }
            Item { width: 1; height: 12 }

            Flow {
                width: parent.width
                spacing: 8
                topPadding: 10
                Repeater {
                    model: wall.model.chips
                    delegate: AbstractButton {
                        id: chip
                        required property var modelData
                        height: 28
                        hoverEnabled: true
                        focusPolicy: Qt.TabFocus
                        onClicked: wall.filter = modelData.key
                        contentItem: Row {
                            leftPadding: 12
                            rightPadding: 12
                            spacing: 5
                            Label {
                                anchors.verticalCenter: parent.verticalCenter
                                text: chip.modelData.label
                                font.family: Style.family
                                font.pixelSize: Style.body
                                color: chip.modelData.on ? Style.bg : chip.hovered ? Style.text : Style.sub
                            }
                            Label {
                                anchors.verticalCenter: parent.verticalCenter
                                text: chip.modelData.count
                                font.family: Style.family
                                font.pixelSize: Style.body
                                color: chip.modelData.on ? Qt.alpha(Style.bg, 0.6) : Style.muted
                            }
                        }
                        background: Rectangle {
                            radius: height / 2
                            color: chip.modelData.on ? Style.text : Style.own ? Style.group : "transparent"
                            border.color: chip.modelData.on ? Style.text : chip.hovered ? Style.muted : Style.sep
                            border.width: chip.visualFocus ? 2 : 1
                            Behavior on color { ColorAnimation { duration: Style.fast } }
                        }
                    }
                }
            }

            Label {
                visible: wall.count === 0
                topPadding: 30
                text: "No theme matches “" + search.text + "”."
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.muted
            }

            Repeater {
                // Keyed by count, so a changed card updates in place instead of being made again.
                model: wall.model.sections.length
                delegate: Column {
                    id: section
                    required property int index
                    readonly property var info: wall.model.sections[index]
                    width: body.width
                    spacing: 12
                    topPadding: 16
                    Row {
                        spacing: 8
                        Label {
                            text: section.info.title
                            font.family: Style.family
                            font.pixelSize: Style.title
                            font.weight: Font.DemiBold
                            color: Style.text
                        }
                        Label {
                            anchors.baseline: parent.children[0].baseline
                            text: section.info.themes.length
                            font.family: Style.family
                            font.pixelSize: Style.body
                            color: Style.muted
                        }
                    }
                    Grid {
                        columns: wall.columns
                        columnSpacing: 18
                        rowSpacing: 22
                        Repeater {
                            model: section.info.themes.length
                            delegate: ThemeCard {
                                id: themeCard
                                required property int index
                                readonly property string tid: theme.id
                                onTidChanged: wall.frames[tid] = themeCard.frame
                                Component.onCompleted: wall.frames[tid] = themeCard.frame
                                theme: section.info.themes[index]
                                width: (body.width - 18 * (wall.columns - 1)) / wall.columns
                                focus: theme.index === wall.current
                                keyboard: wall.keyboard
                                // Only the keyboard scrolls a card into view, once the grid has placed it: a filter makes the
                                // cards again, and a new card isn't placed yet when it takes the focus.
                                onActiveFocusChanged: if (activeFocus) {
                                    wall.current = theme.index
                                    if (wall.keyboard)
                                        Qt.callLater(wall.reveal, themeCard)
                                }
                                onOpen: (id, from) => wall.openTheme(id, from)
                            }
                        }
                    }
                }
            }
        }
    }

    // Clear over the hero, solid once the cards scroll under it.
    Rectangle {
        id: toolbar
        width: parent.width
        height: 56
        color: Qt.alpha(Style.chrome, wall.depth)
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Style.sep; opacity: wall.depth }

        Row {
            x: 56
            anchors.verticalCenter: parent.verticalCenter
            spacing: 9
            Image {
                width: 26
                height: 26
                anchors.verticalCenter: parent.verticalCenter
                source: "file://" + wall.backend.iconPath
                sourceSize: Qt.size(52, 52)
                smooth: true
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Darwan"
                font.family: Style.family
                font.pixelSize: Style.title
                font.weight: Font.DemiBold
                color: wall.depth > 0.5 ? Style.text : "white"
            }
        }

        PageSwitch {
            anchors.centerIn: parent
            current: "themes"
            glass: wall.depth < 0.5
            onPicked: v => wall.switchPage(v)
        }

        Row {
            anchors.right: parent.right
            anchors.rightMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            spacing: 8
            TextField {
                id: search
                width: 280
                height: 30
                leftPadding: 32
                placeholderText: "Search themes"
                placeholderTextColor: Style.muted
                color: Style.text
                font.family: Style.family
                font.pixelSize: Style.body
                Keys.onEscapePressed: text === "" ? wall.forceActiveFocus() : text = ""
                Keys.onDownPressed: {
                    wall.keyboard = true
                    wall.forceActiveFocus()
                }
                Keys.onReturnPressed: {
                    wall.keyboard = true
                    wall.forceActiveFocus()
                }
                background: Rectangle {
                    radius: 8
                    color: Style.control
                    border.width: search.activeFocus ? 2 : 0
                    border.color: Style.accentSoft
                    Icon { x: 10; anchors.verticalCenter: parent.verticalCenter; name: "search"; size: 14; color: Style.sub }
                    Label {
                        anchors.right: parent.right
                        anchors.rightMargin: 10
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !search.activeFocus && search.text === ""
                        text: "Ctrl F"
                        font.family: Style.family
                        font.pixelSize: 11
                        color: Style.muted
                    }
                }
            }
            // A light from cheap checks; clicking runs the full doctor.
            ActionButton {
                text: wall.health.ok ? "All good" : wall.health.problems.length + (wall.health.problems.length === 1 ? " thing to look at" : " things to look at")
                flat: true
                leftPadding: 30
                tip: wall.health.ok ? "Run the full check of the session, the themes and SDDM" : wall.health.problems.join("\n") + "\n\nClick for the full check."
                onActivated: wall.doctor()
                Rectangle {
                    x: 13
                    anchors.verticalCenter: parent.verticalCenter
                    width: 8
                    height: 8
                    radius: 4
                    color: wall.health.ok ? Style.ok : Style.warn
                }
            }
            ActionButton {
                id: gear
                glyph: "gear"
                flat: true
                tip: "Settings for every theme"
                onActivated: wall.settings(gear)
            }
        }
    }
}
