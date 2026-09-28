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

    // Chosen once, when the item first has a window: moving to another window (the saver becoming the lock) passes
    // through none, and reloading there would restart the video.
    property string backend: ""
    function choose() {
        if (backend === "" && GraphicsInfo.api !== GraphicsInfo.Unknown)
            backend = GraphicsInfo.api === GraphicsInfo.OpenGL ? "MpvOutput.qml" : "NativeOutput.qml"
    }
    Component.onCompleted: choose()
    Connections {
        target: output.GraphicsInfo
        function onApiChanged() { output.choose() }
    }

    // Items loaded from a file don't see this file's ids, so each backend finds this item as its Loader's parent.
    // Without darwan's plugin the mpv backend can't load; Qt's player still plays the video.
    Loader {
        anchors.fill: parent
        source: output.backend
        onStatusChanged: if (status === Loader.Error && output.backend === "MpvOutput.qml") Qt.callLater(() => output.backend = "NativeOutput.qml")
    }
}
