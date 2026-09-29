import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import org.darwan

// The control model::form chose for a setting; every change comes out as `commit`, empty meaning the default.
Item {
    id: control

    required property var field
    // The theme's still, shown as the background when it has none of its own.
    property string still: ""
    property var desktop: null

    signal commit(string value)
    signal pickColour(Item from)

    implicitWidth: loader.implicitWidth
    implicitHeight: loader.implicitHeight

    Loader {
        id: loader
        width: control.width
        sourceComponent: ({
            switch: toggle,
            looks: looks,
            segmented: segmented,
            menu: menu,
            slider: slider,
            well: well,
            media: media,
            font: font,
            file: file,
        })[control.field.control] || text
    }

    Component {
        id: toggle
        Toggle {
            on: control.field.value === "true"
            onFlipped: v => control.commit(v ? "true" : "false")
        }
    }

    // Light, dark or following the desktop, as small pictures of each.
    Component {
        id: looks
        Row {
            spacing: 10
            Repeater {
                model: control.field.choices
                delegate: AbstractButton {
                    id: tile
                    required property var modelData
                    readonly property bool on: modelData.value === control.field.value
                    readonly property string look: modelData.value === "light" || modelData.value === "dark" ? modelData.value : "auto"
                    width: 58
                    height: 58
                    hoverEnabled: true
                    focusPolicy: Qt.TabFocus
                    onClicked: control.commit(modelData.value)
                    contentItem: Item {}
                    Column {
                        spacing: 5
                        Rectangle {
                            width: 58
                            height: 38
                            radius: 7
                            border.width: tile.on || tile.visualFocus ? 2 : 1
                            border.color: tile.on || tile.visualFocus ? Style.accent : Style.sep
                            gradient: Gradient {
                                orientation: Gradient.Horizontal
                                GradientStop { position: 0; color: tile.look === "dark" ? "#2b2b3a" : "#f5efe6" }
                                GradientStop { position: 0.5; color: tile.look === "dark" ? "#20202a" : tile.look === "auto" ? "#f5efe6" : "#ece2d2" }
                                GradientStop { position: 0.501; color: tile.look === "auto" ? "#1a1a24" : tile.look === "dark" ? "#20202a" : "#ece2d2" }
                                GradientStop { position: 1; color: tile.look === "light" ? "#e6dccb" : "#14141c" }
                            }
                            Rectangle {
                                x: 8
                                y: parent.height - 12
                                width: 30
                                height: 5
                                radius: 3
                                color: tile.look === "light" ? Qt.rgba(0, 0, 0, 0.3) : Qt.rgba(1, 1, 1, 0.35)
                            }
                        }
                        Label {
                            width: 58
                            horizontalAlignment: Text.AlignHCenter
                            text: tile.look === "auto" ? "Auto" : tile.modelData.label
                            font.family: Style.family
                            font.pixelSize: Style.caption
                            font.weight: tile.on ? Font.DemiBold : Font.Normal
                            color: tile.on ? Style.text : Style.sub
                        }
                    }
                }
            }
        }
    }

    Component {
        id: segmented
        Segmented {
            small: true
            options: control.field.choices
            current: control.field.value
            onPicked: v => control.commit(v)
        }
    }

    Component {
        id: menu
        MenuButton {
            options: control.field.choices
            current: control.field.value
            isDefault: !control.field.isSet
            onPicked: v => control.commit(v)
        }
    }

    Component {
        id: slider
        SliderField {
            from: Math.max(control.field.min, -1e9)
            to: Math.min(control.field.max, 1e9)
            step: control.field.step || 1
            value: parseFloat(control.field.value || 0)
            unit: control.field.unit
            onCommitted: v => control.commit(String(v))
        }
    }

    Component {
        id: well
        Row {
            spacing: 10
            Label {
                anchors.verticalCenter: parent.verticalCenter
                readonly property bool generated: control.field.value === "generate"
                readonly property string shown: generated ? (control.field.shown || "") : control.field.value
                text: generated ? "from background" : /^#/.test(shown) ? shown.toLowerCase() : "theme’s own"
                font.family: Style.mono
                font.pixelSize: Style.caption
                color: Style.sub
            }
            ColorWell {
                id: w
                colour: control.field.value === "generate" ? (control.field.shown || "") : control.field.value
                generated: control.field.value === "generate"
                onClicked: control.pickColour(w)
            }
        }
    }

    // What the theme shows behind itself: a thumbnail you can drop a file on, and one menu for every source.
    Component {
        id: media
        Row {
            spacing: 12
            DropArea {
                id: dropTile
                width: 112
                height: 63
                readonly property var kinds: ["png", "jpg", "jpeg", "webp", "bmp", "gif", "mp4", "mkv", "webm", "mov"]
                function pathOf(d) {
                    return d.hasUrls ? decodeURIComponent(d.urls[0].toString().replace(/^file:\/\//, "")) : ""
                }
                onEntered: d => d.accepted = kinds.includes(pathOf(d).split(".").pop().toLowerCase())
                onDropped: d => control.commit(pathOf(d))
                Rounded {
                    anchors.fill: parent
                    radius: 7
                    Rectangle {
                        anchors.fill: parent
                        color: /^#/.test(control.field.value) ? control.field.value : Style.bgDeep
                    }
                    Image {
                        anchors.fill: parent
                        readonly property string path: control.field.value === "" ? control.still
                                                      : control.field.value === "desktop" ? (control.desktop ? control.desktop.path : "")
                                                      : /\.(png|jpe?g|webp|bmp|gif)$/i.test(control.field.value) ? control.field.value : ""
                        source: path === "" ? "" : "file://" + path
                        sourceSize.width: 224
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                    }
                    Icon {
                        anchors.centerIn: parent
                        visible: /\.(mp4|mkv|webm|mov)$/i.test(control.field.value)
                        name: "film"
                        size: 22
                        color: "white"
                    }
                }
                Rectangle {
                    anchors.fill: parent
                    anchors.margins: -2
                    radius: 9
                    color: "transparent"
                    border.width: dropTile.containsDrag ? 2 : 1
                    border.color: dropTile.containsDrag ? Style.accent : Style.sep
                }
            }
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6
                MenuButton {
                    readonly property string source: control.field.value === "" ? "" : control.field.value === "desktop" ? "desktop"
                                                   : /^#/.test(control.field.value) ? "#colour" : "#file"
                    options: [
                        { value: "", label: "The theme’s own" },
                        { value: "desktop", label: control.desktop ? "Desktop wallpaper (" + control.desktop.source + ")" : "Desktop wallpaper" },
                        { value: "#colour", label: "A colour…" },
                        { value: "#file", label: "An image or video…" },
                    ]
                    current: source
                    isDefault: !control.field.isSet
                    onPicked: v => v === "#colour" ? colourDialog.open() : v === "#file" ? mediaPicker.open() : control.commit(v)
                }
                Label {
                    text: "or drop a file on the picture"
                    font.family: Style.family
                    font.pixelSize: Style.caption
                    color: Style.muted
                }
            }
            FileDialog {
                id: mediaPicker
                nameFilters: ["Images and videos (" + (control.field.filters || []).join(" ") + ")"]
                onAccepted: control.commit(decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")))
            }
            ColorDialog {
                id: colourDialog
                onAccepted: control.commit(selectedColor.toString())
            }
        }
    }

    // A family from the list, or a font file; the sample is written in it.
    Component {
        id: font
        Column {
            spacing: 6
            readonly property bool isFile: /\.(ttf|otf)$/i.test(control.field.value)
            Row {
                spacing: 8
                MenuButton {
                    width: 200
                    options: [{ value: "", label: "Theme default" }].concat(Qt.fontFamilies().map(f => ({ value: f, label: f })))
                    current: parent.parent.isFile ? "" : control.field.value
                    isDefault: !control.field.isSet
                    onPicked: v => control.commit(v)
                }
                ActionButton {
                    text: "Font file…"
                    compact: true
                    onActivated: fontPicker.open()
                }
            }
            Label {
                visible: control.field.value !== ""
                text: parent.isFile ? control.field.value.split("/").pop() : "The quick brown fox · 12:34"
                font.family: parent.isFile ? Style.family : control.field.value
                font.pixelSize: parent.isFile ? Style.caption : 16
                color: parent.isFile ? Style.sub : Style.text
                elide: Text.ElideMiddle
                width: 300
            }
            FileDialog {
                id: fontPicker
                nameFilters: ["Fonts (*.ttf *.otf)"]
                onAccepted: control.commit(decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")))
            }
        }
    }

    Component {
        id: file
        Row {
            spacing: 8
            TextField {
                width: 170
                height: 26
                text: control.field.value
                color: Style.text
                font.family: Style.family
                font.pixelSize: Style.body
                onEditingFinished: control.commit(text)
                background: Rectangle { radius: Style.small; color: Style.control; border.width: parent.activeFocus ? 2 : 0; border.color: Style.accentSoft }
            }
            ActionButton {
                text: "Browse…"
                compact: true
                onActivated: picker.open()
            }
            FileDialog {
                id: picker
                nameFilters: [control.field.label + " (" + (control.field.filters || []).map(f => "*" + f).join(" ") + ")"]
                onAccepted: control.commit(decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")))
            }
        }
    }

    Component {
        id: text
        TextField {
            width: 190
            height: 26
            text: control.field.value
            placeholderText: "Theme default"
            placeholderTextColor: Style.muted
            color: Style.text
            font.family: Style.family
            font.pixelSize: Style.body
            onEditingFinished: control.commit(text)
            background: Rectangle { radius: Style.small; color: Style.control; border.width: parent.activeFocus ? 2 : 0; border.color: Style.accentSoft }
        }
    }
}
