import QtQuick

// A picture with rounded corners, cropped to fill, drawn by one shader straight from the image's texture: no
// offscreen layers, so a grid of them scrolls cheaply. It fades in once the picture is decoded.
Item {
    id: rounded

    property alias source: picture.source
    property alias sourceSize: picture.sourceSize
    property alias status: picture.status
    property real radius: 16
    property bool fade: true

    Image {
        id: picture
        visible: false
        asynchronous: true
        smooth: true
    }

    ShaderEffect {
        id: effect
        anchors.fill: parent
        visible: picture.status === Image.Ready
        opacity: rounded.fade ? 0 : 1
        readonly property Image source: picture
        readonly property size size: Qt.size(width, height)
        readonly property real radius: Math.min(rounded.radius, width / 2, height / 2)
        // Crop to fill: the part of the texture with the item's shape, centred.
        readonly property vector4d uvRect: {
            const iw = picture.implicitWidth, ih = picture.implicitHeight
            if (iw <= 0 || ih <= 0 || width <= 0 || height <= 0)
                return Qt.vector4d(0, 0, 1, 1)
            const scale = Math.max(width / iw, height / ih)
            const sw = width / (iw * scale), sh = height / (ih * scale)
            return Qt.vector4d((1 - sw) / 2, (1 - sh) / 2, sw, sh)
        }
        fragmentShader: "rounded.frag.qsb"
        onVisibleChanged: if (visible && rounded.fade) appear.restart()
        OpacityAnimator {
            id: appear
            target: effect
            from: 0
            to: 1
            duration: 260
            easing.type: Easing.OutCubic
        }
    }
}
