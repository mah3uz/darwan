import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import QtQuick.Shapes
import QtQuick.Window
import org.darwan

// One theme, full window: the live theme over its own still, the bars around it, and room for the settings panel.
FocusScope {
    id: stage

    required property Backend backend
    // The Wall's order ({ id, title, still }), which ←/→ and the strip step through.
    property var order: []
    property int current: -1
    property string mode: "lock"
    property bool inspectorOpen: true
    property real inspectorWidth: 392
    property bool shown: false
    property bool comparing: false
    // How many of this theme's own settings differ from how it ships; "Compare changes" only makes sense above 0.
    property int changes: 0
    property bool placing: false
    // True once the frame has grown into place: only then does the live theme load, so the two don't fight for
    // the frames of the animation.
    property bool settled: false
    readonly property bool resizing: widthStep.running || heightStep.running
    // A picture of each theme as it last looked live, by theme and mode, shown at once when it opens again while the
    // live theme loads behind it. The last dozen are kept; each is a texture the size of the frame.
    property var snapshots: ({})
    property var snapshotOrder: []
    readonly property string snapshotKey: themeId + "|" + mode
    // The open theme's settings as the draft has them: when they change, the live theme reloads to show it. The
    // Loader's short delay folds a run of changes (a dragged slider, a typed colour) into one reload.
    readonly property string settings: backend.revision >= 0 && themeId !== "" ? backend.fields(themeId) : ""
    onSettingsChanged: if (settled) live.reload()
    readonly property string snapshot: snapshots[snapshotKey] ? snapshots[snapshotKey].url : ""
    function keep(key, grab) {
        const all = Object.assign({}, snapshots)
        all[key] = grab
        const order = snapshotOrder.filter(k => k !== key).concat([key])
        while (order.length > 12)
            delete all[order.shift()]
        snapshotOrder = order
        snapshots = all
    }
    property rect from: Qt.rect(0, 0, 0, 0)
    property alias preview: live
    property alias toast: toast
    property alias useButton: useButton
    // What the Stage covers: while it is open, focus that lands there (a theme's late grab handed "past" the preview)
    // comes back to the Stage.
    property Item behind: null

    readonly property string themeId: current >= 0 && current < order.length ? order[current].id : ""
    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property var details: backend.revision >= 0 ? (themeId === "" ? null : JSON.parse(backend.details(themeId))) : null
    readonly property var avail: backend.revision >= 0 ? JSON.parse(backend.availability()) : null
    readonly property string busyReason: backend.busy ? "wait for the running command to finish" : ""
    readonly property bool idle: idleTimer.idle && !menu.opened && !barHover.hovered && !topHover.hovered && !stripHover.hovered
    // From the Stage-wide pointer: hover reaches only the item under it and its parents, not a zone beside the bar.
    readonly property bool nearBottom: pointer.hovered && pointer.point.position.y > height - 180 && pointer.point.position.x < freeWidth
    readonly property real freeWidth: width - (inspectorOpen ? inspectorWidth + 24 : 0)
    readonly property real aspect: Screen.width / Math.max(1, Screen.height)
    readonly property rect target: {
        const left = 56, right = freeWidth - 32, top = 72, bottom = height - 100
        let w = right - left, h = w / aspect
        if (h > bottom - top) {
            h = bottom - top
            w = h * aspect
        }
        return Qt.rect(left + (right - left - w) / 2, top + (bottom - top - h) / 2, w, h)
    }

    signal back()
    signal run(var args, string title)
    signal check()
    signal use(Item from)

    // The frame starts where the card was and grows into place; closing shrinks it back into the card.
    function open(index, rect) {
        placing = true
        from = rect
        current = index
        settled = false
        state = "card"
        shown = true
        placing = false
        state = ""
        settling.restart()
        keys.forceActiveFocus()
    }
    function close(rect) {
        settled = false
        settling.stop()
        from = rect
        state = rect.width > 0 ? "card" : "gone"
        closing.restart()
    }
    function step(by) {
        if (order.length > 0)
            current = (current + by + order.length) % order.length
    }

    visible: shown

    // Where the Stage's keys are read. Not the Stage itself: once a theme has taken focus, the Stage's own focus
    // leads back into the preview, whose guard would hand it away again.
    Item {
        id: keys
        focus: true
    }
    Keys.onLeftPressed: step(-1)
    Keys.onRightPressed: step(1)
    Keys.onEscapePressed: back()
    Keys.onPressed: e => {
        if (e.key === Qt.Key_Backslash && !e.isAutoRepeat && changes > 0)
            comparing = true
    }
    Keys.onReleased: e => {
        if (e.key === Qt.Key_Backslash && !e.isAutoRepeat)
            comparing = false
    }

    // Pictured a moment after it loads, once its intro has played, so the picture is how it looks at rest.
    Timer {
        id: picture
        property string key: ""
        interval: 1500
        onTriggered: {
            const key = picture.key
            if (key === stage.snapshotKey && live.loaded && !stage.comparing)
                live.grabToImage(grab => stage.keep(key, grab), Qt.size(frame.width, frame.height))
        }
    }
    Connections {
        target: stage.Window.window
        function onActiveFocusItemChanged() {
            const f = stage.Window.activeFocusItem
            if (!stage.shown || !stage.behind || !f)
                return
            for (let i = f; i; i = i.parent)
                if (i === stage.behind) {
                    Qt.callLater(() => keys.forceActiveFocus())
                    return
                }
        }
    }
    Connections {
        target: live
        function onLoadedChanged() {
            if (live.loaded) {
                picture.key = stage.snapshotKey
                picture.restart()
            }
        }
    }
    Timer {
        id: settling
        interval: Style.slow + 20
        onTriggered: stage.settled = true
    }
    Timer {
        id: closing
        interval: Style.slow
        onTriggered: {
            stage.shown = false
            stage.state = ""
        }
    }

    states: [
        State {
            name: "card"
            PropertyChanges { frame.x: stage.from.x; frame.y: stage.from.y; frame.width: stage.from.width; frame.height: stage.from.height }
            PropertyChanges { backdrop.opacity: 0; chrome.opacity: 0 }
        },
        State {
            name: "gone"
            PropertyChanges { frame.opacity: 0; backdrop.opacity: 0; chrome.opacity: 0 }
        }
    ]

    // Its own still, blurred once (it doesn't move, so it costs nothing after), behind everything.
    Item {
        id: backdrop
        anchors.fill: parent
        Behavior on opacity { enabled: !stage.placing; NumberAnimation { duration: Style.slow } }
        Rectangle { anchors.fill: parent; color: Style.bgDeep }
        Image {
            id: backImage
            anchors.fill: parent
            anchors.margins: -60
            visible: false
            source: stage.details ? "file://" + stage.details.still : ""
            sourceSize.width: 640
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
        MultiEffect {
            anchors.fill: backImage
            source: backImage
            blurEnabled: true
            blur: 1
            blurMax: 48
            saturation: 0.15
        }
        Rectangle { anchors.fill: parent; color: "black"; opacity: Style.backdropDim }
    }

    Item {
        id: frame
        x: stage.target.x
        y: stage.target.y
        width: stage.target.width
        height: stage.target.height
        Behavior on x { enabled: !stage.placing; NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
        Behavior on y { enabled: !stage.placing; NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
        Behavior on width { enabled: !stage.placing; NumberAnimation { id: widthStep; duration: Style.slow; easing.type: Style.ease } }
        Behavior on height { enabled: !stage.placing; NumberAnimation { id: heightStep; duration: Style.slow; easing.type: Style.ease } }
        Behavior on opacity { NumberAnimation { duration: Style.slow } }

        RectangularShadow {
            anchors.fill: parent
            offset.y: 18
            blur: 60
            radius: 12
            color: Qt.rgba(0, 0, 0, 0.55)
        }

        // Rounded through the layer, not by hiding the theme in a mask source, so it still takes clicks and keys.
        // Off while the frame changes size: a layer resized every frame is what made opening a theme stutter.
        Item {
            id: screen
            anchors.fill: parent
            layer.enabled: stage.settled && !stage.resizing
            layer.effect: MultiEffect {
                maskEnabled: true
                maskSource: screenMask
                maskThresholdMin: 0.5
                maskSpreadAtMin: 1
            }
            Rectangle { anchors.fill: parent; color: "black" }
            Image {
                anchors.fill: parent
                source: stage.details ? "file://" + stage.details.still : ""
                // The card's size, so the image is already decoded and the frame never grows empty.
                sourceSize.width: 640
                fillMode: Image.PreserveAspectCrop
            }
            Image {
                anchors.fill: parent
                // Not while comparing: it is this theme with your changes, which is what Compare looks past.
                visible: stage.snapshot !== "" && !stage.comparing
                source: stage.snapshot
                cache: false
            }
            // The theme as it ships, at full size, for Compare changes; loaded as soon as there is something to compare.
            Image {
                anchors.fill: parent
                visible: stage.comparing
                source: stage.changes > 0 && stage.details ? "file://" + stage.details.still : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
            LivePreview {
                id: live
                anchors.fill: parent
                backend: stage.backend
                themeId: stage.shown && stage.settled ? stage.themeId : ""
                themePath: stage.details ? stage.details.dir : ""
                mode: stage.mode
                fallbackFocus: keys
                opacity: loaded && !stage.comparing ? 1 : 0
                Behavior on opacity { NumberAnimation { duration: stage.comparing ? 120 : Style.medium } }
            }
        }
        Rectangle {
            id: screenMask
            anchors.fill: parent
            radius: 12
            visible: false
            layer.enabled: true
        }
        Rectangle {
            anchors.fill: parent
            radius: 12
            color: "transparent"
            border.color: Qt.rgba(1, 1, 1, 0.06)
        }

        // The first time a theme opens: it is loading, not stuck. After that its picture stands in, with a small note.
        Item {
            anchors.fill: parent
            opacity: stage.shown && !live.loaded && stage.themeId !== "" && stage.snapshot === "" ? 1 : 0
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: Style.medium } }
            Rectangle { anchors.fill: parent; radius: 12; color: "black"; opacity: 0.45 }
            Column {
                anchors.centerIn: parent
                spacing: 14
                Spinner { anchors.horizontalCenter: parent.horizontalCenter; size: 30 }
                Label {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: "Loading " + (stage.details ? stage.details.name : "") + "\u2026"
                    font.family: Style.family
                    font.pixelSize: Style.title
                    font.weight: Font.DemiBold
                    color: "white"
                }
            }
        }
        Pill {
            x: 14
            y: 12
            shown: stage.settled && !live.loaded && stage.themeId !== "" && stage.snapshot !== ""
            busy: true
            text: "Loading\u2026"
        }
        Pill {
            anchors.right: parent.right
            anchors.rightMargin: 12
            y: 12
            shown: stage.comparing
            text: "Without your changes"
        }
        Pill {
            x: 14
            y: 12
            shown: live.usingFallback
            text: "The theme failed to load; this is the fallback prompt"
            warn: true
        }
    }

    // Qt repeats hover events while the theme animates under a still pointer; only a real move counts.
    HoverHandler {
        id: pointer
        property point last
        cursorShape: stage.idle ? Qt.BlankCursor : Qt.ArrowCursor
        onPointChanged: {
            if (point.position === last)
                return
            last = point.position
            idleTimer.poke()
        }
    }
    Timer {
        id: idleTimer
        property bool idle: false
        function poke() {
            idle = false
            restart()
        }
        interval: 2500
        running: stage.shown
        onTriggered: idle = true
    }

    Item {
        id: chrome
        anchors.fill: parent
        Behavior on opacity { NumberAnimation { duration: Style.medium } }

        Item {
            id: top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.rightMargin: stage.inspectorOpen ? stage.inspectorWidth + 24 : 0
            height: 64
            opacity: stage.idle ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: Style.medium } }
            Behavior on anchors.rightMargin { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
            HoverHandler { id: topHover }

            ActionButton {
                id: backButton
                x: 16
                anchors.verticalCenter: parent.verticalCenter
                text: "All themes"
                glyph: "left"
                pill: true
                onActivated: stage.back()
            }
            Column {
                anchors.left: backButton.right
                anchors.leftMargin: 14
                anchors.right: modes.left
                anchors.rightMargin: 14
                anchors.verticalCenter: parent.verticalCenter
                Label {
                    width: parent.width
                    text: (stage.details ? stage.details.name : "") + (stage.backend.dirty ? "  — Edited" : "")
                    font.family: Style.family
                    font.pixelSize: 14
                    font.weight: Font.DemiBold
                    color: "white"
                    elide: Text.ElideRight
                }
                Label {
                    width: parent.width
                    text: stage.details ? "by " + stage.details.author + " · " + stage.details.background + " background"
                                          + (stage.details.isLock ? " · your lockscreen" : "") + (stage.details.isSddm ? " · your login screen" : "") : ""
                    font.family: Style.family
                    font.pixelSize: Style.caption
                    color: Qt.rgba(1, 1, 1, 0.7)
                    elide: Text.ElideRight
                }
            }
            Segmented {
                id: modes
                anchors.centerIn: parent
                options: [{ value: "lock", label: "Lockscreen" }, { value: "sddm", label: "Login screen" }]
                current: stage.mode
                onPicked: v => stage.mode = v
            }
            ActionButton {
                anchors.right: parent.right
                anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                glyph: "sidebar"
                tip: stage.inspectorOpen ? "Hide the settings (Ctrl+I)" : "Show the settings (Ctrl+I)"
                onActivated: stage.inspectorOpen = !stage.inspectorOpen
            }
        }

        // The themes either side of this one, shown while the pointer is near the bottom.
        Rectangle {
            id: strip
            readonly property bool wanted: !stage.idle && (stage.nearBottom || stripHover.hovered) && !menu.opened
            x: (stage.freeWidth - width) / 2
            y: parent.height - 92 - height + (wanted ? 0 : 12)
            width: Math.min(stage.freeWidth - 64, 9 * 100 + 16)
            height: 70
            radius: 14
            color: Style.panel
            border.color: Style.panelBorder
            GlassEdge {}
            opacity: wanted ? 1 : 0
            visible: opacity > 0
            Behavior on opacity { NumberAnimation { duration: Style.fast } }
            Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
            Behavior on x { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
            HoverHandler { id: stripHover }

            ListView {
                id: strips
                anchors.fill: parent
                anchors.margins: 8
                orientation: ListView.Horizontal
                spacing: 8
                clip: true
                model: stage.order
                currentIndex: stage.current
                highlightRangeMode: ListView.ApplyRange
                preferredHighlightBegin: width / 2 - 46
                preferredHighlightEnd: width / 2 + 46
                highlightMoveDuration: Style.medium
                delegate: AbstractButton {
                    id: thumb
                    required property var modelData
                    required property int index
                    width: 92
                    height: 52
                    hoverEnabled: true
                    focusPolicy: Qt.NoFocus
                    onClicked: stage.current = index
                    ToolTip.visible: hovered
                    ToolTip.text: modelData.title
                    ToolTip.delay: 500
                    opacity: index === stage.current || hovered ? 1 : 0.6
                    Behavior on opacity { NumberAnimation { duration: Style.fast } }
                    background: Rounded {
                        radius: 7
                        Image {
                            anchors.fill: parent
                            source: "file://" + thumb.modelData.still
                            sourceSize.width: 184
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                        }
                    }
                    contentItem: Item {}
                    Rectangle {
                        anchors.fill: parent
                        radius: 7
                        color: "transparent"
                        border.width: 2
                        border.color: Style.accent
                        visible: thumb.index === stage.current
                    }
                }
            }
        }

        Item {
            id: dock
            x: (stage.freeWidth - bar.width) / 2
            y: parent.height - 72 + (stage.idle ? 8 : 0)
            opacity: stage.idle ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: Style.medium } }
            Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
            Behavior on x { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }

            Rectangle {
                id: bar
                width: barRow.width + 12
                height: 48
                radius: 24
                color: Style.panel
                border.color: Style.panelBorder
                GlassEdge {}
                HoverHandler { id: barHover }
                Row {
                    id: barRow
                    x: 6
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4
                    ActionButton {
                        id: tryButton
                        text: "Try"
                        glyph: "play"
                        flat: true
                        pill: true
                        onActivated: menu.opened ? menu.close() : menu.open()
                        Popover {
                            id: menu
                            y: -height - 12
                            x: (tryButton.width - width) / 2
                            width: 260
                            transformOrigin: Popup.Bottom
                            Column {
                                width: 248
                                MenuRow {
                                    text: "Full-screen preview"
                                    hint: "Esc leaves"
                                    reason: stage.avail.wayland || stage.busyReason
                                    onChosen: { menu.close(); stage.run(["preview", stage.themeId].concat(stage.mode === "sddm" ? ["--sddm"] : []), "Full-screen preview") }
                                }
                                MenuRow {
                                    text: "Screensaver (this theme)"
                                    reason: stage.avail.wayland || stage.busyReason
                                    onChosen: { menu.close(); stage.run(["preview", stage.themeId, "--saver"], "Screensaver preview") }
                                }
                                MenuRow {
                                    text: "Through SDDM’s own greeter"
                                    reason: stage.avail.sddmPreview || stage.busyReason
                                    onChosen: { menu.close(); stage.run(["sddm", "preview", stage.themeId], "SDDM test mode") }
                                }
                                Rectangle { x: 6; width: parent.width - 12; height: 1; color: Style.sep }
                                MenuRow {
                                    text: "Lock now with this theme"
                                    reason: stage.avail.wayland || stage.busyReason
                                    onChosen: { menu.close(); stage.run(["lock", stage.themeId], "Lock now") }
                                }
                            }
                        }
                    }
                    ActionButton {
                        text: "Check"
                        glyph: "check"
                        flat: true
                        pill: true
                        reason: stage.busyReason
                        tip: "Load it offscreen, look for QML errors and missing fonts, and type the password"
                        onActivated: stage.check()
                    }
                    ActionButton {
                        visible: stage.changes > 0
                        text: "Compare changes"
                        glyph: "compare"
                        flat: true
                        pill: true
                        tip: "Hold to see it as it ships (or hold \\)"
                        onPressedChanged: stage.comparing = pressed
                    }
                    Rectangle { anchors.verticalCenter: parent.verticalCenter; width: 1; height: 20; color: Style.sep }
                    ActionButton {
                        id: useButton
                        text: stage.backend.busy ? "Applying…" : "Use as…"
                        glyph: stage.backend.busy ? "" : "down"
                        primary: true
                        pill: true
                        reason: stage.busyReason
                        onActivated: stage.use(useButton)
                    }
                }
            }
            Toast {
                id: toast
                anchors.left: bar.right
                anchors.leftMargin: 12
                anchors.verticalCenter: bar.verticalCenter
            }
        }
    }
}
