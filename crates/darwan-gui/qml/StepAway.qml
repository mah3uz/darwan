import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// What happens when you step away, in one line under the gates; it opens into the whole screensaver panel in place.
Rectangle {
    id: step

    required property Backend backend
    property bool open: false
    // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;` statement,
    // and the dependency with it, so the value would never refresh.
    readonly property var p: backend.revision >= 0 ? JSON.parse(backend.saverPanel()) : null

    signal preview()

    height: line.height + (open ? panel.height + 18 : 0)
    radius: 12
    color: Style.group
    border.color: p.warn ? Qt.alpha(Style.warn, 0.45) : open || hover.hovered ? Style.muted : Style.sep
    clip: true
    Behavior on height { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
    HoverHandler { id: hover }
    GlassEdge {}

    AbstractButton {
        id: line
        width: parent.width
        height: 46
        focusPolicy: Qt.TabFocus
        onClicked: step.open = !step.open
        contentItem: Item {}
        Icon {
            id: moon
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            name: step.p.warn ? "warn" : "moon"
            size: 16
            color: step.p.warn ? Style.warn : Style.sub
        }
        Label {
            id: heading
            anchors.left: moon.right
            anchors.leftMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            text: "When you step away"
            font.family: Style.family
            font.pixelSize: Style.body
            font.weight: Font.DemiBold
            color: Style.text
        }
        // Takes whatever is left between the heading and the action, which stays on the right edge.
        Label {
            anchors.left: heading.right
            anchors.leftMargin: 10
            anchors.right: action.left
            anchors.rightMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: step.p.summary
            font.family: Style.family
            font.pixelSize: Style.body
            color: Style.sub
            elide: Text.ElideRight
        }
        Label {
            id: action
            anchors.right: parent.right
            anchors.rightMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: step.open ? "Done" : step.p.warn ? "Set up\u2026" : "Change\u2026"
            font.family: Style.family
            font.pixelSize: Style.body
            color: Style.accent
        }
        Rectangle {
            anchors.fill: parent
            radius: 12
            color: "transparent"
            border.width: line.visualFocus ? 2 : 0
            border.color: Style.accent
        }
    }
    Rectangle { y: line.height; width: parent.width; height: 1; color: Style.sep; visible: step.open }

    Loader {
        id: panel
        x: 16
        y: line.height + 14
        width: parent.width - 32
        active: step.open
        opacity: step.open ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Style.medium } }
        sourceComponent: SaverPanel {
            width: panel.width
            backend: step.backend
            wide: true
            onPreview: step.preview()
        }
    }
}
