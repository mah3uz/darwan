import QtQuick

// A wheel notch glides a set distance (a card's height, say), where Qt moves a fixed 72 px; notches in quick
// succession add up. It never blocks: Qt still handles every event, so a touchpad scrolls exactly as before, and for a
// notch the flick Qt starts is replaced, just after, by one that covers the distance.
WheelHandler {
    id: wheel

    required property Flickable view
    property real step: 240
    property real owed: 0
    property bool queued: false

    blocking: false
    target: null

    function go() {
        const d = owed
        queued = false
        owed = 0
        view.flick(0, -Math.sign(d) * Math.sqrt(2 * view.flickDeceleration * Math.abs(d)))
    }
    onWheel: event => {
        const dy = event.angleDelta.y
        // A touchpad reports pixels too; a wheel's notches are whole multiples of 120.
        if (dy === 0 || (event.pixelDelta.y !== 0 && dy % 120 !== 0))
            return
        if (!queued) {
            const v = view.moving ? view.verticalVelocity : 0
            owed = v * Math.abs(v) / (2 * view.flickDeceleration)
            queued = true
            Qt.callLater(go)
        }
        owed -= dy / 120 * step
    }
}
