import QtQuick

// A picture that changes by crossfading: the next one decodes behind the current and fades in only once it's ready, so
// there is never an empty frame between them.
Item {
    id: fade

    property url source
    property int fillMode: Image.PreserveAspectCrop
    property int duration: 380
    property Image front: one
    readonly property Image back: front === one ? two : one
    readonly property bool ready: String(front.source) === String(source) && front.status === Image.Ready

    function settle() {
        back.opacity = 0
        back.source = ""
    }
    function reveal() {
        back.z = 1
        front.z = 0
        front = back
        fadeIn.target = front
        fadeIn.restart()
    }
    onSourceChanged: {
        fadeIn.complete()
        settle()
        if (String(source) === "") {
            front.source = ""
            return
        }
        if (String(source) !== String(front.source))
            back.source = source
    }

    Image {
        id: one
        anchors.fill: parent
        fillMode: fade.fillMode
        asynchronous: true
        opacity: 0
        onStatusChanged: if (one === fade.back && status === Image.Ready && String(source) === String(fade.source)) fade.reveal()
    }
    Image {
        id: two
        anchors.fill: parent
        fillMode: fade.fillMode
        asynchronous: true
        opacity: 0
        onStatusChanged: if (two === fade.back && status === Image.Ready && String(source) === String(fade.source)) fade.reveal()
    }
    NumberAnimation {
        id: fadeIn
        property: "opacity"
        from: 0
        to: 1
        duration: fade.duration
        easing.type: Easing.InOutQuad
        onFinished: fade.settle()
    }
}
