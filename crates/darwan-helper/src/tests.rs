use super::*;

const MANIFEST: &str = r#"name = "Orbital"
author = "a"
background = "color"

[[font]]
file = "Andy Bold.ttf"
family = "Andy Bold"
license = "Commercial"

[[option]]
key = "themeMode"
label = "Appearance"
type = "enum"
choices = [{ value = "dark", label = "Dark" }, { value = "light", label = "Light" }]
"#;

struct Fixture {
    _tmp: tempfile::TempDir,
    roots: Roots,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().canonicalize().unwrap();
    let roots = Roots {
        themes: base.join("themes"),
        sddm_themes: base.join("sddm"),
        conf_d: base.join("conf.d"),
    };
    let dir = roots.themes.join("clockwork/orbital");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&roots.sddm_themes).unwrap();
    std::fs::write(dir.join("Main.qml"), "").unwrap();
    std::fs::write(dir.join("theme.conf"), "[General]\nthemeMode=dark\n").unwrap();
    std::fs::write(dir.join("darwan.toml"), MANIFEST).unwrap();
    Fixture { _tmp: tmp, roots }
}

fn theme_dir(f: &Fixture) -> PathBuf {
    f.roots.themes.join("clockwork/orbital")
}

#[test]
fn apply_writes_overlay_symlink_and_conf() {
    let f = fixture();
    apply(&f.roots, "clockwork/orbital", br#"{"themeMode":"light"}"#).unwrap();
    assert_eq!(
        std::fs::read_to_string(theme_dir(&f).join("theme.conf.user")).unwrap(),
        "[General]\nthemeMode=light\n"
    );
    assert_eq!(
        std::fs::read_link(f.roots.sddm_themes.join("darwan")).unwrap(),
        theme_dir(&f)
    );
    assert_eq!(
        std::fs::read_to_string(f.roots.conf_d.join("zz-darwan.conf")).unwrap(),
        "[Theme]\nCurrent=darwan\n"
    );
    let mode = std::fs::metadata(theme_dir(&f).join("theme.conf.user"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o644, "the sddm user must be able to read it");
}

#[test]
fn applying_again_repoints_the_symlink() {
    let f = fixture();
    let other = f.roots.themes.join("osu");
    std::fs::create_dir_all(&other).unwrap();
    for file in ["Main.qml", "theme.conf", "darwan.toml"] {
        std::fs::copy(theme_dir(&f).join(file), other.join(file)).unwrap();
    }
    apply(&f.roots, "clockwork/orbital", b"{}").unwrap();
    apply(&f.roots, "osu", b"{}").unwrap();
    assert_eq!(
        std::fs::read_link(f.roots.sddm_themes.join("darwan")).unwrap(),
        other
    );
}

#[test]
fn traversal_and_absolute_ids_are_rejected_before_touching_the_filesystem() {
    let f = fixture();
    for id in [
        "../../etc",
        "/etc",
        "clockwork/../../etc",
        "Clockwork/Orbital",
        "",
    ] {
        assert!(
            apply(&f.roots, id, b"{}")
                .unwrap_err()
                .contains("invalid theme id"),
            "{id:?}"
        );
    }
    assert!(!f.roots.conf_d.exists());
}

#[test]
fn a_theme_reached_through_a_symlink_is_rejected() {
    let f = fixture();
    let outside = f.roots.themes.parent().unwrap().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    for file in ["Main.qml", "theme.conf", "darwan.toml"] {
        std::fs::copy(theme_dir(&f).join(file), outside.join(file)).unwrap();
    }
    std::os::unix::fs::symlink(&outside, f.roots.themes.join("evil")).unwrap();
    let err = apply(&f.roots, "evil", b"{}").unwrap_err();
    assert!(err.contains("through a symlink"), "{err}");
    assert!(
        !outside.join("theme.conf.user").exists(),
        "root must not write outside the themes root"
    );
}

#[test]
fn unknown_keys_bad_values_and_injection_are_rejected() {
    let f = fixture();
    for json in [
        r#"{"background":"/etc/shadow"}"#,
        r#"{"themeMode":"blue"}"#,
        r#"{"themeMode":"dark\n[Theme]\nCurrent=evil"}"#,
        r#"{"themeMode":["dark"]}"#,
        "not json",
    ] {
        assert!(
            apply(&f.roots, "clockwork/orbital", json.as_bytes()).is_err(),
            "{json}"
        );
    }
    assert!(!theme_dir(&f).join("theme.conf.user").exists());
    assert!(!f.roots.conf_d.join("zz-darwan.conf").exists());
}

#[test]
fn an_oversized_overlay_is_rejected() {
    let f = fixture();
    let big = format!(r#"{{"themeMode":"{}"}}"#, "x".repeat(MAX_OVERLAY_BYTES));
    assert!(
        apply(&f.roots, "clockwork/orbital", big.as_bytes())
            .unwrap_err()
            .contains("too large")
    );
}

#[test]
fn a_real_directory_named_darwan_is_never_replaced() {
    let f = fixture();
    std::fs::create_dir_all(f.roots.sddm_themes.join("darwan")).unwrap();
    assert!(
        apply(&f.roots, "clockwork/orbital", b"{}")
            .unwrap_err()
            .contains("not darwan's symlink")
    );
    assert!(reset(&f.roots).unwrap().contains("left"));
    assert!(f.roots.sddm_themes.join("darwan").is_dir());
}

#[test]
fn reset_removes_everything_apply_created() {
    let f = fixture();
    apply(&f.roots, "clockwork/orbital", br#"{"themeMode":"light"}"#).unwrap();
    reset(&f.roots).unwrap();
    assert!(
        f.roots
            .sddm_themes
            .join("darwan")
            .symlink_metadata()
            .is_err()
    );
    assert!(!f.roots.conf_d.join("zz-darwan.conf").exists());
    assert!(!theme_dir(&f).join("theme.conf.user").exists());
    assert_eq!(reset(&f.roots).unwrap(), "nothing to reset");
}

#[test]
fn font_import_accepts_only_a_declared_real_font() {
    let f = fixture();
    let ttf = [&[0u8, 1, 0, 0][..], &[0u8; 64][..]].concat();
    assert!(
        import_font(&f.roots, "clockwork/orbital", "evil.ttf", &ttf)
            .unwrap_err()
            .contains("does not need")
    );
    assert!(import_font(&f.roots, "clockwork/orbital", "../../etc/x", &ttf).is_err());
    assert!(
        import_font(
            &f.roots,
            "clockwork/orbital",
            "Andy Bold.ttf",
            b"root:x:0:0"
        )
        .unwrap_err()
        .contains("not a TrueType")
    );
    let huge = vec![0u8; MAX_FONT_BYTES + 1];
    assert!(import_font(&f.roots, "clockwork/orbital", "Andy Bold.ttf", &huge).is_err());
    import_font(&f.roots, "clockwork/orbital", "Andy Bold.ttf", &ttf).unwrap();
    let installed = theme_dir(&f).join("font/Andy Bold.ttf");
    assert_eq!(std::fs::read(&installed).unwrap(), ttf);
    assert_eq!(
        std::fs::metadata(installed).unwrap().permissions().mode() & 0o777,
        0o644
    );
}

#[test]
fn font_import_refuses_a_font_dir_that_is_a_symlink() {
    let f = fixture();
    let elsewhere = f.roots.themes.parent().unwrap().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, theme_dir(&f).join("font")).unwrap();
    let ttf = [0u8, 1, 0, 0, 9, 9];
    assert!(import_font(&f.roots, "clockwork/orbital", "Andy Bold.ttf", &ttf).is_err());
    assert!(!elsewhere.join("Andy Bold.ttf").exists());
}
