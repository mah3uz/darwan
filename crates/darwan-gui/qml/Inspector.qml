import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// The settings beside the Stage: this theme's, and the ones every theme shares. Changes go into the draft, which the
// preview shows at once and Save keeps.
Rectangle {
    id: inspector

    required property Backend backend
    property string themeId: ""
    property bool open: true
    property string tab: "theme"
    property var details: null

    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property string formJson: backend.revision >= 0 ? (themeId === "" ? "{\"groups\":[],\"changed\":0}" : backend.form(themeId)) : ""
    readonly property var form: JSON.parse(formJson)
    readonly property string globalsJson: backend.revision >= 0 ? backend.globals(themeId) : ""
    readonly property var globals: JSON.parse(globalsJson)
    readonly property var desktop: backend.revision >= 0 ? JSON.parse(backend.desktopWallpaper()) : null
    readonly property var saver: backend.revision >= 0 ? JSON.parse(backend.saverPanel()) : null

    signal pickColour(string key, Item from)
    signal saverSettings(Item from)
    signal importFont(string file)
    signal testSddm()

    function commit(field, value) {
        if (value === "")
            backend.resetValue(field.key)
        else if (value !== field.value)
            backend.setValue(field.key, value)
    }

    width: 380
    radius: 16
    color: Style.panel
    border.color: Style.panelBorder
    GlassEdge {}
    x: open ? parent.width - width - 12 : parent.width + 12
    Behavior on x { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }

    // Keeps clicks and the wheel inside the panel.
    MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons; onWheel: w => w.accepted = false }

    Column {
        id: head
        x: 16
        y: 14
        width: parent.width - 32
        spacing: 12
        Row {
            width: parent.width
            Label {
                width: parent.width - more.width
                anchors.verticalCenter: parent.verticalCenter
                text: inspector.details ? inspector.details.name : ""
                font.family: Style.family
                font.pixelSize: 14
                font.weight: Font.DemiBold
                color: Style.text
                elide: Text.ElideRight
            }
            ActionButton {
                id: more
                glyph: "more"
                flat: true
                compact: true
                tip: "More"
                onActivated: moreMenu.open()
                Popover {
                    id: moreMenu
                    y: more.height + 6
                    x: more.width - width
                    width: 250
                    transformOrigin: Popup.TopRight
                    Column {
                        width: 238
                        MenuRow {
                            text: "Reset " + (inspector.details ? inspector.details.name : "") + "…"
                            reason: inspector.form.changed > 0 ? "" : "every setting is already the theme’s default"
                            onChosen: { moreMenu.close(); inspector.backend.resetTheme(inspector.themeId) }
                        }
                        MenuRow {
                            text: "Show the theme’s folder"
                            onChosen: { moreMenu.close(); Qt.openUrlExternally("file://" + inspector.details.dir) }
                        }
                    }
                }
            }
        }
        Segmented {
            width: parent.width
            stretch: true
            options: [
                { value: "theme", label: "This theme", badge: inspector.form.changed },
                { value: "all", label: "All themes", badge: inspector.globals.changed },
            ]
            current: inspector.tab
            onPicked: v => inspector.tab = v
        }
    }
    Rectangle { y: head.y + head.height + 12; width: parent.width; height: 1; color: Style.sep }

    Flickable {
        id: body
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: head.bottom
        anchors.topMargin: 13
        anchors.bottom: saveBar.top
        contentHeight: content.height + 28
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        ScrollBar.vertical: ScrollBar {}

        Column {
            id: content
            x: 14
            y: 14
            width: body.width - 28
            spacing: 18

            // This theme: its groups, keyed by count so a change updates rows in place and keeps focus.
            Repeater {
                model: inspector.tab === "theme" ? inspector.form.groups.length : 0
                delegate: SettingGroup {
                    id: group
                    required property int index
                    readonly property var info: inspector.form.groups[index]
                    width: content.width
                    title: info.title
                    changed: info.changed
                    notes: info.notes
                    Repeater {
                        model: group.info.fields.length
                        delegate: SettingRow {
                            id: row
                            required property int index
                            readonly property var f: group.info.fields[index]
                            width: content.width
                            first: index === 0
                            label: f.label
                            changed: f.isSet
                            off: f.disabled !== ""
                            stacked: f.control === "media" || f.control === "font"
                            onReset: inspector.backend.resetValue(f.key)
                            FieldControl {
                                width: row.stacked ? row.width - 22 : implicitWidth
                                field: row.f
                                still: inspector.details ? inspector.details.still : ""
                                desktop: inspector.desktop
                                onCommit: v => inspector.commit(row.f, v)
                                onPickColour: from => inspector.pickColour(row.f.key, from)
                            }
                        }
                    }
                    // A licensed font the theme needs but doesn't ship, with where to get it.
                    Repeater {
                        model: group.info.title === "Fonts" && inspector.details ? inspector.details.fonts : []
                        delegate: SettingRow {
                            required property var modelData
                            width: content.width
                            label: modelData.family
                            sub: modelData.license + (modelData.installed ? "" : " · missing, a fallback font is used")
                            Row {
                                spacing: 6
                                Label {
                                    visible: modelData.installed
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: "Installed"
                                    font.family: Style.family
                                    font.pixelSize: Style.caption
                                    color: Style.ok
                                }
                                ActionButton {
                                    visible: !modelData.installed && modelData.url !== ""
                                    text: "Get it"
                                    compact: true
                                    onActivated: Qt.openUrlExternally(modelData.url)
                                }
                                ActionButton {
                                    visible: !modelData.installed
                                    text: "Import…"
                                    compact: true
                                    onActivated: inspector.importFont(modelData.file)
                                }
                            }
                        }
                    }
                }
            }

            // All themes: what every theme shows the same way, then where it is used.
            SettingGroup {
                visible: inspector.tab === "all"
                width: content.width
                title: "Clock and date"
                changed: inspector.globals.changed
                notes: inspector.globals.notes.concat(["Used by every theme that shows a clock or date. Left at the default, each theme keeps its own design."])
                Repeater {
                    model: inspector.tab === "all" ? inspector.globals.fields.length : 0
                    delegate: SettingRow {
                        id: grow
                        required property int index
                        readonly property var f: inspector.globals.fields[index]
                        width: content.width
                        first: index === 0
                        label: f.label
                        changed: f.isSet
                        off: f.disabled !== ""
                        onReset: inspector.backend.resetValue(f.key)
                        FieldControl {
                            field: grow.f
                            onCommit: v => inspector.commit(grow.f, v)
                        }
                    }
                }
            }
            SettingGroup {
                visible: inspector.tab === "all"
                width: content.width
                title: "Screensaver"
                foldable: false
                notes: ["The lockscreen’s background and animation when you step away."]
                SettingRow {
                    width: content.width
                    first: true
                    label: "When you step away"
                    sub: inspector.saver.summary
                    ActionButton {
                        id: saverButton
                        text: "Settings…"
                        compact: true
                        onActivated: inspector.saverSettings(saverButton)
                    }
                }
            }
            SettingGroup {
                visible: inspector.tab === "all"
                width: content.width
                title: "Login screen"
                foldable: false
                SettingRow {
                    width: content.width
                    first: true
                    label: "Test it in SDDM’s own greeter"
                    sub: "Opens a window; your session stays as it is."
                    ActionButton {
                        text: "Open…"
                        compact: true
                        onActivated: inspector.testSddm()
                    }
                }
            }
        }
    }

    // Slides up only while there is something to keep or throw away.
    Rectangle {
        id: saveBar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: inspector.backend.dirty ? 54 : 0
        clip: true
        color: "transparent"
        radius: inspector.radius
        Behavior on height { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        Rectangle { width: parent.width; height: 1; color: Style.sep }
        Row {
            x: 16
            y: 12
            width: parent.width - 32
            spacing: 8
            Label {
                width: parent.width - discard.width - save.width - 16
                anchors.verticalCenter: parent.verticalCenter
                text: "Unsaved changes"
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.sub
            }
            ActionButton {
                id: discard
                text: "Discard"
                onActivated: inspector.backend.discardChanges()
            }
            ActionButton {
                id: save
                text: "Save"
                primary: true
                tip: "Ctrl+S"
                onActivated: inspector.backend.saveChanges()
            }
        }
    }
}
