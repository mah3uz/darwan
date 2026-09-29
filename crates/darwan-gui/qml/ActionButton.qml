import QtQuick
import QtQuick.Controls.Basic

// Unavailable actions stay visible and say why when hovered or clicked, as the TUI does.
Button {
    id: button

    property string reason: ""
    property bool primary: false
    property bool pill: false
    property bool compact: false
    property string glyph: ""
    property string tip: ""
    readonly property bool available: reason === ""

    signal activated()
    signal refused(string reason)

    hoverEnabled: true
    // A click doesn't take the keyboard focus away from where it was.
    focusPolicy: Qt.TabFocus
    onClicked: available ? activated() : refused(reason)

    ToolTip.visible: hovered && (!available || tip !== "")
    ToolTip.text: available ? tip : reason
    ToolTip.delay: 400

    font.family: Style.family
    font.pixelSize: compact ? Style.caption : Style.body
    font.weight: primary ? Font.DemiBold : Font.Normal
    leftPadding: text === "" ? 0 : pill ? 16 : 12
    rightPadding: text === "" ? 0 : pill ? 16 : 12

    scale: down ? 0.97 : 1
    Behavior on scale { NumberAnimation { duration: 100 } }

    contentItem: Item {
        implicitWidth: row.implicitWidth
        implicitHeight: row.implicitHeight
        Row {
            id: row
            anchors.centerIn: parent
            spacing: 6
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: button.glyph !== ""
                name: button.glyph
                size: button.compact ? 13 : 15
                color: label.color
            }
            Label {
                id: label
                anchors.verticalCenter: parent.verticalCenter
                visible: button.text !== ""
                text: button.text
                font: button.font
                color: !button.available ? Style.muted : button.primary ? Style.accentText : Style.text
            }
        }
    }

    background: Rectangle {
        implicitHeight: button.compact ? 26 : 32
        implicitWidth: button.text === "" ? implicitHeight : 64
        radius: button.pill || button.text === "" ? height / 2 : Style.small
        color: button.primary && button.available ? (button.hovered ? Qt.lighter(Style.accent, 1.08) : Style.accent)
             : button.flat && !button.hovered && !button.down ? "transparent"
             : button.down || (button.hovered && button.available) ? Style.controlHover
             : Style.control
        border.width: button.visualFocus ? 2 : 0
        border.color: Style.accent
        Behavior on color { ColorAnimation { duration: Style.fast } }
    }
}
