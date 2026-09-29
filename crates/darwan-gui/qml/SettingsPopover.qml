import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Settings that aren't one theme's: the clock and date every theme shares, and the screensaver.
Popover {
    id: pop

    required property Backend backend
    property string themeId: ""
    property string tab: "general"
    property Item from: null

    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property var globals: backend.revision >= 0 ? JSON.parse(backend.globals(themeId)) : null

    signal preview()

    // Under the button that opened it, lined up with its right edge, or beside the settings panel.
    function openFrom(item, which) {
        tab = which || "general"
        from = item
        parent = item
        const beside = item.mapToItem(null, 0, 0).x > Overlay.overlay.width * 0.6 && which === "saver"
        x = beside ? -width - 16 : item.width - width
        y = beside ? (item.height - height) / 2 : item.height + 8
        transformOrigin = beside ? Popup.Right : Popup.TopRight
        open()
    }

    width: 440
    height: Math.min(Overlay.overlay ? Overlay.overlay.height - 80 : 700, head.height + body.contentHeight + saveBar.height + 30)
    padding: 0

    Column {
        id: head
        x: 16
        y: 14
        width: pop.width - 32
        spacing: 12
        Label {
            text: "Settings"
            font.family: Style.family
            font.pixelSize: 14
            font.weight: Font.DemiBold
            color: Style.text
        }
        Segmented {
            width: parent.width
            stretch: true
            options: [
                { value: "general", label: "General", badge: pop.globals.changed },
                { value: "saver", label: "Screensaver" },
            ]
            current: pop.tab
            onPicked: v => pop.tab = v
        }
    }
    Rectangle { y: head.y + head.height + 12; width: pop.width; height: 1; color: Style.sep }

    Flickable {
        id: body
        y: head.y + head.height + 13
        width: pop.width
        height: pop.height - y - saveBar.height
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

            SettingGroup {
                visible: pop.tab === "general"
                width: content.width
                title: "Clock and date"
                changed: pop.globals.changed
                notes: pop.globals.notes.concat(["Used by every theme that shows a clock or date. Left at the default, each theme keeps its own design."])
                Repeater {
                    model: pop.tab === "general" ? pop.globals.fields.length : 0
                    delegate: SettingRow {
                        id: row
                        required property int index
                        readonly property var f: pop.globals.fields[index]
                        width: content.width
                        first: index === 0
                        label: f.label
                        changed: f.isSet
                        off: f.disabled !== ""
                        onReset: pop.backend.resetValue(f.key)
                        FieldControl {
                            field: row.f
                            onCommit: v => v === "" ? pop.backend.resetValue(row.f.key) : pop.backend.setValue(row.f.key, v)
                        }
                    }
                }
            }
            Loader {
                active: pop.tab === "saver" && pop.visible
                width: content.width
                sourceComponent: SaverPanel {
                    width: content.width
                    backend: pop.backend
                    onPreview: pop.preview()
                }
            }
        }
    }

    Rectangle {
        id: saveBar
        anchors.bottom: parent.bottom
        width: pop.width
        height: pop.backend.dirty ? 54 : 0
        clip: true
        color: "transparent"
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
            ActionButton { id: discard; text: "Discard"; onActivated: pop.backend.discardChanges() }
            ActionButton { id: save; text: "Save"; primary: true; onActivated: pop.backend.saveChanges() }
        }
    }
}
