use darwan_core::form::{Field, FieldKind};

use crate::session::WaylandSession;

pub struct Environment {
    pub wayland: Result<(), String>,
    pub sddm: Result<(), String>,
    pub helper: Result<(), String>,
}

impl Environment {
    pub fn detect() -> Self {
        let on_path = |p: &str| {
            std::env::var_os("PATH")
                .is_some_and(|path| std::env::split_paths(&path).any(|d| d.join(p).is_file()))
        };
        let sddm: Result<(), String> = if on_path("sddm-greeter-qt6") {
            Ok(())
        } else {
            Err("SDDM (Qt 6) is not installed".into())
        };
        let helper = match &sddm {
            Err(e) => Err(e.clone()),
            Ok(()) if std::path::Path::new(crate::sddm::HELPER).is_file() => Ok(()),
            Ok(()) => Err("darwan-helper is not installed".into()),
        };
        Self {
            wayland: WaylandSession::discover().map(drop),
            sddm,
            helper,
        }
    }
}

pub fn display_name(theme: &darwan_core::catalog::Theme) -> String {
    match &theme.manifest.family {
        Some(f) => format!("{f} · {}", theme.manifest.name),
        None => theme.manifest.name.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListRow {
    Section { title: String, count: usize },
    Theme(usize),
}

pub const OTHER_SECTION: &str = "Other themes";

// Families (Clockwork, Pixel) get their own sections; everything else goes last, all by name.
pub fn matches(theme: &darwan_core::catalog::Theme, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || display_name(theme).to_lowercase().contains(&q) || theme.id.contains(&q)
}

pub fn list_rows(catalog: &darwan_core::catalog::Catalog, query: &str) -> Vec<ListRow> {
    let themes = catalog.themes();
    let mut groups: std::collections::BTreeMap<(bool, String), Vec<usize>> = Default::default();
    for (i, t) in themes.iter().enumerate().filter(|(_, t)| matches(t, query)) {
        let key = match &t.manifest.family {
            Some(f) => (false, f.to_lowercase()),
            None => (true, String::new()),
        };
        groups.entry(key).or_default().push(i);
    }
    let mut rows = Vec::new();
    for ((other, _), mut members) in groups {
        members.sort_by_key(|&i| themes[i].manifest.name.to_lowercase());
        let title = if other {
            OTHER_SECTION.to_string()
        } else {
            themes[members[0]]
                .manifest
                .family
                .clone()
                .unwrap_or_default()
        };
        rows.push(ListRow::Section {
            title,
            count: members.len(),
        });
        rows.extend(members.into_iter().map(ListRow::Theme));
    }
    rows
}

// The value one step along a choice or boolean, or None for kinds edited as text.
pub fn step(field: &Field, forward: bool) -> Option<String> {
    match &field.kind {
        FieldKind::Bool => Some(
            if field.value == "true" {
                "false"
            } else {
                "true"
            }
            .to_string(),
        ),
        FieldKind::Choice(choices) if !choices.is_empty() => {
            let i = choices
                .iter()
                .position(|(v, _)| *v == field.value)
                .unwrap_or(0);
            let n = choices.len();
            let next = if forward {
                (i + 1) % n
            } else {
                (i + n - 1) % n
            };
            Some(choices[next].0.clone())
        }
        FieldKind::Int { min, max } => {
            let n: i64 = field.value.parse().unwrap_or(*min);
            let next = if forward {
                n.saturating_add(1)
            } else {
                n.saturating_sub(1)
            };
            Some(next.clamp(*min, *max).to_string())
        }
        _ => None,
    }
}

pub fn display_value(field: &Field) -> String {
    match &field.kind {
        FieldKind::Choice(choices) => choices
            .iter()
            .find(|(v, _)| *v == field.value)
            .map_or(field.value.clone(), |(_, l)| l.clone()),
        FieldKind::Bool => if field.value == "true" { "on" } else { "off" }.into(),
        FieldKind::Text if field.value.is_empty() => "theme default".into(),
        _ => field.value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darwan_core::settings::Key;

    fn field(kind: FieldKind, value: &str) -> Field {
        Field {
            key: Key::ClockFormat,
            label: "x".into(),
            kind,
            value: value.into(),
            is_set: false,
            disabled: None,
        }
    }

    #[test]
    fn choices_wrap_both_ways() {
        let kind = FieldKind::Choice(vec![
            ("a".into(), "A".into()),
            ("b".into(), "B".into()),
            ("c".into(), "C".into()),
        ]);
        assert_eq!(step(&field(kind.clone(), "c"), true).as_deref(), Some("a"));
        assert_eq!(step(&field(kind, "a"), false).as_deref(), Some("c"));
    }

    #[test]
    fn ints_stay_inside_the_manifest_bounds() {
        let kind = FieldKind::Int { min: 1, max: 5 };
        assert_eq!(step(&field(kind.clone(), "5"), true).as_deref(), Some("5"));
        assert_eq!(step(&field(kind, "1"), false).as_deref(), Some("1"));
    }

    #[test]
    fn free_text_kinds_are_not_stepped() {
        assert_eq!(step(&field(FieldKind::Text, ""), true), None);
        assert_eq!(step(&field(FieldKind::Color, "#fff"), true), None);
    }

    #[test]
    fn families_get_sections_and_names_sort_within_them() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let cat = darwan_core::catalog::Catalog::load(&root).unwrap().0;
        let rows = list_rows(&cat, "");
        let sections: Vec<&str> = rows
            .iter()
            .filter_map(|r| match r {
                ListRow::Section { title, .. } => Some(title.as_str()),
                ListRow::Theme(_) => None,
            })
            .collect();
        assert_eq!(
            sections,
            ["Clockwork", "Pixel", OTHER_SECTION],
            "families first, the rest last"
        );
        let names: Vec<&str> = rows
            .iter()
            .filter_map(|r| match r {
                ListRow::Theme(i) => Some(cat.themes()[*i].manifest.name.as_str()),
                ListRow::Section { .. } => None,
            })
            .collect();
        assert_eq!(
            names.len(),
            cat.themes().len(),
            "every theme appears exactly once"
        );
        let pos = |n: &str| names.iter().position(|x| *x == n).unwrap();
        assert!(
            pos("Honkai: Star Rail") < pos("Minecraft"),
            "star-rail's id sorts last, its name doesn't"
        );
    }

    #[test]
    fn search_keeps_only_matching_themes_and_their_sections() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let cat = darwan_core::catalog::Catalog::load(&root).unwrap().0;
        let rows = list_rows(&cat, "ORBIT");
        assert_eq!(
            rows[0],
            ListRow::Section {
                title: "Clockwork".into(),
                count: 2
            },
            "neo-orbital and orbital"
        );
        assert_eq!(rows.len(), 3, "no empty Pixel or Other sections");
        assert!(list_rows(&cat, "reverse-1999").len() > 1, "ids match too");
        assert!(list_rows(&cat, "zzz").is_empty());
    }

    #[test]
    fn values_are_shown_by_their_labels() {
        let kind = FieldKind::Choice(vec![("static".into(), "Fixed".into())]);
        assert_eq!(display_value(&field(kind, "static")), "Fixed");
        assert_eq!(display_value(&field(FieldKind::Bool, "true")), "on");
        assert_eq!(display_value(&field(FieldKind::Text, "")), "theme default");
    }
}
