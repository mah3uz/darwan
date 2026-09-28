import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.darwan

// hypridle starts the screensaver: this window shows whether it can, writes its hypridle.conf, and holds the saver's
// own settings. hypridle changes are written at once (hypridle reads the file, not this window's draft).
Popup {
    id: dialog

    required property Backend backend
    signal preview()
    signal said(string text, bool ok)

    property int checks: 0
    readonly property var st: {
        backend.revision
        dialog.checks
        return dialog.visible ? JSON.parse(backend.idleStatus()) : null
    }
    readonly property var conf: st ? st.config : null

    readonly property var saverTimes: [[60, "1 minute"], [120, "2 minutes"], [180, "3 minutes"], [300, "5 minutes"],
                                        [600, "10 minutes"], [900, "15 minutes"], [1800, "30 minutes"]]
    readonly property var screenTimes: [[0, "Never"], [300, "5 minutes"], [600, "10 minutes"], [900, "15 minutes"],
                                         [1800, "30 minutes"], [3600, "1 hour"]]
    readonly property var suspendTimes: [[0, "Never"], [900, "15 minutes"], [1800, "30 minutes"], [3600, "1 hour"],
                                          [7200, "2 hours"]]
    readonly property var lockTimes: [["0", "At once"], ["5", "After 5 seconds"], ["10", "After 10 seconds"],
                                       ["30", "After 30 seconds"], ["60", "After a minute"], ["300", "After 5 minutes"],
                                       ["never", "Never"]]
    readonly property var qualities: [
        ["full", "Full", "Videos as shipped, on every monitor. The smoothest, and the most GPU work, power and memory (a 4K video theme: about 4% CPU and 1 GB)."],
        ["auto", "Automatic", "Chosen from this machine: smaller copies on integrated graphics, on battery, without a video decoder driver or with under 8 GB of memory; still frames in power-saver mode. Only the focused monitor plays video."],
        ["eco", "Eco", "Copies at up to 1080p and 30 fps, made once when you pick a theme or background: about a quarter of the decoding work, softer on large screens. Only the focused monitor plays video."],
        ["still", "Still", "The first frame of each video: almost no GPU work or power, and no motion in video backgrounds. Other animations still play."]
    ]

    function minutes(secs) {
        return secs % 60 === 0 ? (secs / 60) + (secs === 60 ? " minute" : " minutes") : secs + " seconds"
    }
    // A timeout the list doesn't have (set by hand) still shows as itself.
    function withValue(list, value) {
        if (value === null || value === undefined || list.some(e => e[0] === value))
            return list
        return list.concat([[value, minutes(value)]])
    }
    function change(action, value) {
        backend.idleChange(action, String(value))
    }

    parent: Overlay.overlay
    anchors.centerIn: parent
    width: Math.min(760, parent.width - 64)
    height: Math.min(implicitHeight, parent.height - 64)
    modal: true
    padding: 0
    closePolicy: Popup.CloseOnEscape
    focus: true

    Overlay.modal: Rectangle { color: Qt.alpha(Style.crust, 0.65) }
    background: Rectangle { radius: 14; color: Style.mantle; border.color: Style.border }
    enter: Transition { NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120 } }
    exit: Transition { NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 90 } }

    component Heading: Label {
        topPadding: 10
        color: Style.muted
        font.pixelSize: 11
        font.bold: true
        font.letterSpacing: 1
    }
    component Note: Label {
        Layout.fillWidth: true
        color: Style.subtext
        wrapMode: Text.WordWrap
        font.pixelSize: 12
    }
    component Code: Rectangle {
        id: codeBox
        property string code: ""
        Layout.fillWidth: true
        implicitHeight: codeRow.implicitHeight + 16
        radius: 6
        color: Style.crust
        RowLayout {
            id: codeRow
            anchors.fill: parent
            anchors.margins: 8
            TextEdit {
                id: codeText
                Layout.fillWidth: true
                text: codeBox.code
                readOnly: true
                selectByMouse: true
                color: Style.text
                font.family: "monospace"
                font.pixelSize: 12
                wrapMode: TextEdit.WrapAnywhere
            }
            ActionButton {
                text: "Copy"
                onActivated: {
                    codeText.selectAll()
                    codeText.copy()
                    codeText.deselect()
                    dialog.said("copied", true)
                }
            }
        }
    }
    component Setting: RowLayout {
        property string label: ""
        Layout.fillWidth: true
        spacing: 12
        Label {
            Layout.preferredWidth: 250
            text: parent.label
            color: Style.text
            wrapMode: Text.WordWrap
        }
    }
    component Choice: ComboBox {
        property var entries: []
        property var current
        signal chosen(var value)
        Layout.fillWidth: true
        model: entries.map(e => e[1])
        currentIndex: entries.findIndex(e => e[0] === current)
        onActivated: i => chosen(entries[i][0])
    }

    contentItem: ColumnLayout {
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 20
            spacing: 10
            Label {
                Layout.fillWidth: true
                text: "Screensaver"
                color: Style.text
                font.pixelSize: 17
                font.bold: true
            }
            Tag {
                text: !dialog.st ? "" : !dialog.st.installed ? "HYPRIDLE MISSING" : dialog.st.running ? "HYPRIDLE RUNNING" : "HYPRIDLE STOPPED"
                tint: dialog.st && dialog.st.running ? Style.ok : Style.warn
            }
        }

        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Style.surface }

        Flickable {
            id: body
            Layout.fillWidth: true
            Layout.fillHeight: true
            implicitHeight: content.implicitHeight + 32
            contentHeight: content.implicitHeight + 32
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}

            ColumnLayout {
                id: content
                x: 20
                y: 16
                width: body.width - 40
                spacing: 10

                Note {
                    text: "When you're idle, hypridle starts the screensaver: your lock theme's background and animation, without its widgets. A key or click brings the lock back, and what you type goes into the password field."
                }

                // 1. hypridle itself
                Heading { text: "HYPRIDLE" }
                ColumnLayout {
                    visible: dialog.st !== null && !dialog.st.installed
                    Layout.fillWidth: true
                    spacing: 8
                    Note { text: "hypridle, Hyprland's idle daemon, isn't installed. Install it, then check again." }
                    Code { code: "sudo pacman -S hypridle" }
                }
                ColumnLayout {
                    visible: dialog.st !== null && dialog.st.installed && !dialog.st.running
                    Layout.fillWidth: true
                    spacing: 8
                    Note {
                        text: dialog.st && dialog.st.uwsm
                              ? "hypridle isn't running. Your session runs under uwsm, so let systemd start it with every login:"
                              : "hypridle isn't running. Start it with Hyprland, in " + (dialog.st && dialog.st.lua ? "~/.config/hypr/hyprland.lua:" : "~/.config/hypr/hyprland.conf:")
                    }
                    Code {
                        code: !dialog.st ? ""
                              : dialog.st.uwsm ? "systemctl --user enable --now hypridle.service"
                              : dialog.st.lua ? "hl.on(\"hyprland.start\", function() hl.exec_cmd(\"hypridle\") end)"
                              : "exec-once = hypridle"
                    }
                    Note {
                        text: dialog.st && dialog.st.uwsm
                              ? "Without uwsm, start it from Hyprland's config instead (hl.exec_cmd(\"hypridle\") in hyprland.start, or exec-once = hypridle)."
                              : "If Hyprland is started with uwsm, use systemctl --user enable --now hypridle.service instead."
                    }
                    RowLayout {
                        ActionButton {
                            text: "Start now"
                            primary: true
                            onActivated: dialog.backend.idleStart()
                        }
                        Note { text: "Runs it for this session only; the line above starts it every time." }
                    }
                }
                Note {
                    visible: dialog.st !== null && dialog.st.running
                    text: "hypridle is running. Turn your shell's own idle lock and lock-before-sleep off (DankMaterialShell: Settings → Power & Sleep), or both answer."
                }

                // 2. hypridle.conf
                Heading { visible: dialog.st !== null && dialog.st.installed; text: "IDLE AND SLEEP" }
                ColumnLayout {
                    visible: dialog.st !== null && dialog.st.installed && dialog.conf === null
                    Layout.fillWidth: true
                    spacing: 8
                    Note { text: "There is no " + (dialog.st ? dialog.st.path : "hypridle.conf") + " yet. Enable writes one: the screensaver after 5 minutes, the screen off after 10, darwan locking before sleep and turning the screen back on after it." }
                    ActionButton {
                        text: "Enable the screensaver"
                        primary: true
                        onActivated: dialog.change("enable", "")
                    }
                }
                ColumnLayout {
                    visible: dialog.conf !== null
                    Layout.fillWidth: true
                    spacing: 10

                    Setting {
                        label: "Start the screensaver when idle"
                        Switch {
                            checked: dialog.conf !== null && dialog.conf.saver !== null
                            onToggled: dialog.change(checked ? "enable" : "disable", "")
                        }
                        Item { Layout.fillWidth: true }
                    }
                    Setting {
                        visible: dialog.conf !== null && dialog.conf.saver !== null
                        label: "Idle time before it starts"
                        Choice {
                            entries: dialog.withValue(dialog.saverTimes, dialog.conf ? dialog.conf.saver : null)
                            current: dialog.conf ? dialog.conf.saver : null
                            onChosen: v => dialog.change("saver", v)
                        }
                    }
                    Setting {
                        label: "Turn the screen off after"
                        Choice {
                            entries: dialog.withValue(dialog.screenTimes, dialog.conf ? dialog.conf.screenOff : null)
                            current: dialog.conf && dialog.conf.screenOff !== null ? dialog.conf.screenOff : 0
                            onChosen: v => dialog.change("screenOff", v)
                        }
                    }
                    Note {
                        visible: dialog.conf !== null && dialog.conf.saver !== null && dialog.conf.screenOff !== null && dialog.conf.screenOff <= dialog.conf.saver
                        color: Style.warn
                        text: "The screen turns off before the screensaver would start, so you'll never see it."
                    }
                    Setting {
                        label: "Suspend after"
                        Choice {
                            entries: dialog.withValue(dialog.suspendTimes, dialog.conf ? dialog.conf.suspend : null)
                            current: dialog.conf && dialog.conf.suspend !== null ? dialog.conf.suspend : 0
                            onChosen: v => dialog.change("suspend", v)
                        }
                    }
                    Setting {
                        label: "Lock with darwan (lock_cmd)"
                        Switch {
                            checked: dialog.conf !== null && dialog.conf.lockWithDarwan
                            onToggled: dialog.change("lockWithDarwan", checked)
                        }
                        Note {
                            text: dialog.conf && dialog.conf.otherLocker
                                  ? "Now: " + dialog.conf.otherLocker + ". Turning this on replaces it."
                                  : "loginctl lock-session and power menus then show your darwan theme."
                        }
                    }
                    Setting {
                        label: "Lock before sleep (before_sleep_cmd)"
                        Switch {
                            checked: dialog.conf !== null && dialog.conf.lockBeforeSleep
                            onToggled: dialog.change("lockBeforeSleep", checked)
                        }
                        Note { text: "Waking from sleep then shows the password prompt, never the screensaver or your desktop." }
                    }
                    Note {
                        visible: dialog.conf !== null && (!dialog.conf.resumed || (dialog.conf.lockWithDarwan && !dialog.conf.waitsForLock))
                        color: Style.warn
                        text: (!dialog.conf || dialog.conf.resumed ? "" : "after_sleep_cmd doesn't run `darwan resumed`, so opening a lid without touching anything can bring the screensaver straight back. ")
                              + (dialog.conf && dialog.conf.lockWithDarwan && !dialog.conf.waitsForLock ? "inhibit_sleep isn't 3, so the machine can sleep before darwan's lock is up. " : "")
                              + "Enable fixes both and keeps your other settings."
                    }
                    ActionButton {
                        visible: dialog.conf !== null && (!dialog.conf.resumed || (dialog.conf.lockWithDarwan && !dialog.conf.waitsForLock))
                        text: "Fix"
                        onActivated: dialog.change("enable", "")
                    }
                    Note {
                        text: "Saved to " + (dialog.st ? dialog.st.path : "") + (dialog.st && dialog.st.running ? "; hypridle restarts to apply each change." : ".")
                        color: Style.muted
                    }
                }

                // 3. darwan's own saver settings
                Heading { text: "SCREENSAVER" }
                Setting {
                    label: "Lock after the screensaver starts"
                    Choice {
                        entries: dialog.withValue(dialog.lockTimes, dialog.st ? dialog.st.lockAfter : null)
                        current: dialog.st ? dialog.st.lockAfter : "10"
                        onChosen: v => dialog.backend.setValueNow("saver.lock_after", v)
                    }
                }
                Note { text: "Until then any key or click takes you straight back to the desktop, without a password." }
                Setting {
                    label: "Video quality"
                    Choice {
                        id: quality
                        entries: dialog.qualities
                        current: dialog.st ? dialog.st.quality : "full"
                        onChosen: v => dialog.backend.setValueNow("saver.quality", v)
                    }
                }
                Note {
                    text: {
                        const q = dialog.qualities.find(e => dialog.st && e[0] === dialog.st.quality)
                        return q ? q[2] : ""
                    }
                }
            }
        }

        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Style.surface }

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 14
            ActionButton {
                text: "Check again"
                onActivated: dialog.checks++
            }
            ActionButton {
                text: "Preview the screensaver"
                onActivated: dialog.preview()
            }
            Item { Layout.fillWidth: true }
            ActionButton {
                text: "Close"
                primary: true
                onActivated: dialog.close()
            }
        }
    }
}
