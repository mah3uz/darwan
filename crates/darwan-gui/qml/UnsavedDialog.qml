import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// Asked before anything that would lose the draft or run darwan on the saved file instead of it.
// A sheet that slides down from the top of the window.
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
    x: (parent.width - width) / 2
    y: 0
    width: Math.min(460, parent.width - 64)
    modal: true
    padding: 22
    closePolicy: Popup.CloseOnEscape
    focus: true
    onOpened: saveButton.forceActiveFocus()

    Overlay.modal: Rectangle { color: Qt.rgba(0, 0, 0, 0.35) }

    enter: Transition {
        NumberAnimation { property: "y"; from: -dialog.height; to: 0; duration: Style.slow; easing.type: Style.ease }
    }
    exit: Transition {
        NumberAnimation { property: "y"; to: -dialog.height; duration: Style.medium; easing.type: Easing.InCubic }
    }

    background: Item {
        Rectangle {
            anchors.fill: parent
            anchors.topMargin: -Style.radius
            radius: Style.radius + 2
            color: Style.popover
            border.color: Style.panelBorder
        }
    }

    contentItem: Column {
        spacing: 16

        Label {
            width: parent.width
            text: "Save your changes before you " + dialog.action + "?"
            font.family: Style.family
            font.pixelSize: Style.title
            font.weight: Font.DemiBold
            color: Style.text
            wrapMode: Text.WordWrap
        }
        Label {
            width: parent.width
            text: "Previews, checks and applying read the saved settings. Discarded changes go back to what was saved."
            font.family: Style.family
            font.pixelSize: Style.body
            color: Style.sub
            wrapMode: Text.WordWrap
        }
        Item {
            width: parent.width
            height: saveButton.height
            ActionButton {
                text: "Discard"
                onActivated: {
                    dialog.backend.discardChanges()
                    dialog.finish(true)
                }
            }
            Row {
                anchors.right: parent.right
                spacing: 8
                ActionButton {
                    text: "Cancel"
                    onActivated: dialog.finish(false)
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
}
