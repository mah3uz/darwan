import QtQuick
import QtQuick.Controls.Basic

// One notice at a time, beside the action bar: what is happening, then how it went. A new one replaces the last.
// Confirmations fade; a problem stays until it is closed or replaced, and offers the details.
Rectangle {
    id: toast

    property string text: ""
    property string image: ""
    // 0..1, or below 0 while there is no telling how long.
    property real progress: -1
    // "", "done" or "fail"
    property string kind: ""
    property bool shown: false

    signal details()

    function show(text, image, progress, kind) {
        toast.text = text
        toast.image = image
        toast.progress = progress
        toast.kind = kind
        toast.shown = true
    }

    implicitWidth: 300
    implicitHeight: 50
    radius: height / 2
    color: Style.panel
    border.color: Style.panelBorder
    GlassEdge {}
    opacity: shown ? 1 : 0
    visible: opacity > 0
    transform: Translate { x: toast.shown ? 0 : -10 }
    Behavior on opacity { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }

    Timer {
        running: toast.shown && toast.kind === "done"
        interval: 2400
        onTriggered: toast.shown = false
    }

    Rounded {
        id: thumb
        x: 8
        anchors.verticalCenter: parent.verticalCenter
        width: 34
        height: 34
        radius: 17
        Image {
            anchors.fill: parent
            source: toast.image
            sourceSize.width: 68
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
        }
    }
    Column {
        anchors.left: thumb.right
        anchors.leftMargin: 10
        anchors.right: actions.left
        anchors.rightMargin: 10
        anchors.verticalCenter: parent.verticalCenter
        spacing: 6
        Label {
            width: parent.width
            text: toast.text
            font.family: Style.family
            font.pixelSize: Style.body
            font.weight: Font.DemiBold
            color: toast.kind === "fail" ? Style.warn : Style.text
            elide: Text.ElideRight
        }
        Rectangle {
            width: parent.width
            height: 3
            radius: 2
            color: Style.sep
            clip: true
            Rectangle {
                readonly property bool waiting: toast.progress < 0 && toast.kind === ""
                visible: !waiting
                height: parent.height
                radius: 2
                width: parent.width * (toast.kind === "" ? Math.max(0, Math.min(1, toast.progress)) : 1)
                color: toast.kind === "done" ? Style.ok : toast.kind === "fail" ? Style.warn : Style.text
                Behavior on width { NumberAnimation { duration: Style.slow; easing.type: Style.ease } }
            }
            Rectangle {
                id: sweep
                visible: toast.progress < 0 && toast.kind === ""
                height: parent.height
                width: parent.width * 0.3
                radius: 2
                color: Style.text
                NumberAnimation on x {
                    running: sweep.visible && toast.shown
                    loops: Animation.Infinite
                    from: -sweep.width
                    to: sweep.parent.width
                    duration: 1100
                    easing.type: Easing.InOutQuad
                }
            }
        }
    }
    Row {
        id: actions
        anchors.right: parent.right
        anchors.rightMargin: 8
        anchors.verticalCenter: parent.verticalCenter
        spacing: 4
        ActionButton {
            visible: toast.kind === "fail"
            text: "Details"
            compact: true
            pill: true
            onActivated: toast.details()
        }
        ActionButton {
            visible: toast.kind === "fail"
            glyph: "close"
            flat: true
            compact: true
            tip: "Dismiss"
            onActivated: toast.shown = false
        }
    }
}
