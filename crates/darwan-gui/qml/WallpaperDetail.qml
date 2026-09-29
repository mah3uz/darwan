pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import org.darwan

// One wallpaper, full-window, with a bar to set it: on every screen or the ones chosen, then on the lockscreen too.
FocusScope {
    id: detail

    required property Backend backend
    property var item: null
    // "library" or "online"
    property string mode: "library"
    property bool shown: false
    readonly property var owner: backend.wallRevision >= 0 ? JSON.parse(backend.wallOwner()) : null
    readonly property var screens: Qt.application.screens
    // The notice beside the bar: what is happening, then how it went.
    property string notice: ""
    property string noticeKind: ""
    property string appliedPath: ""
    property string lockNote: ""

    function open(it, m) {
        item = it
        mode = m
        notice = ""
        noticeKind = ""
        lockNote = ""
        shown = true
        forceActiveFocus()
    }
    function close() {
        shown = false
    }
    function screenName(s) {
        const label = [s.manufacturer, s.model].filter(x => x && x.trim()).join(" ").trim()
        return label || s.name
    }
    function apply(outputs, restart) {
        const all = []
        for (let i = 0; i < screens.length; i++)
            all.push(screens[i].name)
        const req = mode === "online" ? { online: item.id } : { path: item.path }
        req.outputs = outputs
        req.all = all
        req.restart = restart === true
        notice = mode === "online" && !item.saved ? "Downloading…" : "Setting…"
        noticeKind = "busy"
        backend.wallApply(JSON.stringify(req))
    }
    function setClicked() {
        if (screens.length > 1 && owner && owner.perOutput) {
            chooser.parent = setButton
            chooser.x = (setButton.width - chooser.width) / 2
            chooser.y = -chooser.height - 14
            chooser.open()
        } else {
            apply([])
        }
    }

    visible: opacity > 0
    opacity: shown ? 1 : 0
    Behavior on opacity { NumberAnimation { duration: Style.medium; easing.type: Style.ease } }
    Keys.onEscapePressed: close()

    Connections {
        target: detail.backend
        function onWallProgress(text, image) {
            if (!detail.shown)
                return
            detail.notice = text
            detail.noticeKind = "busy"
        }
        function onWallDone(ok, text, path) {
            if (!detail.shown)
                return
            detail.notice = ok ? "Wallpaper applied" : text
            detail.noticeKind = ok ? "done" : "fail"
            detail.appliedPath = path
        }
        function onWallConsent(tool, request) {
            if (!detail.shown)
                return
            detail.notice = ""
            consent.tool = tool
            consent.request = request
            consent.open()
        }
    }

    Rectangle { anchors.fill: parent; color: "black" }
    Image {
        id: preview
        anchors.fill: parent
        source: !detail.item ? "" : detail.mode === "online" ? detail.item.thumb : (detail.item.kind === "video" ? detail.item.thumb : detail.item.file)
        sourceSize.width: Math.min(3840, detail.width * Screen.devicePixelRatio)
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        opacity: status === Image.Ready ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Style.slow } }
    }
    // The grid's picture stands in while the full one loads.
    Image {
        anchors.fill: parent
        visible: preview.status !== Image.Ready && detail.item !== null
        source: detail.item ? detail.item.thumb : ""
        fillMode: Image.PreserveAspectCrop
    }
    TapHandler { onTapped: detail.close() }

    ActionButton {
        x: 20
        y: 20
        glyph: "left"
        tip: "Back (Esc)"
        onActivated: detail.close()
    }

    // The bar, as the Stage's: title and facts, credit, and Set.
    Rectangle {
        id: bar
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 28
        width: barRow.width + 28
        height: 64
        radius: 32
        color: Style.panel
        border.color: Style.panelBorder
        GlassEdge {}
        TapHandler {}

        Row {
            id: barRow
            x: 22
            anchors.verticalCenter: parent.verticalCenter
            spacing: 16
            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2
                Label {
                    text: !detail.item ? "" : detail.mode === "online" ? detail.item.title : detail.item.name
                    width: Math.min(implicitWidth, 360)
                    elide: Text.ElideRight
                    font.family: Style.family
                    font.pixelSize: Style.title
                    font.weight: Font.DemiBold
                    color: Style.text
                }
                Label {
                    text: !detail.item ? "" : [detail.item.size, detail.item.bytes, detail.item.credit ? detail.item.credit.author : ""].filter(s => s).join("  ·  ")
                    width: Math.min(implicitWidth, 360)
                    elide: Text.ElideRight
                    font.family: Style.family
                    font.pixelSize: Style.caption
                    color: Style.sub
                }
            }
            ActionButton {
                anchors.verticalCenter: parent.verticalCenter
                visible: detail.item !== null && detail.item.credit !== null && detail.item.credit !== undefined
                glyph: "external"
                flat: true
                text: visible ? detail.item.credit.licence.length > 28 ? detail.item.credit.source : detail.item.credit.licence : ""
                tip: visible ? detail.item.credit.licence + "\n" + detail.item.credit.page : ""
                onActivated: Qt.openUrlExternally(detail.item.credit.page)
            }
            ActionButton {
                id: setButton
                anchors.verticalCenter: parent.verticalCenter
                text: detail.backend.wallBusy ? "Setting…" : "Set Wallpaper"
                primary: true
                pill: true
                reason: detail.owner && !detail.owner.found ? detail.owner.note
                        : detail.item && detail.item.kind === "video" && detail.owner && !detail.owner.video ? detail.owner.name + " can't show videos" : ""
                enabled: !detail.backend.wallBusy
                onActivated: detail.setClicked()
            }
        }
    }

    // What happened, beside the bar; after a wallpaper is set it offers the lockscreen too.
    Rectangle {
        anchors.left: bar.right
        anchors.leftMargin: 12
        anchors.verticalCenter: bar.verticalCenter
        visible: detail.notice !== ""
        width: noticeRow.width + 28
        height: 50
        radius: 25
        color: Style.panel
        border.color: detail.noticeKind === "fail" ? Qt.alpha(Style.danger, 0.6) : Style.panelBorder
        GlassEdge {}
        TapHandler {}
        Row {
            id: noticeRow
            x: 14
            anchors.verticalCenter: parent.verticalCenter
            spacing: 10
            Spinner {
                anchors.verticalCenter: parent.verticalCenter
                visible: detail.noticeKind === "busy"
                color: Style.sub
            }
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                visible: detail.noticeKind === "done" || detail.noticeKind === "fail"
                name: detail.noticeKind === "done" ? "check" : "warn"
                color: detail.noticeKind === "done" ? Style.ok : Style.danger
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: detail.lockNote !== "" ? detail.lockNote : detail.notice
                width: Math.min(implicitWidth, 320)
                elide: Text.ElideRight
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.text
            }
            ActionButton {
                anchors.verticalCenter: parent.verticalCenter
                visible: detail.noticeKind === "done" && detail.lockNote === ""
                text: "Use on lockscreen too"
                glyph: "lock"
                pill: true
                onActivated: detail.lockNote = detail.backend.wallLockToo(detail.appliedPath)
            }
        }
    }

    // Which screens, as wallspace asks it: each display by its name, or all of them.
    Popover {
        id: chooser
        width: Math.max(260, choices.width + 24)
        contentItem: Column {
            spacing: 10
            Item {
                width: choices.width
                height: 26
                Label {
                    x: 6
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Choose display"
                    font.family: Style.family
                    font.pixelSize: Style.body
                    font.weight: Font.DemiBold
                    color: Style.text
                }
                ActionButton {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "All"
                    pill: true
                    compact: true
                    onActivated: {
                        chooser.close()
                        detail.apply([])
                    }
                }
            }
            Row {
                id: choices
                spacing: 10
                Repeater {
                    model: detail.screens
                    delegate: AbstractButton {
                        id: screenChoice
                        required property var modelData
                        width: 150
                        height: 110
                        hoverEnabled: true
                        onClicked: {
                            chooser.close()
                            detail.apply([modelData.name])
                        }
                        background: Rectangle {
                            radius: Style.radius
                            color: screenChoice.hovered ? Style.controlHover : Style.group
                            border.color: Style.sep
                        }
                        contentItem: Column {
                            topPadding: 16
                            spacing: 6
                            Icon { anchors.horizontalCenter: parent.horizontalCenter; name: "display"; size: 28; color: Style.sub }
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                width: 134
                                horizontalAlignment: Text.AlignHCenter
                                elide: Text.ElideRight
                                text: detail.screenName(screenChoice.modelData)
                                font.family: Style.family
                                font.pixelSize: Style.body
                                font.weight: Font.DemiBold
                                color: Style.text
                            }
                            Label {
                                anchors.horizontalCenter: parent.horizontalCenter
                                text: screenChoice.modelData.name + " · " + Math.round(screenChoice.modelData.width * screenChoice.modelData.devicePixelRatio) + "×" + Math.round(screenChoice.modelData.height * screenChoice.modelData.devicePixelRatio)
                                font.family: Style.family
                                font.pixelSize: 11
                                color: Style.sub
                            }
                        }
                    }
                }
            }
        }
    }

    // A tool that changes only by being restarted, and Darwan didn't start it: the user says so once or for good.
    Popup {
        id: consent
        property string tool: ""
        property string request: ""
        anchors.centerIn: parent
        modal: true
        width: 420
        padding: 22
        background: Rectangle {
            radius: Style.radius + 4
            color: Style.popover
            border.color: Style.panelBorder
        }
        contentItem: Column {
            spacing: 14
            Label {
                width: 376
                text: "Restart " + consent.tool + "?"
                font.family: Style.family
                font.pixelSize: Style.title
                font.weight: Font.DemiBold
                color: Style.text
            }
            Label {
                width: 376
                wrapMode: Text.WordWrap
                text: consent.tool + " can only change the wallpaper by being restarted, and Darwan didn't start it. Darwan starts a new one with the new picture, then stops the old one; your other settings for it are kept."
                font.family: Style.family
                font.pixelSize: Style.body
                color: Style.sub
            }
            Row {
                anchors.right: parent.right
                spacing: 8
                ActionButton {
                    text: "Not now"
                    onActivated: {
                        consent.close()
                        detail.notice = ""
                    }
                }
                ActionButton {
                    text: "Just this time"
                    onActivated: {
                        consent.close()
                        const r = JSON.parse(consent.request)
                        detail.apply(r.outputs, true)
                    }
                }
                ActionButton {
                    text: "Always allow"
                    primary: true
                    onActivated: {
                        consent.close()
                        detail.backend.wallAllowRestart(consent.tool)
                        const r = JSON.parse(consent.request)
                        detail.apply(r.outputs, true)
                    }
                }
            }
        }
    }
}
