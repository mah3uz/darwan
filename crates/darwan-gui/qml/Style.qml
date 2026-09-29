pragma Singleton
import QtQuick

// Two looks: "darwan" (its own greys and blue, see-through panels) and "system" (the Qt theme's own palette and font,
// solid panels). gui.look picks one; App sets `look`.
QtObject {
    property string look: "darwan"
    readonly property bool own: look !== "system"
    readonly property SystemPalette sys: SystemPalette { colorGroup: SystemPalette.Active }
    readonly property bool hasSf: Qt.fontFamilies().indexOf("SF Pro Text") >= 0

    readonly property color bg: own ? "#1e1e1e" : sys.window
    readonly property color bgDeep: own ? "#141414" : sys.dark
    readonly property color chrome: own ? Qt.rgba(0.1, 0.1, 0.12, 0.42) : sys.window
    readonly property color panel: own ? Qt.rgba(0.12, 0.12, 0.14, 0.6) : sys.window
    readonly property color popover: own ? Qt.rgba(0.13, 0.13, 0.15, 0.9) : sys.window
    readonly property color panelBorder: own ? Qt.rgba(1, 1, 1, 0.15) : Qt.alpha(sys.windowText, 0.12)
    // The lit top edge of a glass surface; none in the system look.
    readonly property color edge: own ? Qt.rgba(1, 1, 1, 0.28) : "transparent"
    readonly property color group: own ? Qt.rgba(1, 1, 1, 0.065) : Qt.tint(sys.base, Qt.alpha(sys.text, 0.05))
    readonly property color control: own ? Qt.rgba(1, 1, 1, 0.1) : Qt.tint(sys.button, Qt.alpha(sys.buttonText, 0.1))
    readonly property color controlHover: own ? Qt.rgba(1, 1, 1, 0.16) : Qt.tint(sys.button, Qt.alpha(sys.buttonText, 0.16))
    readonly property color sep: own ? Qt.rgba(1, 1, 1, 0.09) : Qt.alpha(sys.windowText, 0.1)
    readonly property color text: own ? Qt.rgba(1, 1, 1, 0.88) : sys.windowText
    readonly property color sub: own ? Qt.rgba(1, 1, 1, 0.56) : Qt.tint(sys.window, Qt.alpha(sys.windowText, 0.62))
    readonly property color muted: own ? Qt.rgba(1, 1, 1, 0.32) : sys.placeholderText
    readonly property color accent: own ? "#0a84ff" : sys.highlight
    readonly property color accentText: own ? "#ffffff" : sys.highlightedText
    readonly property color accentSoft: Qt.alpha(accent, 0.22)
    readonly property color switchOff: own ? Qt.rgba(1, 1, 1, 0.16) : Qt.tint(sys.window, Qt.alpha(sys.windowText, 0.18))
    readonly property color lock: own ? "#64d2ff" : "#5ac8fa"
    readonly property color login: "#ff6ab4"
    readonly property color warn: own ? "#ffd60a" : "#ffcc00"
    readonly property color ok: own ? "#30d158" : "#34c759"
    readonly property color danger: "#ff453a"

    readonly property int radius: own ? 12 : 10
    readonly property int small: 6
    readonly property real backdropDim: own ? 0.6 : 0.7

    readonly property string family: own && hasSf ? "SF Pro Text" : Qt.application.font.family
    readonly property string mono: "JetBrainsMono Nerd Font"
    readonly property int body: 13
    readonly property int caption: 12
    readonly property int title: 15

    readonly property int fast: 150
    readonly property int medium: 250
    readonly property int slow: 350
    readonly property int ease: Easing.OutCubic

    readonly property int gap: 12

    // Older names, until every screen is on the tokens above.
    readonly property color crust: bgDeep
    readonly property color mantle: popover
    readonly property color base: bg
    readonly property color surface: control
    readonly property color border: sep
    readonly property color subtext: sub
    readonly property color sddm: login
    readonly property color error: danger
}
