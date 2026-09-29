import QtQuick

// The light catching the top edge of a glass surface: a hairline that fades out toward the corners.
Rectangle {
    anchors.top: parent.top
    anchors.topMargin: 1
    anchors.horizontalCenter: parent.horizontalCenter
    width: parent.width - Math.max(parent.radius || 0, 8) * 2
    height: 1
    visible: Style.own
    gradient: Gradient {
        orientation: Gradient.Horizontal
        GradientStop { position: 0; color: "transparent" }
        GradientStop { position: 0.5; color: Style.edge }
        GradientStop { position: 1; color: "transparent" }
    }
}
