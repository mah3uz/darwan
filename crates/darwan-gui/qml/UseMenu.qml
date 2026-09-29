import QtQuick
import QtQuick.Controls.Basic

// Where to use the open theme: a card per gate, each showing the theme there now and this one.
Popover {
    id: pop

    property var theme: null
    property var gates: ({ lock: null, sddm: null })
    property string sddmReason: ""

    signal chosen(string gate)

    width: 470
    padding: 10
    transformOrigin: Popup.Bottom

    component Gate: AbstractButton {
        id: gate
        property string target: ""
        property string label: ""
        property string glyph: ""
        property var now: null
        property string reason: ""
        readonly property bool already: pop.theme !== null && now !== null && now.id === pop.theme.id
        width: 221
        height: 118
        hoverEnabled: true
        enabled: !already && reason === ""
        focusPolicy: Qt.TabFocus
        onClicked: pop.chosen(target)
        ToolTip.visible: hovered && reason !== ""
        ToolTip.text: reason
        background: Rectangle {
            radius: 10
            color: gate.hovered && gate.enabled ? Style.accentSoft : Style.group
            border.width: gate.visualFocus || (gate.hovered && gate.enabled) ? 1.5 : 1
            border.color: gate.hovered && gate.enabled || gate.visualFocus ? Style.accent : Style.sep
            opacity: gate.enabled ? 1 : 0.5
        }
        contentItem: Column {
            leftPadding: 10
            topPadding: 10
            spacing: 8
            opacity: gate.enabled ? 1 : 0.55
            Row {
                spacing: 6
                Rounded {
                    width: 52
                    height: 29
                    radius: 5
                    opacity: 0.6
                    Image {
                        anchors.fill: parent
                        source: gate.now ? "file://" + gate.now.still : ""
                        sourceSize.width: 104
                        fillMode: Image.PreserveAspectCrop
                    }
                }
                Icon { anchors.verticalCenter: parent.verticalCenter; name: "right"; size: 14; color: Style.muted }
                Rounded {
                    width: 76
                    height: 43
                    radius: 6
                    Image {
                        anchors.fill: parent
                        source: pop.theme ? "file://" + pop.theme.still : ""
                        sourceSize.width: 152
                        fillMode: Image.PreserveAspectCrop
                    }
                }
            }
            Row {
                spacing: 6
                Icon { anchors.verticalCenter: parent.verticalCenter; name: gate.glyph; size: 14; color: Style.text }
                Label { text: gate.label; font.family: Style.family; font.pixelSize: Style.body; font.weight: Font.DemiBold; color: Style.text }
            }
            Label {
                text: gate.already ? "Already this theme" : "Now: " + (gate.now ? gate.now.name : "nothing")
                font.family: Style.family
                font.pixelSize: Style.caption
                color: Style.sub
            }
        }
    }

    Column {
        width: 450
        spacing: 8
        Label {
            leftPadding: 4
            text: "Use " + (pop.theme ? pop.theme.name : "") + " as"
            font.family: Style.family
            font.pixelSize: Style.caption
            font.weight: Font.DemiBold
            color: Style.sub
        }
        Row {
            spacing: 8
            Gate { target: "lock"; label: "Lockscreen"; glyph: "lock"; now: pop.gates.lock }
            Gate { target: "sddm"; label: "Login screen"; glyph: "login"; now: pop.gates.sddm; reason: pop.sddmReason }
        }
        ActionButton {
            width: parent.width
            text: "Both"
            reason: pop.sddmReason
            onActivated: pop.chosen("both")
        }
    }
}
