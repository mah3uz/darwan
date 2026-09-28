import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import org.darwan

Rectangle {
    id: form

    required property Backend backend
    property string themeId: ""
    property string themeName: ""
    readonly property var fields: {
        backend.revision
        return themeId === "" ? [] : JSON.parse(backend.fields(themeId))
    }

    color: Style.mantle

    function commit(field, text) {
        if (text === field.value)
            return
        if (text === "" && ["text", "media", "font", "color"].includes(field.kind))
            backend.resetValue(field.key)
        else
            backend.setValue(field.key, text)
    }

    ColorPopover {
        id: colorPopover
        fields: form.fields
        x: (form.width - width) / 2
        y: Style.gap * 4
        onChosen: (field, value) => form.commit(field, value)
    }

    function fileUrlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""))
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Style.gap
        spacing: Style.gap

        RowLayout {
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true
                text: "Settings"
                color: Style.accent
                font.bold: true
                font.pixelSize: 16
            }
            ActionButton {
                text: "Reset theme"
                reason: form.fields.some(f => f.isSet && f.key.startsWith(form.themeId + "."))
                        ? "" : "every setting is already the theme's default"
                ToolTip.visible: hovered
                ToolTip.text: available ? "Every option and customisation of " + form.themeName + " back to its default; Discard undoes it until you save" : reason
                onActivated: form.backend.resetTheme(form.themeId)
            }
        }
        Label {
            Layout.fillWidth: true
            text: form.themeName + " options first, then its customisations and the global settings. The preview shows changes at once; Save (Ctrl+S) keeps them. Drop an image or video on the preview to use it as the background."
            color: Style.muted
            wrapMode: Text.WordWrap
            font.pixelSize: 12
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: 18

                // Rows are keyed by position, so saving a value updates them in place and keeps focus.
                Repeater {
                    model: form.fields.length

                    delegate: ColumnLayout {
                        id: row
                        required property int index
                        readonly property var field: form.fields[index]
                        readonly property bool off: field.disabled !== ""
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            visible: row.field.group !== "" && (row.index === 0 || form.fields[row.index - 1].group !== row.field.group)
                            Layout.topMargin: row.index === 0 ? 0 : 10
                            text: row.field.group.toUpperCase()
                            color: Style.accent
                            font.bold: true
                            font.pixelSize: 12
                            font.letterSpacing: 1.5
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Label {
                                Layout.fillWidth: true
                                text: row.field.label
                                color: row.off ? Style.muted : Style.text
                                font.bold: true
                            }
                            Label {
                                visible: row.field.isSet
                                text: "reset"
                                color: resetHover.hovered ? Style.accent : Style.subtext
                                font.underline: resetHover.hovered
                                font.pixelSize: 12
                                HoverHandler { id: resetHover; cursorShape: Qt.PointingHandCursor }
                                TapHandler { onTapped: form.backend.resetValue(row.field.key) }
                            }
                        }

                        Loader {
                            Layout.fillWidth: true
                            enabled: !row.off
                            opacity: row.off ? 0.45 : 1
                            sourceComponent: ({
                                bool: boolEditor,
                                choice: choiceEditor,
                                int: intEditor,
                                range: rangeEditor,
                                color: colorEditor,
                                file: fileEditor,
                                media: mediaEditor,
                                font: fontEditor,
                                text: textEditor,
                            })[row.field.kind] || textEditor

                            Component {
                                id: boolEditor
                                Switch {
                                    checked: row.field.value === "true"
                                    text: checked ? "On" : "Off"
                                    onToggled: form.commit(row.field, checked ? "true" : "false")
                                }
                            }

                            Component {
                                id: choiceEditor
                                ComboBox {
                                    model: row.field.choices
                                    textRole: "label"
                                    valueRole: "value"
                                    currentIndex: row.field.choices.findIndex(c => c.value === row.field.value)
                                    onActivated: form.commit(row.field, currentValue)
                                }
                            }

                            Component {
                                id: intEditor
                                SpinBox {
                                    from: Math.max(row.field.min, -2147483648)
                                    to: Math.min(row.field.max, 2147483647)
                                    value: parseInt(row.field.value)
                                    editable: true
                                    onValueModified: form.commit(row.field, String(value))
                                }
                            }

                            Component {
                                id: rangeEditor
                                RowLayout {
                                    spacing: 10
                                    Slider {
                                        id: slider
                                        Layout.fillWidth: true
                                        from: row.field.min
                                        to: row.field.max
                                        stepSize: row.field.step
                                        snapMode: Slider.SnapAlways
                                        value: parseFloat(row.field.value)
                                        // Saved on release, so dragging doesn't reload the preview at every step.
                                        onPressedChanged: if (!pressed) form.commit(row.field, String(Math.round(value * 100) / 100))
                                    }
                                    Label {
                                        text: (Math.round(slider.value * 100) / 100) + (row.field.key.endsWith(".motion_speed") ? "×" : "")
                                        color: Style.text
                                        Layout.preferredWidth: 44
                                    }
                                }
                            }

                            Component {
                                id: colorEditor
                                // The colour the preview uses and where it comes from; the popover changes it.
                                Button {
                                    id: chip
                                    readonly property bool generated: row.field.value === "generate"
                                    readonly property string shown: generated ? (row.field.shown || "") : row.field.value
                                    readonly property bool valid: /^#[0-9a-fA-F]{3,8}$/.test(shown)
                                    hoverEnabled: true
                                    onClicked: colorPopover.openFor(row.field.key)
                                    background: Rectangle {
                                        implicitHeight: 40
                                        radius: 6
                                        color: chip.hovered ? Qt.lighter(Style.surface, 1.2) : Style.surface
                                        border.color: chip.visualFocus ? Style.accent : "transparent"
                                        border.width: 2
                                    }
                                    contentItem: RowLayout {
                                        spacing: 10
                                        Rectangle {
                                            Layout.preferredWidth: 26
                                            Layout.preferredHeight: 26
                                            radius: 6
                                            color: chip.valid ? chip.shown : "transparent"
                                            border.color: Style.border
                                        }
                                        Label {
                                            text: chip.valid ? chip.shown.toLowerCase()
                                                : chip.generated ? "no image to generate from" : "the theme's own colours"
                                            color: Style.text
                                            font.family: chip.valid ? "monospace" : Qt.application.font.family
                                        }
                                        Item { Layout.fillWidth: true }
                                        // The form already says "theme default" and offers "reset"; only the source needs naming.
                                        Label {
                                            visible: chip.generated
                                            text: "from your background"
                                            color: Style.muted
                                            font.pixelSize: 12
                                        }
                                    }
                                }
                            }

                            Component {
                                id: mediaEditor
                                ColumnLayout {
                                    id: mediaBox
                                    spacing: 6
                                    readonly property var desktop: JSON.parse(form.backend.desktopWallpaper())
                                    RowLayout {
                                        spacing: 8
                                        TextField {
                                            Layout.fillWidth: true
                                            text: row.field.value === "desktop" ? "" : row.field.value
                                            placeholderText: row.field.value === "desktop" ? "the desktop wallpaper" : "the theme's own background"
                                            onEditingFinished: if (text !== "") form.commit(row.field, text)
                                        }
                                        ActionButton {
                                            text: "Browse…"
                                            onActivated: mediaPicker.open()
                                        }
                                    }
                                    RowLayout {
                                        spacing: 8
                                        ActionButton {
                                            text: mediaBox.desktop ? "Desktop wallpaper (" + mediaBox.desktop.source + ")" : "No desktop wallpaper found"
                                            enabled: mediaBox.desktop !== null
                                            onActivated: form.commit(row.field, "desktop")
                                        }
                                        ActionButton {
                                            text: "Colour…"
                                            onActivated: backgroundColour.open()
                                        }
                                    }
                                    FileDialog {
                                        id: mediaPicker
                                        nameFilters: ["Images and videos (" + row.field.filters.join(" ") + ")"]
                                        onAccepted: form.commit(row.field, form.fileUrlToPath(selectedFile))
                                    }
                                    ColorDialog {
                                        id: backgroundColour
                                        onAccepted: form.commit(row.field, selectedColor.toString())
                                    }
                                }
                            }

                            Component {
                                id: fontEditor
                                ColumnLayout {
                                    id: fontBox
                                    spacing: 6
                                    readonly property bool isFile: /\.(ttf|otf)$/i.test(row.field.value)
                                    RowLayout {
                                        spacing: 8
                                        ComboBox {
                                            id: family
                                            Layout.fillWidth: true
                                            editable: true
                                            model: Qt.fontFamilies()
                                            currentIndex: model.indexOf(row.field.value)
                                            editText: fontBox.isFile ? "" : row.field.value
                                            onAccepted: form.commit(row.field, editText)
                                            onActivated: form.commit(row.field, currentText)
                                        }
                                        ActionButton {
                                            text: "Font file…"
                                            onActivated: fontPicker.open()
                                        }
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        visible: row.field.value !== "" && !fontBox.isFile
                                        text: "The quick brown fox · 12:34"
                                        font.family: row.field.value
                                        font.pixelSize: 16
                                        color: Style.text
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        visible: fontBox.isFile
                                        text: row.field.value.split("/").pop()
                                        color: Style.muted
                                        elide: Text.ElideMiddle
                                    }
                                    FileDialog {
                                        id: fontPicker
                                        nameFilters: ["Fonts (*.ttf *.otf)"]
                                        onAccepted: form.commit(row.field, form.fileUrlToPath(selectedFile))
                                    }
                                }
                            }

                            Component {
                                id: textEditor
                                RowLayout {
                                    spacing: 8
                                    Rectangle {
                                        visible: row.field.kind === "color"
                                        Layout.preferredWidth: 28
                                        Layout.preferredHeight: 28
                                        radius: 4
                                        color: /^#[0-9a-fA-F]{3,8}$/.test(input.text) ? input.text : "transparent"
                                        border.color: Style.border
                                    }
                                    TextField {
                                        id: input
                                        Layout.fillWidth: true
                                        text: row.field.value
                                        placeholderText: row.field.kind === "text" ? "theme default" : ""
                                        onEditingFinished: form.commit(row.field, text)
                                    }
                                }
                            }

                            Component {
                                id: fileEditor
                                RowLayout {
                                    spacing: 8
                                    TextField {
                                        id: path
                                        Layout.fillWidth: true
                                        text: row.field.value
                                        onEditingFinished: form.commit(row.field, text)
                                    }
                                    ActionButton {
                                        text: "Browse…"
                                        onActivated: picker.open()
                                    }
                                    FileDialog {
                                        id: picker
                                        nameFilters: [row.field.label + " (" + row.field.filters.map(f => "*" + f).join(" ") + ")"]
                                        onAccepted: form.commit(row.field, decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")))
                                    }
                                }
                            }
                        }

                        Label {
                            Layout.fillWidth: true
                            visible: row.off || !row.field.isSet
                            text: row.off ? row.field.disabled : "theme default"
                            color: row.off ? Style.warn : Style.muted
                            font.pixelSize: 12
                            font.italic: !row.off
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Label {
                Layout.fillWidth: true
                text: form.backend.dirty ? "Unsaved changes" : "All changes saved"
                color: form.backend.dirty ? Style.warn : Style.muted
                font.pixelSize: 12
            }
            ActionButton {
                text: "Discard"
                reason: form.backend.dirty ? "" : "nothing to discard"
                onActivated: form.backend.discardChanges()
            }
            ActionButton {
                text: "Save"
                primary: true
                reason: form.backend.dirty ? "" : "nothing to save"
                onActivated: form.backend.saveChanges()
            }
        }
    }
}
