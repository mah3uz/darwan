import QtQuick

Item {
    id: output

    enum FillMode {
        Stretch = 0,
        PreserveAspectFit = 1,
        PreserveAspectCrop = 2
    }

    property int fillMode: 1
    property QtObject player
    // A hidden video is paused rather than decoded for nobody.
    readonly property bool playing: player !== null && player.playing && visible
    readonly property url source: player ? player.resolvedSource : ""
    readonly property bool loop: player !== null && player.loops < 0

    function fail(reason) {
        if (player)
            player.errorOccurred(1, reason)
    }

    // Items loaded from a file don't see this file's ids, so each backend finds this item as its Loader's parent.
    Loader {
        anchors.fill: parent
        active: output.GraphicsInfo.api !== GraphicsInfo.Unknown
        source: !active ? "" : output.GraphicsInfo.api === GraphicsInfo.OpenGL ? "MpvOutput.qml" : "NativeOutput.qml"
    }
}
