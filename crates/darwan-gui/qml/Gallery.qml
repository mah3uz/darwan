import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import org.darwan

Rectangle {
    id: gallery

    required property Backend backend
    property string currentId: ""
    readonly property alias searchField: search
    readonly property alias list: list

    color: Style.mantle

    // Updated in place so a settings change doesn't reset the scroll position.
    function sync() {
        const rows = JSON.parse(backend.rows(search.text)).map(r => ({
            kind: r.kind,
            title: r.title || "",
            count: r.count || 0,
            themeId: r.id || "",
            name: r.name || "",
            background: r.background || "",
            preview: r.preview || "",
            isLock: r.isLock || false,
            isSddm: r.isSddm || false,
            missingFonts: r.missingFonts || 0,
        }))
        if (rows.length !== rowModel.count) {
            rowModel.clear()
            rows.forEach(r => rowModel.append(r))
        } else {
            rows.forEach((r, i) => rowModel.set(i, r))
        }
        const ids = rows.filter(r => r.kind === "theme").map(r => r.themeId)
        if (ids.length > 0 && !ids.includes(currentId))
            currentId = ids[0]
        themeCount = ids.length
    }

    property int themeCount: 0
    property int totalCount: 0

    function step(delta) {
        let i = -1
        for (let j = 0; j < rowModel.count; j++)
            if (rowModel.get(j).themeId === currentId) i = j
        for (let j = i + delta; j >= 0 && j < rowModel.count; j += delta) {
            if (rowModel.get(j).kind === "theme") {
                currentId = rowModel.get(j).themeId
                list.positionViewAtIndex(j, ListView.Contain)
                return
            }
        }
    }

    Component.onCompleted: {
        sync()
        totalCount = themeCount
    }
    Connections {
        target: gallery.backend
        function onRevisionChanged() { gallery.sync() }
    }

    ListModel { id: rowModel }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Style.gap
        spacing: Style.gap

        RowLayout {
            Layout.fillWidth: true
            Label {
                text: "Themes"
                color: Style.accent
                font.bold: true
                font.pixelSize: 16
                Layout.fillWidth: true
            }
            Label {
                text: search.text === "" ? gallery.totalCount : gallery.themeCount + " of " + gallery.totalCount
                color: Style.muted
            }
        }

        TextField {
            id: search
            Layout.fillWidth: true
            placeholderText: "Search by name or id  (Ctrl+F)"
            color: Style.text
            placeholderTextColor: Style.muted
            background: Rectangle {
                radius: 6
                color: Style.base
                border.color: search.activeFocus ? Style.accent : Style.border
            }
            onTextChanged: gallery.sync()
            Keys.onDownPressed: gallery.step(1)
            Keys.onUpPressed: gallery.step(-1)
            Keys.onEscapePressed: text = ""
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: rowModel
            spacing: 2
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}
            activeFocusOnTab: true
            focus: true
            Keys.onDownPressed: gallery.step(1)
            Keys.onUpPressed: gallery.step(-1)

            delegate: Loader {
                required property var model
                width: ListView.view.width - 8
                sourceComponent: model.kind === "section" ? sectionRow : themeRow

                Component {
                    id: sectionRow
                    Label {
                        topPadding: 12
                        bottomPadding: 4
                        leftPadding: 4
                        text: model.title.toUpperCase() + "  ·  " + model.count
                        color: Style.muted
                        font.pixelSize: 11
                        font.bold: true
                        font.letterSpacing: 1
                    }
                }

                Component {
                    id: themeRow
                    Rectangle {
                        id: card
                        readonly property bool selected: model.themeId === gallery.currentId
                        height: 64
                        radius: 6
                        color: selected ? Style.surface : hover.hovered ? Qt.alpha(Style.surface, 0.5) : "transparent"
                        border.color: selected ? Style.accent : "transparent"

                        HoverHandler { id: hover }
                        TapHandler {
                            onTapped: {
                                gallery.currentId = model.themeId
                                list.forceActiveFocus()
                            }
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 6
                            spacing: 10

                            Rectangle {
                                Layout.preferredWidth: 92
                                Layout.fillHeight: true
                                radius: 4
                                color: Style.crust
                                clip: true
                                Image {
                                    anchors.fill: parent
                                    source: model.preview === "" ? "" : "file://" + model.preview
                                    sourceSize.width: 184
                                    fillMode: Image.PreserveAspectCrop
                                    asynchronous: true
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 4
                                Label {
                                    Layout.fillWidth: true
                                    text: model.name
                                    color: Style.text
                                    elide: Text.ElideRight
                                    font.bold: card.selected
                                }
                                Row {
                                    spacing: 4
                                    Tag { visible: model.isLock; text: "LOCK"; tint: Style.lock }
                                    Tag { visible: model.isSddm; text: "LOGIN"; tint: Style.sddm }
                                    Tag { visible: model.background === "video"; text: "VIDEO"; tint: Style.muted }
                                    Tag {
                                        visible: model.missingFonts > 0
                                        text: model.missingFonts + (model.missingFonts === 1 ? " FONT" : " FONTS") + " MISSING"
                                        tint: Style.warn
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
