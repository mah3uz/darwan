use std::path::Path;

use darwan_core::catalog::Catalog;
use darwan_core::lint::lint;
use darwan_core::manifest::Background;

fn catalog() -> Catalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    let (catalog, problems) = Catalog::load(&root).unwrap();
    assert!(
        problems.is_empty(),
        "themes that failed to load: {problems:#?}"
    );
    catalog
}

#[test]
fn every_shipped_theme_loads_and_passes_lint() {
    let catalog = catalog();
    // A discovery bug that finds no themes would make the lint pass vacuously.
    assert_eq!(
        catalog.themes().len(),
        40,
        "update this count when adding or removing a theme"
    );
    let failures: Vec<String> = catalog
        .themes()
        .iter()
        .flat_map(|t| lint(t).into_iter().map(move |p| format!("{}: {p}", t.id)))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn background_type_matches_whether_the_theme_ships_a_video() {
    for theme in catalog().themes() {
        let has_video = std::fs::read_dir(&theme.dir)
            .unwrap()
            .any(|e| e.unwrap().path().extension().is_some_and(|x| x == "mp4"));
        assert_eq!(
            theme.manifest.background == Background::Video,
            has_video,
            "{}: `background = \"video\"` drives the missing-multimedia-backend warning",
            theme.id
        );
    }
}

#[test]
fn every_required_font_is_git_ignored_or_openly_licensed() {
    let gitignore =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"))
            .unwrap();
    for theme in catalog().themes() {
        for font in &theme.manifest.fonts {
            let path = format!("themes/{}/font/{}", theme.id, font.file);
            let dir_glob = format!("themes/{}/font/*", theme.id);
            let ext_glob = format!("{dir_glob}.{}", font.file.rsplit('.').next().unwrap());
            let ignored = gitignore
                .lines()
                .any(|l| l == path || l == dir_glob || l == ext_glob);
            let bundled = theme.dir.join("font").join(&font.file).exists();
            assert!(
                ignored || !bundled,
                "{path} is a [[font]] the user must supply, so it must never be committed; add it to .gitignore"
            );
        }
    }
}

#[test]
fn every_vendored_theme_kit_is_identical_to_the_canonical_one() {
    let kit = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../runtime/theme-kit/darwan");
    for theme in catalog().themes() {
        let dir = theme.dir.join("darwan");
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&kit).unwrap() {
            let name = entry.unwrap().file_name();
            let copy = std::fs::read(dir.join(&name)).unwrap_or_default();
            let canonical = std::fs::read(kit.join(&name)).unwrap();
            assert!(
                copy == canonical,
                "{}/darwan/{} differs from runtime/theme-kit; run `just theme-kit`",
                theme.id,
                name.to_string_lossy()
            );
        }
    }
}
