use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use darwan_core::ini;

fn qmltestrunner() -> PathBuf {
    let bundled = Path::new("/usr/lib/qt6/bin/qmltestrunner");
    if bundled.exists() {
        bundled.to_path_buf()
    } else {
        PathBuf::from("qmltestrunner")
    }
}

#[test]
fn runtime_qml_tests_pass() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests = manifest.join("tests/qml");
    let generated = tests.join("generated");
    std::fs::create_dir_all(&generated).unwrap();

    // Values chosen to hit every escaping rule in the Rust writer.
    let values: BTreeMap<String, String> = [
        ("plain", "game"),
        ("comma", "ddd, MMM d"),
        ("quote", "say \"hi\""),
        ("backslash", "C:\\path"),
        ("padded", "  spaced  "),
        ("at", "@Variant(AAAA)"),
        ("hash", "#1a1a4a"),
        ("equals", "a=b"),
        ("unicode", "দারোয়ান"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    std::fs::write(
        generated.join("overlay.conf"),
        ini::write_general(&values).unwrap(),
    )
    .unwrap();
    std::fs::write(
        generated.join("expected.js"),
        format!(
            ".pragma library\nvar values = {};\n",
            serde_json::to_string(&values).unwrap()
        ),
    )
    .unwrap();

    image::RgbaImage::from_pixel(8, 8, image::Rgba([40, 80, 160, 255]))
        .save(generated.join("bg.png"))
        .unwrap();

    let out = Command::new(qmltestrunner())
        .arg("-input")
        .arg(&tests)
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QML_XHR_ALLOW_FILE_READ", "1")
        .env("QML2_IMPORT_PATH", manifest.join("../../runtime/imports"))
        .output()
        .expect("qmltestrunner is part of qt6-declarative");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{log}");
    assert!(log.contains("0 failed"), "{log}");
}
