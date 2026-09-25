import QtQuick

Rectangle {
    id: prompt

    property string userName
    signal submit(string password)

    function rejected() {
        input.text = ""
        status.text = "Wrong password"
        input.forceActiveFocus()
    }

    color: "#101014"
    onVisibleChanged: if (visible) input.forceActiveFocus()

    Column {
        anchors.centerIn: parent
        spacing: 14
        width: 320

        Text {
            width: parent.width
            text: "The lock theme failed to load. Unlock with your password."
            color: "#9a9aa6"
            font.pixelSize: 13
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }
        Text {
            width: parent.width
            text: prompt.userName
            color: "#e8e8ee"
            font.pixelSize: 22
            horizontalAlignment: Text.AlignHCenter
        }
        Rectangle {
            width: parent.width
            height: 40
            radius: 6
            color: "#1c1c24"
            border.color: input.activeFocus ? "#7aa2f7" : "#33333f"
            TextInput {
                id: input
                anchors.fill: parent
                anchors.margins: 10
                color: "#e8e8ee"
                font.pixelSize: 16
                echoMode: TextInput.Password
                verticalAlignment: TextInput.AlignVCenter
                onAccepted: {
                    status.text = ""
                    prompt.submit(text)
                }
            }
        }
        Text {
            id: status
            width: parent.width
            color: "#f7768e"
            font.pixelSize: 13
            horizontalAlignment: Text.AlignHCenter
        }
    }
}
