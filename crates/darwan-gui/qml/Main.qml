import QtQuick
import QtQuick.Controls.Basic
import org.darwan

ApplicationWindow {
    id: window

    visible: true
    width: 1440
    height: 900
    minimumWidth: 1100
    minimumHeight: 700
    title: app.backend.dirty ? "Darwan \u2014 Edited" : "Darwan"
    color: Style.bg
    font.family: Style.family
    font.pixelSize: Style.body

    palette {
        window: Style.bg
        windowText: Style.text
        base: Style.bg
        alternateBase: Style.group
        text: Style.text
        button: Style.control
        buttonText: Style.text
        highlight: Style.accent
        highlightedText: Style.accentText
        light: Style.control
        midlight: Style.control
        mid: Style.sep
        dark: Style.sep
        shadow: Style.bgDeep
        placeholderText: Style.muted
        toolTipBase: Style.popover
        toolTipText: Style.text
    }

    App {
        id: app
        anchors.fill: parent
    }

    onClosing: close => app.requestClose(close, () => window.close())
}
