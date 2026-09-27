import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.darwan

// Asked before anything that would lose the draft or run darwan on the saved file instead of it.
Popup {
    id: dialog

    required property Backend backend
    property string action: ""
    property var proceed: null

    function ask(action, proceed) {
        dialog.action = action
        dialog.proceed = proceed
        open()
    }

    function finish(go) {
        const next = dialog.proceed
        dialog.proceed = null
        close()
        if (go && next)
            next()
    }

    parent: Overlay.overlay
    anchors.centerIn: parent
    width: Math.min(480, parent.width - 64)
    modal: true
    padding: 24
    closePolicy: Popup.CloseOnEscape
    focus: true
    onOpened: saveButton.forceActiveFocus()

    Overlay.modal: Rectangle { color: Qt.alpha(Style.crust, 0.65) }

    background: Rectangle {
        radius: 14
        color: Style.mantle
        border.color: Style.border
    }

    contentItem: ColumnLayout {
        spacing: 16

        Label {
            Layout.fillWidth: true
            text: "Unsaved changes"
            color: Style.text
            font.pixelSize: 18
            font.bold: true
        }
        Label {
            Layout.fillWidth: true
            text: "Save your changes before you " + dialog.action + "? Discarded changes go back to what was saved."
            color: Style.subtext
            wrapMode: Text.WordWrap
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Item { Layout.fillWidth: true }
            ActionButton {
                text: "Cancel"
                onActivated: dialog.finish(false)
            }
            ActionButton {
                text: "Discard"
                onActivated: {
                    dialog.backend.discardChanges()
                    dialog.finish(true)
                }
            }
            ActionButton {
                id: saveButton
                text: "Save"
                primary: true
                onActivated: dialog.finish(dialog.backend.saveChanges())
            }
        }
    }
}
