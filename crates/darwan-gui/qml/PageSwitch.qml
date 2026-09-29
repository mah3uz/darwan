pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import org.darwan

// The app's pages in one pill, as wallspace's Home / Explore / Library: the chosen one lit in white.
Rectangle {
    id: nav

    property string current: "themes"
    // Over a picture: darker glass, so it reads on anything.
    property bool glass: false
    property Item chosen: null
    readonly property var pages: [
        { value: "themes", label: "Themes" },
        { value: "home", label: "Home" },
        { value: "explore", label: "Explore" },
        { value: "library", label: "Library" }
    ]

    signal picked(string value)

    implicitWidth: row.implicitWidth + 12
    implicitHeight: 46
    radius: height / 2
    color: glass ? Qt.rgba(0, 0, 0, 0.38) : Style.control
    border.color: Qt.rgba(1, 1, 1, glass ? 0.14 : 0.06)

    Rectangle {
        y: 6
        height: parent.height - 12
        x: nav.chosen ? nav.chosen.x + 6 : 6
        width: nav.chosen ? nav.chosen.width : 0
        visible: nav.chosen !== null
        radius: height / 2
        color: "white"
        Behavior on x { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        Behavior on width { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
    }

    Row {
        id: row
        x: 6
        y: 6
        Repeater {
            model: nav.pages
            delegate: AbstractButton {
                id: tab
                required property var modelData
                required property int index
                readonly property bool on: modelData.value === nav.current
                height: nav.height - 12
                implicitWidth: label.implicitWidth + 40
                hoverEnabled: true
                focusPolicy: Qt.TabFocus
                onClicked: if (!on) nav.picked(modelData.value)
                Binding { when: tab.on; target: nav; property: "chosen"; value: tab }
                contentItem: Item {}
                Label {
                    id: label
                    anchors.centerIn: parent
                    text: tab.modelData.label
                    font.family: Style.family
                    font.pixelSize: 15
                    font.weight: tab.on ? Font.DemiBold : Font.Medium
                    color: tab.on ? "#111111" : tab.hovered ? "white" : Qt.rgba(1, 1, 1, 0.82)
                    Behavior on color { ColorAnimation { duration: Style.fast } }
                }
                // A thin divider between two unlit tabs.
                Rectangle {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    width: 1
                    height: 16
                    color: Qt.rgba(1, 1, 1, 0.18)
                    visible: tab.index < nav.pages.length - 1 && !tab.on && nav.pages[tab.index + 1].value !== nav.current
                }
                Rectangle {
                    anchors.fill: parent
                    radius: height / 2
                    color: "transparent"
                    border.width: tab.visualFocus ? 2 : 0
                    border.color: Style.accent
                }
            }
        }
    }
}
