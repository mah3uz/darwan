import QtQuick
import QtQuick.Window

Item {
    readonly property real s: Screen.height / 768
    id: btn
    property string text: ""
    property bool primary: false
    property int fontPixelSize: 32
    signal clicked()

    width: Math.max(220 * s, buttonText.implicitWidth + 40 * s)
    height: 60 * s

    scale: btnMouse.pressed ? 0.95 : (btnMouse.containsMouse ? 1.05 : 1.0)
    Behavior on scale { NumberAnimation { duration: 100; easing.type: Easing.OutBack } }

    // Outer black border
    Rectangle {
        anchors.fill: parent
        color: "#000000"
        radius: 12 * s
        
        // Inner colored border
        Rectangle {
            anchors.fill: parent
            anchors.margins: 2 * s
            color: btnMouse.containsMouse ? "#fff200" : "#3b4a8e"
            radius: 10 * s
            
            // Background fill
            Rectangle {
                anchors.fill: parent
                anchors.margins: 2 * s
                color: btnMouse.containsMouse ? "#435293" : "#2d3560"
                radius: 8 * s
                
                Text {
                    id: buttonText
                    text: btn.text
                    anchors.centerIn: parent
                    font.family: mainFont.name
                    font.pixelSize: btn.fontPixelSize
                    color: btnMouse.containsMouse ? "#fff200" : "#ffffff"
                    style: Text.Outline
                    styleColor: "#000000"
                }
            }
        }
    }

    MouseArea {
        id: btnMouse
        anchors.fill: parent
        hoverEnabled: true
        onClicked: btn.clicked()
    }
}
