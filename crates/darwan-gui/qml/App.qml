import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import org.darwan

// Everything inside the window: the Wall, the Stage and its settings, and what they open.
Item {
    id: app

    readonly property alias backend: backend

    Backend { id: backend }



    // The draft's look, so choosing one shows at once and Save keeps it.
    Binding {
        target: Style
        property: "look"
        // The revision is read inside the expression: the compiled binding drops a bare `backend.revision;`.
        value: backend.revision >= 0 ? backend.value("gui.look") || "darwan" : "darwan"
    }

    readonly property var gates: JSON.parse(wall.json).gates
    property string jobTitle: ""
    property string jobImage: ""
    property bool jobReports: false
    property string jobDone: ""
    property bool quitting: false
    // "themes", or the wallpaper pages "home", "explore" and "library", switched by the pill at the top.
    property string page: "themes"

    // Where a notice goes: beside the Stage's bar, or in the Wall's corner.
    function notify(text, image, progress, kind) {
        (stage.shown ? stage.toast : wallToast).show(text, image, progress, kind)
    }

    // Commands read the saved config, so the draft is saved or dropped before one runs.
    function guard(action, proceed) {
        if (backend.dirty)
            unsavedDialog.ask(action, proceed)
        else
            proceed()
    }

    function run(args, title, done) {
        guard("run “" + title + "”", () => {
            jobTitle = title
            jobDone = done || ""
            jobReports = false
            jobImage = stage.shown && stage.details ? "file://" + stage.details.still : ""
            notify(title + "…", jobImage, -1, "")
            backend.run(JSON.stringify(args))
        })
    }

    function runReport(args, title, runningText) {
        guard("run “" + title + "”", () => {
            jobTitle = title
            jobDone = ""
            jobReports = true
            reportDialog.start(title, "darwan " + args.join(" "), runningText)
            backend.run(JSON.stringify(args))
        })
    }

    function openTheme(id, from) {
        const index = wall.model.order.findIndex(c => c.id === id)
        if (index < 0)
            return
        stage.order = wall.model.order
        stage.open(index, from ? from.mapToItem(null, 0, 0, from.width, from.height) : Qt.rect(0, 0, 0, 0))
    }

    function closeTheme() {
        const id = stage.themeId
        wall.current = Math.max(0, stage.current)
        wall.keyboard = false
        const f = wall.frameOf(id)
        stage.close(f ? f.mapToItem(null, 0, 0, f.width, f.height) : Qt.rect(0, 0, 0, 0))
        wall.forceActiveFocus()
    }

    function use(gate) {
        const id = stage.themeId
        guard("use it", () => {
            if (gate === "lock" || gate === "both") {
                backend.setValueNow("lock.theme", id)
                notify("Lockscreen set", "file://" + stage.details.still, 1, "done")
            }
            if (gate === "sddm" || gate === "both")
                run(["sddm", "apply", id], "Setting the login screen", "Login screen set")
        })
    }

    Connections {
        target: backend
        function onStatusChanged() {
            if (!backend.statusOk && !backend.busy)
                app.notify(backend.status, "", 1, "fail")
        }
        function onFinished(ok, command, output) {
            if (app.jobReports) {
                reportDialog.finish(app.jobTitle, ok, command, output)
                return
            }
            lastReport.command = command
            lastReport.output = output
            app.notify(ok ? (app.jobDone || app.jobTitle + ": done") : app.jobTitle + " failed", app.jobImage, 1, ok ? "done" : "fail")
        }
    }
    QtObject {
        id: lastReport
        property string command: ""
        property string output: ""
    }

    // A playing video must be unloaded before the window goes, or Qt's FFmpeg backend crashes.
    // Hyprland's close keybind arrives here too, as a close request; only a forced kill skips it.
    function requestClose(close, again) {
        if (backend.dirty && !quitting) {
            close.accepted = false
            guard("close Darwan", again)
        } else if (!quitting) {
            quitting = true
            close.accepted = false
            stage.preview.unload()
            Qt.callLater(Qt.quit)
        }
    }

    Shortcut {
        sequence: "Ctrl+F"
        onActivated: {
            if (stage.shown)
                app.closeTheme()
            wall.search.forceActiveFocus()
            wall.search.selectAll()
        }
    }
    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: app.Window.window.close()
    }
    Shortcut {
        sequence: "Ctrl+S"
        onActivated: if (backend.dirty) backend.saveChanges()
    }
    Shortcut {
        sequence: "Ctrl+I"
        enabled: stage.shown
        onActivated: stage.inspectorOpen = !stage.inspectorOpen
    }

    Wall {
        id: wall
        anchors.fill: parent
        backend: backend
        focus: true
        // Nothing to draw under a Stage that has settled over it, or under the Wallpapers page.
        visible: app.page === "themes" && !(stage.shown && stage.settled)
        onSwitchPage: p => app.page = p
        onOpenTheme: (id, from) => app.openTheme(id, from)
        onLockNow: id => app.run(["lock", id], "Locking")
        onTestSddm: id => app.run(["sddm", "preview", id], "SDDM test mode")
        onDoctor: app.runReport(["doctor"], "System check", "Checking the session, the themes and SDDM…")
        onSettings: from => settings.openFrom(from)
    }
    Toast {
        id: wallToast
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 24
        visible: !stage.shown && shown
        onDetails: reportDialog.finish(app.jobTitle, false, lastReport.command, lastReport.output)
    }

    // A draft made from the ⚙ pop-up or left behind by a theme stays visible on the Wall until it is kept or dropped.
    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height - (backend.dirty && !stage.shown ? height + 24 : -10)
        // Save sits 8 px in on the right, as it does top and bottom, so the pills nest.
        width: wallSaveRow.x + wallSaveRow.width + 8
        height: 48
        radius: 24
        color: Style.panel
        border.color: Style.panelBorder
        GlassEdge {}
        visible: y < parent.height
        Behavior on y { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
        Row {
            id: wallSaveRow
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 10
            Label {
                anchors.verticalCenter: parent.verticalCenter
                rightPadding: 6
                text: "Unsaved changes"
                color: Style.sub
            }
            ActionButton { text: "Discard"; pill: true; onActivated: backend.discardChanges() }
            ActionButton { text: "Save"; primary: true; pill: true; tip: "Ctrl+S"; onActivated: backend.saveChanges() }
        }
    }

    Loader {
        id: wallpapers
        anchors.fill: parent
        // Made the first time it's opened, so the Themes page starts as fast as before.
        active: app.page !== "themes" || item !== null
        visible: app.page !== "themes"
        sourceComponent: WallpaperPage {
            backend: app.backend
            focus: true
            view: app.page === "themes" ? "home" : app.page
            onSwitchPage: p => app.page = p
            onOpenItem: (item, list, index) => wallpaperDetail.open(item, list, index)
        }
    }
    WallpaperDetail {
        id: wallpaperDetail
        anchors.fill: parent
        backend: backend
        visible: app.page !== "themes" && opacity > 0
    }

    Stage {
        id: stage
        anchors.fill: parent
        backend: backend
        behind: wall
        changes: inspector.form.changed
        onBack: app.closeTheme()
        onRun: (args, title) => app.run(args, title)
        onCheck: app.runReport(["check", stage.themeId], "Theme check · " + stage.details.name,
                                  "Loading the theme offscreen, looking for QML errors and missing fonts, then typing the password to check it unlocks. This takes a few seconds.")
        onUse: from => {
            useMenu.theme = { id: stage.themeId, name: stage.order[stage.current].title, still: stage.details.still }
            useMenu.parent = from
            useMenu.x = (from.width - useMenu.width) / 2
            useMenu.y = -useMenu.height - 14
            useMenu.open()
        }
        toast.onDetails: reportDialog.finish(app.jobTitle, false, lastReport.command, lastReport.output)
    }

    Inspector {
        id: inspector
        y: 12
        height: parent.height - 24
        visible: stage.shown
        open: stage.shown && stage.inspectorOpen && stage.state === ""
        backend: backend
        themeId: stage.themeId
        details: stage.details
        onPickColour: (key, from) => {
            colorPopover.fields = JSON.parse(backend.fields(stage.themeId))
            colorPopover.parent = from
            colorPopover.x = -colorPopover.width - 16
            colorPopover.y = (from.height - colorPopover.height) / 2
            colorPopover.openFor(key)
        }
        onSaverSettings: from => settings.openFrom(from, "saver")
        onImportFont: file => {
            fontPicker.fontFile = file
            fontPicker.open()
        }
        onTestSddm: app.run(["sddm", "preview", stage.themeId], "SDDM test mode")
    }

    UseMenu {
        id: useMenu
        gates: app.gates
        sddmReason: JSON.parse(backend.availability()).helper
        onChosen: gate => {
            close()
            app.use(gate)
        }
    }

    SettingsPopover {
        id: settings
        backend: backend
        themeId: stage.themeId
        onPreview: app.run(["preview", app.gates.lock ? app.gates.lock.id : "", "--saver"], "Screensaver preview")
    }

    ColorPopover {
        id: colorPopover
        margins: 8
        onChosen: (field, value) => value === "" ? backend.resetValue(field.key) : backend.setValue(field.key, value)
    }

    FileDialog {
        id: fontPicker
        property string fontFile: ""
        title: "Font file for " + stage.themeId + " (" + fontFile + ")"
        nameFilters: ["Fonts (*.ttf *.otf *.TTF *.OTF)"]
        onAccepted: app.run(["font", "import", stage.themeId,
                                decodeURIComponent(selectedFile.toString().replace(/^file:\/\//, "")),
                                "--as", fontFile], "Importing the font")
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
