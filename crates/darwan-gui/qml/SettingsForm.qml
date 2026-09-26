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
        if (text === "" && field.kind === "text")
            backend.resetValue(field.key)
        else
            backend.setValue(field.key, text)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Style.gap
        spacing: Style.gap

        Label {
            text: "Settings"
            color: Style.accent
            font.bold: true
            font.pixelSize: 16
        }
        Label {
            Layout.fillWidth: true
            text: form.themeName + " options first, then the global ones. Changes are saved at once."
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
                                color: textEditor,
                                file: fileEditor,
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
    }
}
