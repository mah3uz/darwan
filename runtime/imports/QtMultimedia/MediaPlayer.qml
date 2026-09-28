import QtQuick

// The slice of Qt's MediaPlayer that themes use. It only holds the state: the VideoOutput it is attached to plays
// the file, through libmpv where Qt renders with OpenGL and through Qt's own player elsewhere.
QtObject {
    id: player

    enum Loops {
        Infinite = -1
    }

    property url source
    property int loops: 1
    property bool autoPlay: false
    property Item videoOutput
    property bool playing: false
    // Qt 6 leaves a url as written; "bg.mp4" means next to the theme file that declared this player.
    readonly property url resolvedSource: source.toString() === "" ? "" : Qt.resolvedUrl(source, player)

    signal errorOccurred(int error, string errorString)

    function play() { playing = true }
    function pause() { playing = false }
    function stop() { playing = false }

    onVideoOutputChanged: if (videoOutput) videoOutput.player = player
    Component.onCompleted: if (autoPlay) playing = true
}
