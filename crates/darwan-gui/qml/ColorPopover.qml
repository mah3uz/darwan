import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Shapes

// One place to choose a colour: the theme's own, generated from the background, or a custom one.
// Every choice reaches the preview at once; Cancel puts back what was there when it opened.
Popup {
    id: pop

    property var fields: []
    property string key: ""
    readonly property var field: fields.find(f => f.key === key) || null
    property string original: ""
    readonly property string mode: field === null ? "theme"
                                 : field.value === "generate" ? "image"
                                 : field.isSet ? "custom" : "theme"

    signal chosen(var field, string value)

    property real hue: 0
    property real sat: 0
    property real val: 1
    readonly property color current: Qt.hsva(hue, sat, val, 1)
    readonly property string hex: "#" + [current.r, current.g, current.b]
        .map(c => Math.round(c * 255).toString(16).padStart(2, "0")).join("")

    function normalised(text) {
        const t = text.trim().replace(/^#?/, "#").toLowerCase()
        return /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/.test(t) ? t : ""
    }

    // Moves the wheel to a colour without choosing it.
    function show(text) {
        const t = normalised(text)
        if (t === "")
            return false
        const c = Qt.color(t)
        hue = Math.max(c.hsvHue, 0)
        sat = c.hsvSaturation
        val = c.hsvValue
        brightness.value = val
        return true
    }

    function choose(value) {
        if (field !== null && value !== field.value)
            chosen(field, value)
    }

    function chooseCode(text) {
        if (show(text))
            choose(normalised(text))
    }

    function cancel() {
        choose(original)
        close()
    }

    function openFor(k) {
        key = k
        original = field.value
        const start = [field.shown || "", field.value, (field.imageColours[0] || {}).hex || "", field.swatches[0] || ""]
            .find(c => normalised(c) !== "")
        show(start || "#ffffff")
        open()
    }

    width: 340
    padding: Style.gap
    // A click outside keeps the colour, like Done; Escape puts it back, like Cancel.
    closePolicy: Popup.CloseOnPressOutside
    Shortcut {
        sequence: "Escape"
        enabled: pop.visible
        onActivated: pop.cancel()
    }
    background: Rectangle {
        color: Style.base
        radius: Style.radius
        border.color: Style.border
    }

    component Segment: Rectangle {
        id: seg
        property string label
        property bool active
        signal picked()
        Layout.fillWidth: true
        implicitHeight: 30
        radius: 6
        color: active ? Style.accent : segHover.hovered ? Qt.lighter(Style.surface, 1.2) : Style.surface
        Label {
            anchors.centerIn: parent
            text: seg.label
            color: seg.active ? Style.crust : Style.text
            font.pixelSize: 12
        }
        HoverHandler { id: segHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: seg.picked() }
    }

    component Swatch: Rectangle {
        id: sw
        property string hex
        property string tip
        width: 22
        height: 22
        radius: 11
        color: hex
        border.color: pop.mode === "custom" && pop.field.value === hex ? Style.text : Style.border
        border.width: pop.mode === "custom" && pop.field.value === hex ? 2 : 1
        ToolTip.visible: swHover.hovered
        ToolTip.text: (tip ? tip + " · " : "") + hex
        ToolTip.delay: 300
        HoverHandler { id: swHover; cursorShape: Qt.PointingHandCursor }
        TapHandler { onTapped: pop.chooseCode(sw.hex) }
    }

    contentItem: ColumnLayout {
        spacing: 12

        Label {
            text: pop.field ? pop.field.label : ""
            color: Style.text
            font.bold: true
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 4
            Segment {
                label: "Theme"
                active: pop.mode === "theme"
                onPicked: pop.choose("")
            }
            Segment {
                visible: pop.field !== null && pop.field.generate
                label: "From background"
                active: pop.mode === "image"
                onPicked: pop.choose("generate")
            }
            // Starts from the colour showing now, so a generated colour can be tweaked.
            Segment {
                label: "Custom"
                active: pop.mode === "custom"
                onPicked: pop.choose(pop.hex)
            }
        }

        Item {
            id: disc
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: 196
            implicitHeight: 196
            readonly property real r: width / 2

            function pick(px, py) {
                const dx = px - r, dy = r - py
                pop.hue = (Math.atan2(dy, dx) / (2 * Math.PI) + 1) % 1
                pop.sat = Math.min(Math.hypot(dx, dy) / r, 1)
            }

            // Hue around the circle, counter-clockwise from 3 o'clock; white over it fades out towards
            // the rim, which is exactly HSV saturation at full value.
            Shape {
                anchors.fill: parent
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeWidth: -1
                    fillGradient: ConicalGradient {
                        centerX: disc.r
                        centerY: disc.r
                        GradientStop { position: 0 / 6; color: "#ff0000" }
                        GradientStop { position: 1 / 6; color: "#ffff00" }
                        GradientStop { position: 2 / 6; color: "#00ff00" }
                        GradientStop { position: 3 / 6; color: "#00ffff" }
                        GradientStop { position: 4 / 6; color: "#0000ff" }
                        GradientStop { position: 5 / 6; color: "#ff00ff" }
                        GradientStop { position: 6 / 6; color: "#ff0000" }
                    }
                    PathAngleArc {
                        centerX: disc.r
                        centerY: disc.r
                        radiusX: disc.r
                        radiusY: disc.r
                        sweepAngle: 360
                    }
                }
                ShapePath {
                    strokeWidth: -1
                    fillGradient: RadialGradient {
                        centerX: disc.r
                        centerY: disc.r
                        centerRadius: disc.r
                        focalX: disc.r
                        focalY: disc.r
                        GradientStop { position: 0; color: "white" }
                        GradientStop { position: 1; color: "transparent" }
                    }
                    PathAngleArc {
                        centerX: disc.r
                        centerY: disc.r
                        radiusX: disc.r
                        radiusY: disc.r
                        sweepAngle: 360
                    }
                }
            }
            Rectangle {
                width: 16
                height: 16
                radius: 8
                color: pop.current
                border.color: "white"
                border.width: 2
                x: disc.r + Math.cos(pop.hue * 2 * Math.PI) * pop.sat * disc.r - width / 2
                y: disc.r - Math.sin(pop.hue * 2 * Math.PI) * pop.sat * disc.r - height / 2
            }
            // The preview reloads the theme, so the colour is chosen on release, not at every step.
            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.CrossCursor
                onPressed: mouse => disc.pick(mouse.x, mouse.y)
                onPositionChanged: mouse => disc.pick(mouse.x, mouse.y)
                onReleased: pop.choose(pop.hex)
            }
        }

        // Brightness, from black to the wheel's colour at full strength.
        Slider {
            id: brightness
            Layout.fillWidth: true
            from: 0
            to: 1
            onMoved: pop.val = value
            onPressedChanged: if (!pressed) pop.choose(pop.hex)
            background: Rectangle {
                x: brightness.leftPadding
                y: brightness.topPadding + (brightness.availableHeight - height) / 2
                width: brightness.availableWidth
                height: 10
                radius: 5
                border.color: Style.border
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0; color: "black" }
                    GradientStop { position: 1; color: Qt.hsva(pop.hue, pop.sat, 1, 1) }
                }
            }
            handle: Rectangle {
                x: brightness.leftPadding + brightness.visualPosition * (brightness.availableWidth - width)
                y: brightness.topPadding + (brightness.availableHeight - height) / 2
                width: 18
                height: 18
                radius: 9
                color: pop.current
                border.color: "white"
                border.width: 2
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Rectangle {
                Layout.preferredWidth: 28
                Layout.preferredHeight: 28
                radius: 6
                color: pop.current
                border.color: Style.border
            }
            TextField {
                id: code
                Layout.fillWidth: true
                text: pop.hex
                font.family: "monospace"
                // A complete code applies as it's typed; "#abc" waits for Enter, since it may become "#abcdef".
                onTextEdited: if (/^#?([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(text.trim())) pop.chooseCode(text)
                onEditingFinished: pop.chooseCode(text)
            }
            ActionButton {
                text: "Paste"
                onActivated: {
                    code.selectAll()
                    code.paste()
                    pop.chooseCode(code.text)
                }
            }
        }

        Label {
            visible: pop.field !== null && pop.field.imageColours.length > 0
            text: "From your background"
            color: Style.subtext
            font.pixelSize: 12
        }
        Flow {
            Layout.fillWidth: true
            visible: pop.field !== null && pop.field.imageColours.length > 0
            spacing: 6
            Repeater {
                model: pop.field ? pop.field.imageColours : []
                delegate: Swatch {
                    required property var modelData
                    hex: modelData.hex
                    tip: modelData.label
                }
            }
        }

        Label {
            visible: pop.field !== null && pop.field.swatches.length > 0
            text: "The theme's own"
            color: Style.subtext
            font.pixelSize: 12
        }
        Flow {
            Layout.fillWidth: true
            visible: pop.field !== null && pop.field.swatches.length > 0
            spacing: 6
            Repeater {
                model: pop.field ? pop.field.swatches : []
                delegate: Swatch {
                    required property string modelData
                    hex: modelData
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            ActionButton {
                text: "Cancel"
                onActivated: pop.cancel()
            }
            ActionButton {
                text: "Done"
                primary: true
                onActivated: pop.close()
            }
        }
    }
}
