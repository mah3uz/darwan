import QtQuick
import QtQuick.Shapes

// An arc that turns on the render thread, so it keeps turning while the interface is busy building something.
Shape {
    id: spinner

    property real size: 16
    property color color: "white"

    width: size
    height: size
    preferredRendererType: Shape.CurveRenderer

    ShapePath {
        strokeColor: spinner.color
        strokeWidth: Math.max(2, spinner.size / 10)
        fillColor: "transparent"
        capStyle: ShapePath.RoundCap
        PathAngleArc {
            centerX: spinner.size / 2
            centerY: spinner.size / 2
            radiusX: spinner.size / 2 - 2
            radiusY: spinner.size / 2 - 2
            startAngle: 0
            sweepAngle: 270
        }
    }
    RotationAnimator on rotation {
        running: spinner.visible
        loops: Animation.Infinite
        from: 0
        to: 360
        duration: 800
    }
}
