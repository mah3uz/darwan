import QtQuick

// The user's standard customisations (docs/theme-contract.md, "Customisation"), with the theme's own
// values as fallbacks. Vendored into each theme's darwan/ folder, because SDDM loads themes on their own.
QtObject {
    readonly property var cfg: typeof config !== "undefined" ? config : ({})

    readonly property real speed: {
        const s = parseFloat(cfg.animSpeed)
        return s > 0 ? s : 1
    }
    // Ambient loops must stop themselves on this (running: !C.reduceMotion); dur() only shortens transitions.
    readonly property bool reduceMotion: cfg.reduceMotion === "true"
    readonly property string scheme: cfg.colorScheme || ""
    readonly property bool dark: scheme === "dark"

    function dur(ms) {
        return reduceMotion ? 0 : Math.round(ms / speed)
    }

    function ease(themeDefault) {
        const e = cfg.animEasing
        return e && Easing[e] !== undefined ? Easing[e] : themeDefault
    }

    // role: "accent", "text", or a colour key the theme declares in darwan.toml
    function color(role, themeDefault) {
        const key = role === "accent" ? "colorAccent" : role === "text" ? "colorText" : role
        return cfg[key] || themeDefault
    }

    // role: "text" or "clock"; an attached font file wins over a family name
    function font(role, themeDefault) {
        const loader = role === "clock" ? clockFile : textFile
        if (loader.status === FontLoader.Ready)
            return loader.name
        return (role === "clock" ? cfg.fontClock : cfg.fontText) || themeDefault
    }

    property FontLoader textFile: FontLoader {
        source: cfg.fontTextFile ? "file://" + cfg.fontTextFile : ""
    }
    property FontLoader clockFile: FontLoader {
        source: cfg.fontClockFile ? "file://" + cfg.fontClockFile : ""
    }
}
