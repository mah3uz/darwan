import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import org.darwan

// The first screen: the two gates, what happens when you step away, and every theme as a card.
FocusScope {
    id: wall

    required property Backend backend
    property string filter: "all"
    property int current: 0
    property alias search: search
    // Set by the arrow keys, cleared by a real pointer move: whether focus should show on the cards.
    property bool keyboard: false
    // Darwan's own look sits on a theme picked at random each time the app opens, blurred once behind the glass.
    property string wallpaper: ""
    Component.onCompleted: {
        const themes = model.order
        if (themes.length > 0)
            wallpaper = "file://" + themes[Math.floor(Math.random() * themes.length)].still
    }
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
    signal saverPreview()
    signal switchPage(string page)

    // Measured from where the scroll is heading, so keys pressed during a scroll still land the card in view.
    function reveal(item) {
        const y = item.mapToItem(body, 0, 0).y + body.y
        const top = glideStep.running ? glide.targetValue : scroll.contentY
        if (y < top + 8)
            scroll.contentY = Math.max(0, y - 16)
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
    Item {
        anchors.fill: parent
        visible: Style.own && wall.wallpaper !== ""
        Image {
            id: wallpaperImage
            anchors.fill: parent
            anchors.margins: -80
            visible: false
            source: wall.wallpaper
            sourceSize.width: 640
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
        MultiEffect {
            anchors.fill: wallpaperImage
            source: wallpaperImage
            blurEnabled: true
            blur: 1
            blurMax: 64
            saturation: 0.4
            opacity: wallpaperImage.status === Image.Ready ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 600 } }
        }
        Rectangle { anchors.fill: parent; color: "black"; opacity: 0.7 }
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
        anchors.topMargin: toolbar.height
        contentHeight: body.height + 120
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        ScrollBar.vertical: ScrollBar {}
        Behavior on contentY {
            id: glide
            enabled: !scroll.moving
            NumberAnimation { id: glideStep; duration: Style.medium; easing.type: Style.ease }
        }

        // Starts past the look switch on the window's left edge.
        Column {
            id: body
            x: 56
            y: 26
            width: scroll.width - 84
            spacing: 14

            Label {
                text: "YOUR GATES"
                font.family: Style.family
                font.pixelSize: 11
                font.weight: Font.DemiBold
                font.letterSpacing: 0.9
                color: Style.sub
            }
            Row {
                id: gates
                width: parent.width
                height: Math.max(240, Math.min(400, width * 0.3))
                spacing: 16
                GateCard {
                    width: (gates.width - 16) * 0.6
                    height: gates.height
                    theme: wall.model.gates.lock
                    label: "Lockscreen"
                    glyph: "lock"
                    actionText: "Lock now"
                    onAct: wall.lockNow(theme.id)
                    onOpen: (id, from) => wall.openTheme(id, from)
                }
                GateCard {
                    width: (gates.width - 16) * 0.4
                    height: gates.height
                    theme: wall.model.gates.sddm
                    label: "Login screen"
                    glyph: "login"
                    actionText: "Test"
                    onAct: wall.testSddm(theme.id)
                    onOpen: (id, from) => wall.openTheme(id, from)
                }
            }

            StepAway {
                width: parent.width
                backend: wall.backend
                onPreview: wall.saverPreview()
            }

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
                                onActiveFocusChanged: if (activeFocus) {
                                    wall.current = theme.index
                                    wall.reveal(themeCard)
                                }
                                onOpen: (id, from) => wall.openTheme(id, from)
                            }
                        }
                    }
                }
            }
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
                color: Style.text
            }
        }

        PageSwitch {
            anchors.centerIn: parent
            current: "themes"
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
