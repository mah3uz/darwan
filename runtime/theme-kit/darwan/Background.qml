import QtQuick
import QtMultimedia

// The user's own background (image, animated image, video or colour), with fit and dim. The theme keeps
// its own background and hides it while `active`; a file that fails to load leaves `active` false.
Item {
    id: bg

    readonly property var cfg: typeof config !== "undefined" ? config : ({})
    readonly property string kind: cfg.backgroundType || ""
    readonly property string path: cfg.backgroundPath ? "file://" + cfg.backgroundPath : ""
    readonly property bool cover: cfg.backgroundFit !== "contain"
    readonly property real dim: Math.min(80, Math.max(0, parseInt(cfg.backgroundDim) || 0)) / 100
    property bool failed: false
    readonly property bool active: kind !== "" && !failed

    visible: active

    Rectangle {
        anchors.fill: parent
        color: bg.kind === "color" ? (bg.cfg.backgroundColor || "black") : "black"
    }

    Loader {
        anchors.fill: parent
        active: bg.kind === "image" || bg.kind === "animated"
        sourceComponent: bg.kind === "animated" ? animated : still
    }

    Component {
        id: still
        Image {
            source: bg.path
            asynchronous: true
            cache: false
            fillMode: bg.cover ? Image.PreserveAspectCrop : Image.PreserveAspectFit
            sourceSize: Qt.size(Screen.width, Screen.height)
            onStatusChanged: if (status === Image.Error) bg.failed = true
        }
    }

    Component {
        id: animated
        AnimatedImage {
            source: bg.path
            cache: false
            fillMode: bg.cover ? Image.PreserveAspectCrop : Image.PreserveAspectFit
            onStatusChanged: if (status === Image.Error) bg.failed = true
        }
    }

    Loader {
        anchors.fill: parent
        active: bg.kind === "video"
        sourceComponent: Item {
            MediaPlayer {
                id: player
                source: bg.path
                loops: MediaPlayer.Infinite
                videoOutput: output
                onErrorOccurred: bg.failed = true
                Component.onCompleted: play()
            }
            VideoOutput {
                id: output
                anchors.fill: parent
                fillMode: bg.cover ? VideoOutput.PreserveAspectCrop : VideoOutput.PreserveAspectFit
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: "black"
        opacity: bg.dim
        visible: bg.dim > 0
    }
}
