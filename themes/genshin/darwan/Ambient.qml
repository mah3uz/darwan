import QtQuick

// The screensaver's state (docs/theme-contract.md, "Screensaver"). `wake` is 1 while the widgets show and 0 in ambient,
// animated both ways; a theme multiplies its widgets' opacity by it and keeps its own intro animation. Under SDDM
// there is no screensaver, so `wake` stays 1.
Item {
    id: ambient

    readonly property bool active: typeof darwan !== "undefined" && darwan.ambient
    // kit.dur(700) in themes that honour the user's motion settings.
    property int duration: 700
    property real wake: active ? 0 : 1
    Behavior on wake { NumberAnimation { duration: ambient.duration; easing.type: Easing.InOutQuad } }
}
