import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// The screensaver, whole: whether hypridle can start it, what happens when you step away, sleep, and video quality.
// Everything here goes into the draft like any setting; idle::panel decides the order, wording and warnings.
Column {
    id: panel

    required property Backend backend
    property bool wide: false
    property bool hintDismissed: false
    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property var p: backend.revision >= 0 ? JSON.parse(backend.saverPanel()) : null

    signal preview()

    spacing: 14

    Row {
        width: parent.width
        Row {
            width: parent.width - previewButton.width
            anchors.verticalCenter: parent.verticalCenter
            spacing: 8
            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 8
                height: 8
                radius: 4
                color: panel.p.ready ? Style.ok : Style.warn
            }
            Label {
                text: panel.p.ready ? "Ready · hypridle is running" : "Not ready yet"
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.text
            }
        }
        ActionButton {
            id: previewButton
            text: "Preview"
            glyph: "play"
            compact: true
            pill: true
            tip: "Your lockscreen’s screensaver, the one hypridle starts"
            onActivated: panel.preview()
        }
    }

    // One setup step at a time, until there is nothing left to set up.
    Rectangle {
        visible: panel.p.setup !== null
        width: panel.wide ? Math.min(parent.width, 760) : parent.width
        height: setupColumn.height + 28
        radius: Style.radius
        color: Qt.tint(Style.group, Qt.alpha(Style.warn, 0.07))
        border.color: Qt.alpha(Style.warn, 0.35)
        Column {
            id: setupColumn
            x: 14
            y: 14
            width: parent.width - 28
            spacing: 8
            Row {
                spacing: 7
                Icon { anchors.verticalCenter: parent.verticalCenter; name: "warn"; size: 15; color: Style.warn }
                Label {
                    text: panel.p.setup ? panel.p.setup.title : ""
                    font.family: Style.family
                    font.pixelSize: Style.body
                    font.weight: Font.DemiBold
                    color: Style.text
                }
            }
            Label {
                width: parent.width
                text: panel.p.setup ? panel.p.setup.text : ""
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
                wrapMode: Text.WordWrap
                lineHeight: 1.2
            }
            Rectangle {
                visible: panel.p.setup !== null && panel.p.setup.code !== ""
                width: parent.width
                height: 34
                radius: 7
                color: Style.bgDeep
                TextEdit {
                    id: code
                    x: 10
                    width: parent.width - copy.width - 24
                    anchors.verticalCenter: parent.verticalCenter
                    text: panel.p.setup ? panel.p.setup.code : ""
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextEdit.WrapAnywhere
                    font.family: Style.mono
                    font.pixelSize: 11
                    color: Style.text
                }
                ActionButton {
                    id: copy
                    anchors.right: parent.right
                    anchors.rightMargin: 4
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Copy"
                    glyph: "copy"
                    compact: true
                    onActivated: {
                        code.selectAll()
                        code.copy()
                        code.deselect()
                        text = "Copied"
                    }
                }
            }
            Row {
                spacing: 8
                Repeater {
                    model: panel.p.setup ? panel.p.setup.actions : []
                    delegate: ActionButton {
                        required property var modelData
                        text: modelData.label
                        primary: modelData.primary
                        compact: true
                        onActivated: panel.backend.saverDo(modelData.id, "")
                    }
                }
            }
        }
    }

    Rectangle {
        visible: panel.p.ready && !panel.hintDismissed
        width: panel.wide ? Math.min(parent.width, 760) : parent.width
        height: hint.height + 20
        radius: Style.radius
        color: Style.group
        Label {
            id: hint
            x: 12
            y: 10
            width: parent.width - gotIt.width - 36
            text: "If DankMaterialShell’s own idle lock is on, turn it off (Settings → Power & Sleep), or both will answer."
            font.family: Style.family
            font.pixelSize: Style.caption
            color: Style.sub
            wrapMode: Text.WordWrap
        }
        ActionButton {
            id: gotIt
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            text: "Got it"
            compact: true
            onActivated: panel.hintDismissed = true
        }
    }

    // Idle, then each thing in the order it happens; a point opens its choices.
    Column {
        visible: panel.p.timeline !== null
        width: parent.width
        spacing: 6
        Row {
            width: parent.width
            Label {
                width: parent.width - saverSwitch.width - onLabel.width - 8
                anchors.verticalCenter: parent.verticalCenter
                leftPadding: 4
                text: "When you step away"
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Label {
                id: onLabel
                anchors.verticalCenter: parent.verticalCenter
                rightPadding: 8
                text: "Screensaver"
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
            }
            Toggle {
                id: saverSwitch
                on: panel.p.timeline !== null && panel.p.timeline.saverOn
                onFlipped: v => panel.backend.saverDo("saverOn", v ? "true" : "false")
            }
        }
        Rectangle {
            width: parent.width
            height: line.height + note.height + 26
            radius: Style.radius
            color: Style.group
            Item {
                id: line
                x: 16
                y: 14
                width: parent.width - 32
                height: 76
                Rectangle { x: 6; y: 11; width: parent.width - 6; height: 2; radius: 1; color: Style.sep }
                Rectangle {
                    readonly property var open: panel.p.timeline ? panel.p.timeline.open : null
                    visible: open !== null
                    x: open ? open.from * line.width : 0
                    width: open ? (open.to - open.from) * line.width : 0
                    y: 9
                    height: 6
                    radius: 3
                    color: Qt.alpha(Style.warn, 0.55)
                    Behavior on x { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
                    Behavior on width { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
                    HoverHandler { id: stretchHover }
                    ToolTip.visible: stretchHover.hovered
                    ToolTip.text: "Until it locks, a key goes back to the desktop without a password"
                }
                Column {
                    spacing: 6
                    Rectangle { y: 6; width: 12; height: 12; radius: 6; color: Style.group; border.width: 2; border.color: Style.muted }
                    Label { text: "Idle"; font.family: Style.family; font.pixelSize: Style.caption; color: Style.sub }
                }
                // One point per step, keyed by id so a change slides it to its new place.
                Repeater {
                    model: ["saver", "lock", "screenOff", "suspend"]
                    delegate: AbstractButton {
                        id: point
                        required property string modelData
                        readonly property var m: panel.p.timeline ? panel.p.timeline.markers.find(x => x.id === modelData) || null : null
                        visible: m !== null
                        x: m ? m.x * line.width - width / 2 : 0
                        width: 76
                        height: 74
                        hoverEnabled: true
                        focusPolicy: Qt.TabFocus
                        onClicked: choices.open()
                        Behavior on x { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
                        contentItem: Item {}
                        Column {
                            anchors.horizontalCenter: parent.horizontalCenter
                            spacing: 3
                            Rectangle {
                                anchors.horizontalCenter: parent.horizontalCenter
                                width: 24
                                height: 24
                                radius: 12
                                color: !point.m ? "transparent" : point.m.warn ? Style.warn : point.m.never ? Style.switchOff : Style.accent
                                border.width: point.visualFocus || choices.visible ? 3 : 0
                                border.color: Style.accentSoft
                                scale: point.hovered ? 1.12 : 1
                                Behavior on scale { NumberAnimation { duration: Style.fast } }
                                Behavior on color { ColorAnimation { duration: Style.medium } }
                                Icon {
                                    anchors.centerIn: parent
                                    name: point.m ? point.m.icon : ""
                                    size: 13
                                    color: point.m && point.m.warn ? "#1a1a1a" : point.m && point.m.never ? Style.sub : Style.accentText
                                }
                            }
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                topPadding: 3
                                text: point.m ? point.m.value : ""
                                font.family: Style.family
                                font.pixelSize: 12
                                font.weight: point.m && point.m.never ? Font.Normal : Font.DemiBold
                                font.features: { "tnum": 1 }
                                color: point.m && point.m.never ? Style.sub : Style.text
                            }
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: point.m ? point.m.name : ""
                                font.family: Style.family
                                font.pixelSize: 11
                                color: Style.sub
                            }
                        }
                        Popover {
                            id: choices
                            y: 30
                            x: (point.width - width) / 2
                            width: 190
                            transformOrigin: Popup.Top
                            Column {
                                width: 178
                                Repeater {
                                    model: point.m ? point.m.choices : []
                                    delegate: MenuRow {
                                        required property var modelData
                                        text: modelData.label
                                        hint: modelData.on ? "✓" : ""
                                        onChosen: {
                                            choices.close()
                                            panel.backend.saverDo(point.modelData, modelData.value)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Label {
                id: note
                x: 16
                anchors.top: line.bottom
                anchors.topMargin: 4
                width: parent.width - 32
                text: panel.p.timeline ? panel.p.timeline.note : ""
                font.family: Style.family
                font.pixelSize: Style.caption
                color: panel.p.timeline && panel.p.timeline.warn ? Style.warn : Style.sub
                wrapMode: Text.WordWrap
            }
        }
    }

    Grid {
        width: parent.width
        columns: panel.wide ? 2 : 1
        columnSpacing: 28
        rowSpacing: 14
        SettingGroup {
            visible: panel.p.sleep !== null
            width: panel.wide ? (parent.width - 28) / 2 : parent.width
            title: "When the machine sleeps"
            foldable: false
            SettingRow {
                width: parent.width
                first: true
                label: "Lock before sleep"
                sub: "Waking shows the password prompt, never the screensaver or your desktop"
                Toggle {
                    on: panel.p.sleep !== null && panel.p.sleep.lockBeforeSleep === true
                    onFlipped: v => panel.backend.saverDo("lockBeforeSleep", v ? "true" : "false")
                }
            }
            SettingRow {
                width: parent.width
                label: "Lock with Darwan"
                sub: panel.p.sleep ? panel.p.sleep.lockWithDarwanNote : ""
                Toggle {
                    on: panel.p.sleep !== null && panel.p.sleep.lockWithDarwan === true
                    onFlipped: v => panel.backend.saverDo("lockWithDarwan", v ? "true" : "false")
                }
            }
        }
        SettingGroup {
            width: panel.wide ? (parent.width - 28) / 2 : parent.width
            title: "When you lock"
            foldable: false
            SettingRow {
                width: parent.width
                first: true
                label: "Screensaver comes back"
                sub: "Once a lock is left untouched, with nothing typed"
                MenuButton {
                    options: panel.p.returnAfters
                    current: panel.p.returnAfter
                    onPicked: v => panel.backend.saverDo("returnAfter", v)
                }
            }
        }
        Column {
            width: panel.wide ? (parent.width - 28) / 2 : parent.width
            spacing: 8
            Label {
                leftPadding: 4
                text: "Video"
                font.family: Style.family
                font.pixelSize: Style.body
                font.weight: Font.DemiBold
                color: Style.text
            }
            Segmented {
                width: parent.width
                stretch: true
                options: panel.p.qualities
                current: panel.p.quality
                onPicked: v => panel.backend.saverDo("quality", v)
            }
            Label {
                width: parent.width
                leftPadding: 4
                text: panel.p.qualityLine
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
                wrapMode: Text.WordWrap
                HoverHandler { id: qualityHover }
                ToolTip.visible: qualityHover.hovered
                ToolTip.text: panel.p.qualityLong
                ToolTip.delay: 500
            }
        }
    }

    Label {
        leftPadding: 4
        text: "Saved to " + panel.p.path + " when you save, and hypridle restarts to pick it up."
        font.family: Style.family
        font.pixelSize: Style.caption
        color: Style.muted
    }
}
