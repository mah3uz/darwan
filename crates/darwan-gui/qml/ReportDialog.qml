import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.darwan

// Opens as soon as a report command starts, so the click visibly does something, then shows the result.
Popup {
    id: dialog

    required property Backend backend
    property string title: ""
    property string command: ""
    property string runningText: ""
    property bool running: false
    property bool succeeded: true
    property var rows: []

    readonly property int okCount: rows.filter(r => r.kind === "ok").length
    readonly property int warnCount: rows.filter(r => r.kind === "warn").length
    readonly property int failCount: rows.filter(r => r.kind === "fail").length
    readonly property string verdict: !succeeded || failCount > 0 ? "fail" : warnCount > 0 ? "warn" : "ok"

    function start(title, command, runningText) {
        dialog.title = title
        dialog.command = command
        dialog.runningText = runningText
        dialog.rows = []
        dialog.running = true
        open()
    }

    function finish(title, succeeded, command, output) {
        dialog.title = title
        dialog.command = command
        dialog.succeeded = succeeded
        const rows = JSON.parse(backend.report(output))
        dialog.rows = rows.length > 0 ? rows : [{ kind: "text", text: "(no output)" }]
        dialog.running = false
        open()
    }

    parent: Overlay.overlay
    anchors.centerIn: parent
    width: Math.min(Math.max(parent.width * 0.55, 560), 900, parent.width - 64)
    height: Math.min(implicitHeight, parent.height * 0.85)
    modal: true
    padding: 0
    closePolicy: Popup.CloseOnEscape
    focus: true
    onOpened: closeButton.forceActiveFocus()

    Overlay.modal: Rectangle { color: Qt.alpha(Style.crust, 0.65) }

    background: Rectangle {
        radius: 14
        color: Style.mantle
        border.color: Style.border
    }

    enter: Transition { NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120 } }
    exit: Transition { NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 90 } }

    contentItem: ColumnLayout {
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 20
            spacing: 14

            Item {
                implicitWidth: 40
                implicitHeight: 40
                BusyIndicator {
                    anchors.fill: parent
                    visible: dialog.running
                    running: dialog.running
                }
                StatusBadge {
                    anchors.fill: parent
                    visible: !dialog.running
                    size: 40
                    kind: dialog.verdict
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Label {
                    Layout.fillWidth: true
                    text: dialog.title
                    color: Style.text
                    font.pixelSize: 17
                    font.bold: true
                    elide: Text.ElideRight
                }
                Label {
                    Layout.fillWidth: true
                    text: "$ " + dialog.command
                    color: Style.muted
                    font.family: "monospace"
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }
            }

            Row {
                visible: !dialog.running
                spacing: 6
                Tag { visible: dialog.okCount > 0; text: dialog.okCount + " OK"; tint: Style.ok }
                Tag { visible: dialog.warnCount > 0; text: dialog.warnCount + (dialog.warnCount === 1 ? " WARNING" : " WARNINGS"); tint: Style.warn }
                Tag { visible: dialog.failCount > 0; text: dialog.failCount + " FAILED"; tint: Style.error }
            }
        }

        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Style.surface }

        Label {
            visible: dialog.running
            Layout.fillWidth: true
            Layout.margins: 24
            text: dialog.runningText
            color: Style.subtext
            wrapMode: Text.WordWrap
        }

        Flickable {
            id: body
            visible: !dialog.running
            Layout.fillWidth: true
            Layout.fillHeight: true
            implicitHeight: rowsColumn.implicitHeight + 32
            contentHeight: rowsColumn.implicitHeight + 32
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}

            Column {
                id: rowsColumn
                x: 20
                y: 16
                width: body.width - 40
                spacing: 8

                Repeater {
                    model: dialog.rows
                    delegate: Loader {
                        required property var modelData
                        width: rowsColumn.width
                        sourceComponent: modelData.kind === "heading" ? heading
                                       : modelData.kind === "detail" ? detail
                                       : modelData.kind === "text" ? plain
                                       : status

                        Component {
                            id: heading
                            Label {
                                topPadding: 8
                                text: modelData.text.toUpperCase()
                                color: Style.muted
                                font.pixelSize: 11
                                font.bold: true
                                font.letterSpacing: 1
                            }
                        }
                        Component {
                            id: status
                            RowLayout {
                                spacing: 10
                                StatusBadge {
                                    kind: modelData.kind
                                    Layout.alignment: Qt.AlignTop
                                }
                                Label {
                                    Layout.fillWidth: true
                                    text: modelData.text
                                    color: Style.text
                                    wrapMode: Text.Wrap
                                }
                            }
                        }
                        Component {
                            id: detail
                            Label {
                                leftPadding: 30
                                text: modelData.text
                                color: Style.subtext
                                font.family: "monospace"
                                font.pixelSize: 12
                                wrapMode: Text.WrapAnywhere
                            }
                        }
                        Component {
                            id: plain
                            Label {
                                topPadding: 4
                                text: modelData.text
                                color: Style.subtext
                                wrapMode: Text.Wrap
                            }
                        }
                    }
                }
            }
        }

        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Style.surface }

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 14
            Label {
                Layout.fillWidth: true
                visible: dialog.running
                text: "Hiding this keeps the command running; its result opens here when it's done."
                color: Style.muted
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }
            Item { Layout.fillWidth: true; visible: !dialog.running }
            ActionButton {
                id: closeButton
                text: dialog.running ? "Hide" : "Close"
                primary: !dialog.running
                onActivated: dialog.close()
            }
        }
    }
}
