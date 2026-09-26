pragma Singleton
import QtQuick

// The TUI's truecolor palette, so both front ends look like one app.
QtObject {
    readonly property color crust: "#11111b"
    readonly property color mantle: "#181825"
    readonly property color base: "#1e1e2e"
    readonly property color surface: "#313244"
    readonly property color border: "#45475a"
    readonly property color muted: "#6c7086"
    readonly property color subtext: "#a6adc8"
    readonly property color text: "#cdd6f4"
    readonly property color accent: "#b4befe"
    readonly property color lock: "#89dceb"
    readonly property color sddm: "#f5c2e7"
    readonly property color ok: "#a6e3a1"
    readonly property color warn: "#f9e2af"
    readonly property color error: "#f38ba8"

    readonly property int radius: 8
    readonly property int gap: 12
}
