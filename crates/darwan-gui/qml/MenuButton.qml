import QtQuick
import QtQuick.Controls.Basic

// A pop-up button: shows the current choice, opens the list under it.
ComboBox {
    id: box

    // [{ value, label }]
    property var options: []
    property string current: ""
    property bool isDefault: false
    signal picked(string value)

    model: options
    textRole: "label"
    valueRole: "value"
    // Waits for the model, which a JS array only becomes after this binding first runs.
    currentIndex: count > 0 ? indexOfValue(current) : -1
    onActivated: picked(currentValue)
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    implicitHeight: 26
    implicitWidth: Math.min(220, Math.max(90, contentItem.implicitWidth + 36))
    font.family: Style.family
    font.pixelSize: Style.body

    contentItem: Label {
        leftPadding: 10
        text: box.displayText
        font: box.font
        color: box.isDefault ? Style.sub : Style.text
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    indicator: Icon {
        x: box.width - width - 8
        anchors.verticalCenter: parent.verticalCenter
        name: "down"
        size: 12
        color: Style.sub
    }
    background: Rectangle {
        radius: Style.small
        color: box.hovered || box.pressed ? Style.controlHover : Style.control
        border.width: box.visualFocus ? 2 : 0
        border.color: Style.accent
        Behavior on color { ColorAnimation { duration: Style.fast } }
    }
    delegate: ItemDelegate {
        required property var modelData
        required property int index
        width: ListView.view ? ListView.view.width : 0
        height: 28
        highlighted: box.highlightedIndex === index
        contentItem: Label {
            text: modelData.label
            font.family: Style.family
            font.pixelSize: Style.body
            font.weight: modelData.value === box.current ? Font.DemiBold : Font.Normal
            color: parent.highlighted ? Style.accentText : Style.text
            verticalAlignment: Text.AlignVCenter
        }
        background: Rectangle {
            radius: Style.small
            color: parent.highlighted ? Style.accent : "transparent"
        }
    }
    popup: Popup {
        y: box.height + 4
        width: Math.max(box.width, 180)
        implicitHeight: Math.min(contentItem.implicitHeight + 8, 320)
        padding: 4
        margins: 8
        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.fast }
            NumberAnimation { property: "scale"; from: 0.96; to: 1; duration: Style.fast; easing.type: Style.ease }
        }
        exit: Transition { NumberAnimation { property: "opacity"; to: 0; duration: 100 } }
        transformOrigin: Popup.Top
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: box.popup.visible ? box.delegateModel : null
            currentIndex: box.highlightedIndex
            boundsBehavior: Flickable.StopAtBounds
        }
        background: Rectangle {
            radius: Style.radius - 2
            color: Style.popover
            border.color: Style.panelBorder
        }
    }
}
