import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import org.darwan

ApplicationWindow {
    id: window

    visible: true
    width: 1440
    height: 900
    minimumWidth: 1100
    minimumHeight: 700
    title: backend.dirty ? "Darwan · unsaved changes" : "Darwan"
    color: Style.base

    palette {
        window: Style.base
        windowText: Style.text
        base: Style.base
        alternateBase: Style.mantle
        text: Style.text
        button: Style.surface
        buttonText: Style.text
        highlight: Style.accent
        highlightedText: Style.crust
        light: Style.surface
        midlight: Style.surface
        mid: Style.border
        dark: Style.border
        shadow: Style.crust
        placeholderText: Style.muted
        toolTipBase: Style.surface
        toolTipText: Style.text
    }

    Backend { id: backend }

    readonly property string themeId: gallery.currentId
    readonly property var details: {
        backend.revision
        return themeId === "" ? null : JSON.parse(backend.details(themeId))
    }
    readonly property var avail: {
        backend.revision
        return JSON.parse(backend.availability())
    }
    readonly property string busyReason: backend.busy ? "wait for the running command to finish" : ""

    property string message: backend.status
    property string shownFields: ""
    onThemeIdChanged: shownFields = backend.fields(themeId)
    property bool quitting: false

    function need(...reasons) {
        return reasons.find(r => r !== "") || ""
    }

    property string jobTitle: ""
    property bool jobReports: false

    // Commands read the saved config, so the draft is saved or dropped before one runs.
    function guard(action, proceed) {
        if (backend.dirty)
            unsavedDialog.ask(action, proceed)
        else
            proceed()
    }

    function run(args, title) {
        guard("run \u201c" + title + "\u201d", () => {
            jobTitle = title
            jobReports = false
            backend.run(JSON.stringify(args))
        })
    }

    function runReport(args, title, runningText) {
        guard("run \u201c" + title + "\u201d", () => {
            jobTitle = title
            jobReports = true
            reportDialog.start(title, "darwan " + args.join(" "), runningText)
            backend.run(JSON.stringify(args))
        })
    }

    Connections {
        target: backend
        function onStatusChanged() { window.message = backend.status }
        // Only a change to this theme's settings needs the preview reloaded.
        function onRevisionChanged() {
            const fields = backend.fields(window.themeId)
            if (fields !== window.shownFields) {
                window.shownFields = fields
                preview.reload()
            }
        }
        function onFinished(ok, command, output) {
            if (window.jobReports || !ok)
                reportDialog.finish(window.jobReports ? window.jobTitle : window.jobTitle + " failed", ok, command, output)
        }
    }

    // A playing video must be unloaded before the window goes, or Qt's FFmpeg backend crashes.
    // Hyprland's close keybind arrives here too, as a close request; only a forced kill skips it.
    onClosing: close => {
        if (backend.dirty && !quitting) {
            close.accepted = false
            guard("close Darwan", () => window.close())
        } else if (!quitting) {
            quitting = true
            close.accepted = false
            preview.unload()
            Qt.callLater(Qt.quit)
        }
    }

    Shortcut {
        sequence: "Ctrl+F"
        onActivated: gallery.searchField.forceActiveFocus()
    }
    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: window.close()
    }
    Shortcut {
        sequence: "Ctrl+S"
        onActivated: if (backend.dirty) backend.saveChanges()
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Gallery {
            id: gallery
            backend: backend
            guard: window.guard
            Layout.preferredWidth: 320
            Layout.fillHeight: true
        }

        Rectangle { Layout.fillHeight: true; implicitWidth: 1; color: Style.surface }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Style.gap * 1.5
            spacing: Style.gap

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2
                    Label {
                        Layout.fillWidth: true
                        text: window.details ? window.details.name : ""
                        color: Style.text
                        font.pixelSize: 22
                        font.bold: true
                        elide: Text.ElideRight
                    }
                    Label {
                        text: window.details
                              ? window.details.id + "  ·  by " + window.details.author + "  ·  " + window.details.background + " background"
                              : ""
                        color: Style.subtext
                    }
                }
                Tag { visible: window.details && window.details.isLock; text: "LOCK"; tint: Style.lock }
                Tag { visible: window.details && window.details.isSddm; text: "LOGIN"; tint: Style.sddm }
                ActionButton {
                    text: "Doctor"
                    reason: window.busyReason
                    onActivated: window.runReport(["doctor"], "System check", "Checking the session, the themes and SDDM…")
                    onRefused: r => window.message = r
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 4
                Repeater {
                    model: [{ mode: "lock", label: "Lockscreen" }, { mode: "sddm", label: "Login screen layout" }]
                    delegate: Button {
                        required property var modelData
                        text: modelData.label
                        checkable: true
                        checked: preview.mode === modelData.mode
                        onClicked: preview.mode = modelData.mode
                        contentItem: Label {
                            text: parent.text
                            color: parent.checked ? Style.crust : Style.subtext
                            horizontalAlignment: Text.AlignHCenter
                        }
                        background: Rectangle {
                            implicitHeight: 28
                            radius: 14
                            color: parent.checked ? Style.accent : Style.surface
                        }
                    }
                }
                Item { Layout.fillWidth: true }
                Label {
                    text: preview.usingFallback
                          ? "The theme failed to load; this is the fallback prompt"
                          : "Live preview · mock login, the password is \"test\""
                    color: preview.usingFallback ? Style.warn : Style.muted
                    font.pixelSize: 12
                }
            }

            LivePreview {
                id: preview
                backend: backend
                themeId: window.themeId
                themePath: window.details ? window.details.dir : ""
                fallbackFocus: gallery.list
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: Style.radius
                onUnlocked: window.message = "Unlocked with the mock password"
                onFailed: m => window.message = m
            }

            Flow {
                Layout.fillWidth: true
                spacing: 8

                ActionButton {
                    text: window.details && window.details.isLock ? "Lock theme ✓" : "Use for lock"
                    primary: !(window.details && window.details.isLock)
                    reason: window.details && window.details.isLock ? "already the lock theme" : ""
                    onActivated: backend.setValueNow("lock.theme", window.themeId)
                    onRefused: r => window.message = r
                }
                ActionButton {
                    text: "Lock now"
                    reason: window.need(window.avail.wayland, window.busyReason)
                    onActivated: window.run(["lock", window.themeId], "Lock now")
                    onRefused: r => window.message = r
                }
                ActionButton {
                    text: "Full-screen preview"
                    reason: window.need(window.avail.wayland, window.busyReason)
                    onActivated: window.run(["preview", window.themeId].concat(preview.mode === "sddm" ? ["--sddm"] : []), "Full-screen preview")
                    onRefused: r => window.message = r
                }
                ActionButton {
                    text: "Apply to SDDM"
                    reason: window.need(window.avail.helper, window.busyReason)
                    onActivated: window.run(["sddm", "apply", window.themeId], "Apply to SDDM")
                    onRefused: r => window.message = r
                }
                ActionButton {
                    text: "SDDM test mode"
                    reason: window.need(window.avail.sddmPreview, window.busyReason)
                    onActivated: window.run(["sddm", "preview", window.themeId], "SDDM test mode")
                    onRefused: r => window.message = r
                }
                ActionButton {
                    text: "Check"
                    reason: window.busyReason
                    onActivated: window.runReport(["check", window.themeId], "Theme check · " + window.details.name,
                                                  "Loading the theme offscreen, looking for QML errors and missing fonts, then typing the password to check it unlocks. This takes a few seconds.")
                    onRefused: r => window.message = r
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: window.details !== null && window.details.fonts.length > 0
                spacing: 6

                Label {
                    text: "Fonts"
                    color: Style.accent
                    font.bold: true
                }
                Repeater {
                    model: window.details ? window.details.fonts : []
                    delegate: RowLayout {
                        id: fontRow
                        required property var modelData
                        Layout.fillWidth: true
                        spacing: 10

                        StatusBadge {
                            kind: fontRow.modelData.installed ? "ok" : "warn"
                        }
                        Label {
                            Layout.fillWidth: true
                            text: fontRow.modelData.family + " (" + fontRow.modelData.file + ")  ·  " + fontRow.modelData.license
                                  + (fontRow.modelData.installed ? "" : "  ·  missing, a fallback font is used")
                            color: fontRow.modelData.installed ? Style.subtext : Style.text
                            elide: Text.ElideRight
                        }
                        ActionButton {
                            visible: fontRow.modelData.url !== ""
                            text: "Get it"
                            onActivated: Qt.openUrlExternally(fontRow.modelData.url)
                        }
                        ActionButton {
                            visible: !fontRow.modelData.installed
                            text: "Import…"
                            reason: window.need(
                                window.avail.helper === "" ? "" : window.avail.helper + "; in a checkout, put the file in themes/" + window.themeId + "/font/",
                                window.busyReason)
                            onActivated: {
                                fontPicker.fontFile = fontRow.modelData.file
                                fontPicker.open()
                            }
                            onRefused: r => window.message = r
                        }
                    }
                }
            }
        }

        Rectangle { Layout.fillHeight: true; implicitWidth: 1; color: Style.surface }

        SettingsForm {
            backend: backend
            themeId: window.themeId
            themeName: window.details ? window.details.name : ""
            Layout.preferredWidth: 400
            Layout.fillHeight: true
        }
    }

    footer: Rectangle {
        implicitHeight: 30
        color: Style.crust
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Style.gap
            anchors.rightMargin: Style.gap
            BusyIndicator {
                visible: backend.busy
                running: backend.busy
                Layout.preferredWidth: 18
                Layout.preferredHeight: 18
            }
            Label {
                Layout.fillWidth: true
                text: window.message
                color: Style.subtext
                elide: Text.ElideRight
            }
        }
    }

    FileDialog {
        id: fontPicker
        property string fontFile: ""
        title: "Font file for " + window.themeId + " (" + fontFile + ")"
        nameFilters: ["Fonts (*.ttf *.otf *.TTF *.OTF)"]
        onAccepted: window.run(["font", "import", window.themeId,
                                decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")),
                                "--as", fontFile], "Import font")
    }

    ReportDialog {
        id: reportDialog
        backend: backend
    }

    UnsavedDialog {
        id: unsavedDialog
        backend: backend
    }
}
