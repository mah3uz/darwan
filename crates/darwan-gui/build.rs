use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    let qml = QmlModule::new("org.darwan").qml_files([
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/Gallery.qml"),
        QmlFile::from("qml/LivePreview.qml"),
        QmlFile::from("qml/SettingsForm.qml"),
        QmlFile::from("qml/ActionButton.qml"),
        QmlFile::from("qml/Tag.qml"),
        QmlFile::from("qml/StatusBadge.qml"),
        QmlFile::from("qml/ReportDialog.qml"),
        QmlFile::from("qml/UnsavedDialog.qml"),
    ]);
    CxxQtBuilder::new_qml_module(qml)
        .files(["src/bridge.rs"])
        .build();
}
