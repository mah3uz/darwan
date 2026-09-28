import QtQuick
import QtMultimedia 6.0 as Native

Item {
    id: native

    readonly property Item output: parent ? parent.parent : null

    function sync() {
        if (output && output.playing)
            player.play()
        else
            player.pause()
    }

    Native.MediaPlayer {
        id: player
        source: native.output ? native.output.source : ""
        loops: native.output && native.output.loop ? Native.MediaPlayer.Infinite : 1
        videoOutput: video
        onErrorOccurred: (error, errorString) => native.output.fail(errorString)
    }

    Native.VideoOutput {
        id: video
        anchors.fill: parent
        fillMode: native.output ? native.output.fillMode : Native.VideoOutput.PreserveAspectFit
    }

    Connections {
        target: native.output
        function onPlayingChanged() { native.sync() }
    }

    Component.onCompleted: sync()
}
