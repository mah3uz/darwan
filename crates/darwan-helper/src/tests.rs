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

[supports]
background = true
fonts = ["text"]
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
        media: base.join("media"),
    };
    let dir = roots.themes.join("clockwork/orbital");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&roots.sddm_themes).unwrap();
    std::fs::write(dir.join("Main.qml"), "").unwrap();
    std::fs::write(dir.join("theme.conf"), "[General]\nthemeMode=dark\n").unwrap();
    std::fs::write(dir.join("darwan.toml"), MANIFEST).unwrap();
    Fixture { _tmp: tmp, roots }
}

// The helper's request: the overlay on one JSON line, with no attachments.
fn req(overlay: &[u8]) -> Vec<u8> {
    let mut r = b"{\"overlay\": ".to_vec();
    r.extend_from_slice(overlay);
    r.extend_from_slice(b"}\n");
    r
}

fn with_file(overlay: &str, key: &str, ext: &str, bytes: &[u8]) -> Vec<u8> {
    let header = format!(
        r#"{{"overlay": {overlay}, "attachments": [{{"key": "{key}", "ext": "{ext}", "size": {}}}]}}"#,
        bytes.len()
    );
    let mut r = header.into_bytes();
    r.push(b'\n');
    r.extend_from_slice(bytes);
    r
}

const MP4: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00rest of a video";

fn theme_dir(f: &Fixture) -> PathBuf {
    f.roots.themes.join("clockwork/orbital")
}

#[test]
fn apply_writes_overlay_symlink_and_conf() {
    let f = fixture();
    apply(
        &f.roots,
        "clockwork/orbital",
        &mut req(br#"{"themeMode":"light"}"#).as_slice(),
    )
    .unwrap();
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
    apply(&f.roots, "clockwork/orbital", &mut req(b"{}").as_slice()).unwrap();
    apply(&f.roots, "osu", &mut req(b"{}").as_slice()).unwrap();
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
            apply(&f.roots, id, &mut req(b"{}").as_slice())
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
    let err = apply(&f.roots, "evil", &mut req(b"{}").as_slice()).unwrap_err();
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
            apply(
                &f.roots,
                "clockwork/orbital",
                &mut req(json.as_bytes()).as_slice()
            )
            .is_err(),
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
        apply(
            &f.roots,
            "clockwork/orbital",
            &mut req(big.as_bytes()).as_slice()
        )
        .unwrap_err()
        .contains("too large")
    );
}

#[test]
fn a_real_directory_named_darwan_is_never_replaced() {
    let f = fixture();
    std::fs::create_dir_all(f.roots.sddm_themes.join("darwan")).unwrap();
    assert!(
        apply(&f.roots, "clockwork/orbital", &mut req(b"{}").as_slice())
            .unwrap_err()
            .contains("not darwan's symlink")
    );
    assert!(reset(&f.roots).unwrap().contains("left"));
    assert!(f.roots.sddm_themes.join("darwan").is_dir());
}

#[test]
fn reset_removes_everything_apply_created() {
    let f = fixture();
    apply(
        &f.roots,
        "clockwork/orbital",
        &mut req(br#"{"themeMode":"light"}"#).as_slice(),
    )
    .unwrap();
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

#[test]
fn an_attached_background_is_stored_as_root_owned_media_and_named_in_the_overlay() {
    let f = fixture();
    let r = with_file(
        r#"{"backgroundType":"video","backgroundPath":"@attachment"}"#,
        "backgroundPath",
        "mp4",
        MP4,
    );
    apply(&f.roots, "clockwork/orbital", &mut r.as_slice()).unwrap();
    let stored = f.roots.media.join("background.mp4");
    assert_eq!(std::fs::read(&stored).unwrap(), MP4);
    assert_eq!(
        std::fs::metadata(&stored).unwrap().permissions().mode() & 0o777,
        0o644
    );
    let conf = std::fs::read_to_string(theme_dir(&f).join("theme.conf.user")).unwrap();
    assert!(
        conf.contains(&format!("backgroundPath={}", stored.display())),
        "{conf}"
    );
}

#[test]
fn a_file_must_arrive_as_an_attachment_never_as_a_path_root_would_open() {
    let f = fixture();
    let r = req(br#"{"backgroundType":"image","backgroundPath":"/etc/shadow"}"#);
    let err = apply(&f.roots, "clockwork/orbital", &mut r.as_slice()).unwrap_err();
    assert!(err.contains("attachment"), "{err}");
    assert!(!theme_dir(&f).join("theme.conf.user").exists());
}

#[test]
fn a_renamed_file_of_another_kind_is_refused() {
    let f = fixture();
    let r = with_file(
        r#"{"backgroundType":"image","backgroundPath":"@attachment"}"#,
        "backgroundPath",
        "png",
        b"#!/bin/sh\necho hi\n",
    );
    assert!(
        apply(&f.roots, "clockwork/orbital", &mut r.as_slice())
            .unwrap_err()
            .contains("not a real .png")
    );
    assert!(
        !theme_dir(&f).join("theme.conf.user").exists(),
        "a bad file writes nothing"
    );
}

#[test]
fn wrong_extensions_short_input_and_oversize_claims_are_refused() {
    let f = fixture();
    for (key, ext, overlay) in [
        (
            "backgroundPath",
            "exe",
            r#"{"backgroundType":"image","backgroundPath":"@attachment"}"#,
        ),
        ("fontTextFile", "mp4", r#"{"fontTextFile":"@attachment"}"#),
        ("themeMode", "png", r#"{}"#),
    ] {
        let r = with_file(overlay, key, ext, MP4);
        assert!(
            apply(&f.roots, "clockwork/orbital", &mut r.as_slice()).is_err(),
            "{key} .{ext}"
        );
    }
    let mut short = with_file(
        r#"{"backgroundType":"video","backgroundPath":"@attachment"}"#,
        "backgroundPath",
        "mp4",
        MP4,
    );
    short.truncate(short.len() - 4);
    assert!(
        apply(&f.roots, "clockwork/orbital", &mut short.as_slice())
            .unwrap_err()
            .contains("expected")
    );
    let huge = format!(
        "{{\"overlay\": {{\"backgroundType\":\"video\",\"backgroundPath\":\"@attachment\"}}, \"attachments\": [{{\"key\": \"backgroundPath\", \"ext\": \"mp4\", \"size\": {}}}]}}\n",
        MAX_VIDEO_BYTES + 1
    );
    assert!(
        apply(&f.roots, "clockwork/orbital", &mut huge.as_bytes())
            .unwrap_err()
            .contains("MiB")
    );
}

#[test]
fn media_no_longer_used_is_removed_and_reset_removes_the_folder() {
    let f = fixture();
    let r = with_file(
        r#"{"backgroundType":"video","backgroundPath":"@attachment"}"#,
        "backgroundPath",
        "mp4",
        MP4,
    );
    apply(&f.roots, "clockwork/orbital", &mut r.as_slice()).unwrap();
    let font = with_file(
        r#"{"fontTextFile":"@attachment"}"#,
        "fontTextFile",
        "otf",
        b"OTTOfont",
    );
    apply(&f.roots, "clockwork/orbital", &mut font.as_slice()).unwrap();
    assert!(
        !f.roots.media.join("background.mp4").exists(),
        "the old background is gone"
    );
    assert!(f.roots.media.join("fontText.otf").exists());
    assert!(reset(&f.roots).unwrap().contains("media"));
    assert!(!f.roots.media.exists());
}
