import QtQuick

Item {
    objectName: "goodTheme"
    readonly property string mode: config.themeMode
    readonly property bool sawKeyboard: typeof keyboard !== "undefined" && keyboard.numLock !== undefined
    Component.onCompleted: keyboard.numLock = true
}
