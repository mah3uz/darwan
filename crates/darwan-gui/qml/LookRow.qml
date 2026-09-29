import QtQuick
import org.darwan

// Darwan's own look or the system's Qt theme (gui.look, Darwan by default), saved the moment it's picked.
SettingRow {
    id: row

    required property Backend backend

    label: "Look"
    sub: Style.own ? "Darwan's own greys and blue, with see-through panels" : "Your Qt theme's colours and font"
    Segmented {
        options: [{ value: "darwan", label: "Darwan" }, { value: "system", label: "System" }]
        current: Style.look
        onPicked: v => row.backend.setValueNow("gui.look", v)
    }
}
