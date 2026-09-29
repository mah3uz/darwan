use std::path::PathBuf;
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QResource, QResources, QmlFile, QmlModule};
use qt_build_utils::QResourceFile;

// Qt Quick takes shaders precompiled for every graphics API; qsb (qt6-shadertools) makes them at build time.
fn shader(name: &str) -> PathBuf {
    let src = PathBuf::from("shaders").join(name);
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(format!("{name}.qsb"));
    println!("cargo::rerun-if-changed={}", src.display());
    let qsb = ["/usr/lib/qt6/bin/qsb", "qsb6", "qsb"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("qsb not found: install qt6-shadertools to build the GUI");
    let status = Command::new(qsb)
        .args([
            "--qt6",
            "--glsl",
            "100es,120,150",
            "--hlsl",
            "50",
            "--msl",
            "12",
            "-o",
        ])
        .arg(&out)
        .arg(&src)
        .status()
        .expect("cannot run qsb");
    assert!(status.success(), "qsb failed on {}", src.display());
    out
}

fn main() {
    let qml = QmlModule::new("org.darwan").qml_files([
        QmlFile::from("qml/Style.qml").singleton(true),
        QmlFile::from("qml/ActionButton.qml"),
        QmlFile::from("qml/App.qml"),
        QmlFile::from("qml/ColorPopover.qml"),
        QmlFile::from("qml/ColorWell.qml"),
        QmlFile::from("qml/FadeImage.qml"),
        QmlFile::from("qml/FieldControl.qml"),
        QmlFile::from("qml/GateCard.qml"),
        QmlFile::from("qml/GlassEdge.qml"),
        QmlFile::from("qml/Icon.qml"),
        QmlFile::from("qml/Inspector.qml"),
        QmlFile::from("qml/LivePreview.qml"),
        QmlFile::from("qml/LookRow.qml"),
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/Mark.qml"),
        QmlFile::from("qml/MenuButton.qml"),
        QmlFile::from("qml/MenuRow.qml"),
        QmlFile::from("qml/Pill.qml"),
        QmlFile::from("qml/Popover.qml"),
        QmlFile::from("qml/ReportDialog.qml"),
        QmlFile::from("qml/Rounded.qml"),
        QmlFile::from("qml/RoundedImage.qml"),
        QmlFile::from("qml/SaverPanel.qml"),
        QmlFile::from("qml/Segmented.qml"),
        QmlFile::from("qml/SettingGroup.qml"),
        QmlFile::from("qml/SettingRow.qml"),
        QmlFile::from("qml/SettingsPopover.qml"),
        QmlFile::from("qml/SliderField.qml"),
        QmlFile::from("qml/SmoothWheel.qml"),
        QmlFile::from("qml/Spinner.qml"),
        QmlFile::from("qml/Stage.qml"),
        QmlFile::from("qml/StatusBadge.qml"),
        QmlFile::from("qml/Tag.qml"),
        QmlFile::from("qml/ThemeCard.qml"),
        QmlFile::from("qml/Toast.qml"),
        QmlFile::from("qml/Toggle.qml"),
        QmlFile::from("qml/UnsavedDialog.qml"),
        QmlFile::from("qml/UseMenu.qml"),
        QmlFile::from("qml/Wall.qml"),
        QmlFile::from("qml/WallRow.qml"),
        QmlFile::from("qml/WallLibrary.qml"),
        QmlFile::from("qml/WallHome.qml"),
        QmlFile::from("qml/WallGrid.qml"),
        QmlFile::from("qml/WallExplore.qml"),
        QmlFile::from("qml/WallFeed.qml"),
        QmlFile::from("qml/WallCategory.qml"),
        QmlFile::from("qml/Chip.qml"),
        QmlFile::from("qml/WallpaperPage.qml"),
        QmlFile::from("qml/WallpaperDetail.qml"),
        QmlFile::from("qml/WallCard.qml"),
        QmlFile::from("qml/PageSwitch.qml"),
    ]);
    let rounded = shader("rounded.frag");
    CxxQtBuilder::new_qml_module(qml)
        .files(["src/bridge.rs"])
        .qrc_resources(QResources::new().resource(
            QResource::new().file(QResourceFile::new(rounded).alias("qml/rounded.frag.qsb")),
        ))
        .build();
}
