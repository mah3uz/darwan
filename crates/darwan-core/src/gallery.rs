use std::collections::BTreeMap;

use crate::catalog::{Catalog, Theme};

pub fn display_name(theme: &Theme) -> String {
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

pub fn matches(theme: &Theme, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || display_name(theme).to_lowercase().contains(&q) || theme.id.contains(&q)
}

// Families (Clockwork, Pixel) get their own sections; everything else goes last, all by name.
pub fn list_rows(catalog: &Catalog, query: &str) -> Vec<ListRow> {
    let themes = catalog.themes();
    let mut groups: BTreeMap<(bool, String), Vec<usize>> = Default::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_get_sections_and_names_sort_within_them() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let cat = Catalog::load(&root).unwrap().0;
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
        let cat = Catalog::load(&root).unwrap().0;
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
}
