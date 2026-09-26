mod bridge;
mod model;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use darwan_core::paths::Paths;

fn main() {
    let paths = Paths::detect();
    // The same environment the Quickshell hosts give themes (see qs::command in the CLI).
    unsafe {
        std::env::set_var("QML_XHR_ALLOW_FILE_READ", "1");
    }
    if let Err(e) = std::env::set_current_dir(paths.runtime()) {
        eprintln!("darwan-gui: {}: {e}", paths.runtime().display());
    }

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        engine.as_mut().add_import_path(&QString::from(
            paths.runtime().join("imports").display().to_string(),
        ));
        engine.load(&QUrl::from("qrc:/qt/qml/org/darwan/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
