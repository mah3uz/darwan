use std::collections::HashMap;
use std::path::{Path, PathBuf};

use darwan_core::wallpaper::filter::{Allowed, GROUPS};
use darwan_core::wallpaper::library::{COLOURS, Facts, Item, Kind};
use darwan_core::wallpaper::online::{COMMONS, Found, Query, Source};
use darwan_core::wallpaper::set::Owner;
use serde_json::{Value, json};

// The page's data between calls; the workers write into it on the GUI thread. `scan` and `search` count requests,
// so results from an older one are dropped; the cancel counters let a worker stop early.
pub struct State {
    pub folder: PathBuf,
    pub items: Vec<Item>,
    pub ready: HashMap<PathBuf, (PathBuf, Facts)>,
    pub credits: HashMap<PathBuf, Found>,
    pub current: Option<PathBuf>,
    pub owner: Option<Owner>,
    pub scan: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub source: Source,
    pub query: Query,
    pub found: Vec<Found>,
    pub thumbs: HashMap<String, PathBuf>,
    pub loading: bool,
    pub error: String,
    pub refused: String,
    pub more: bool,
    pub search: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub searched: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            folder: PathBuf::new(),
            items: Vec::new(),
            ready: HashMap::new(),
            credits: HashMap::new(),
            current: None,
            owner: None,
            scan: Default::default(),
            source: Source::Bing,
            query: Query::default(),
            found: Vec::new(),
            thumbs: HashMap::new(),
            loading: false,
            error: String::new(),
            refused: String::new(),
            more: false,
            search: Default::default(),
            searched: false,
        }
    }
}

// What the Library shows: the folder's wallpapers, newest first, narrowed by the search text and a colour chip.
// `ready` holds each file's thumbnail and facts as the background worker makes them.
pub struct LibraryView<'a> {
    pub folder: &'a Path,
    pub items: &'a [Item],
    pub ready: &'a HashMap<PathBuf, (PathBuf, Facts)>,
    pub credits: &'a HashMap<PathBuf, Found>,
    pub current: Option<&'a Path>,
    pub now: u64,
}

const WEEK: u64 = 7 * 86400;

fn size_text(w: u32, h: u32) -> String {
    if w == 0 || h == 0 {
        String::new()
    } else {
        format!("{w}×{h}")
    }
}

fn megabytes(bytes: u64) -> String {
    if bytes == 0 {
        String::new()
    } else {
        format!("{:.1} MB", bytes as f64 / 1e6)
    }
}

fn title_of(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let words: Vec<String> = stem
        .split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default()
        })
        .collect();
    words.join(" ")
}

fn credit_json(f: &Found) -> Value {
    json!({
        "source": f.source.name(),
        "author": f.author,
        "licence": f.licence,
        "licenceUrl": f.licence_url,
        "page": f.page,
    })
}

pub fn library(v: &LibraryView, text: &str, colour: &str) -> Value {
    let words: Vec<String> = text
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut shown = Vec::new();
    for item in v.items {
        let facts = v.ready.get(&item.path).map(|(_, f)| f);
        if let Some(f) = facts {
            *counts.entry(f.colour.as_str()).or_default() += 1;
        }
        let name = title_of(&item.path);
        let hay = format!(
            "{} {}",
            name.to_lowercase(),
            item.path.display().to_string().to_lowercase()
        );
        if !words.iter().all(|w| hay.contains(w.as_str())) {
            continue;
        }
        if !colour.is_empty() && facts.is_none_or(|f| f.colour != colour) {
            continue;
        }
        let (thumb, w, h, swatches, group) = match v.ready.get(&item.path) {
            Some((t, f)) => (
                format!("file://{}", t.display()),
                f.width,
                f.height,
                f.summary.swatches.clone(),
                f.colour.clone(),
            ),
            None => (String::new(), 0, 0, Vec::new(), String::new()),
        };
        let in_use = v.current.is_some_and(|c| {
            std::fs::canonicalize(c).unwrap_or_else(|_| c.to_path_buf())
                == std::fs::canonicalize(&item.path).unwrap_or_else(|_| item.path.clone())
        });
        shown.push(json!({
            "path": item.path.display().to_string(),
            "file": format!("file://{}", item.path.display()),
            "name": name,
            "folder": item.path.parent().and_then(|p| p.strip_prefix(v.folder).ok()).map(|p| p.display().to_string()).unwrap_or_default(),
            "kind": match item.kind { Kind::Image => "image", Kind::Animated => "animated", Kind::Video => "video" },
            "thumb": thumb,
            "size": size_text(w, h),
            "bytes": megabytes(item.bytes),
            "colour": group,
            "swatches": swatches,
            "isNew": v.now.saturating_sub(item.modified) < WEEK,
            "inUse": in_use,
            "credit": v.credits.get(&item.path).map(credit_json),
        }));
    }
    let chips: Vec<Value> = COLOURS
        .iter()
        .map(|(id, _, _)| *id)
        .chain(["black", "grey", "white"])
        .filter(|id| counts.get(id).copied().unwrap_or(0) > 0)
        .map(|id| json!({ "value": id, "label": capital(id), "count": counts[id] }))
        .collect();
    json!({
        "folder": v.folder.display().to_string(),
        "exists": v.folder.is_dir(),
        "total": v.items.len(),
        "prepared": v.ready.len(),
        "items": shown,
        "colours": chips,
    })
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

pub struct OnlineView<'a> {
    pub source: Source,
    pub items: &'a [Found],
    pub thumbs: &'a HashMap<String, PathBuf>,
    pub loading: bool,
    pub error: &'a str,
    pub refused: &'a str,
    pub more: bool,
    pub allowed: &'a Allowed,
    pub folder: &'a Path,
}

pub fn online(v: &OnlineView) -> Value {
    let items: Vec<Value> = v
        .items
        .iter()
        .map(|f| {
            json!({
                "id": f.id,
                "title": if f.title.is_empty() { f.source.name().to_string() } else { f.title.clone() },
                "thumb": v.thumbs.get(&f.id).map(|p| format!("file://{}", p.display())).unwrap_or_default(),
                "size": size_text(f.width, f.height),
                "bytes": megabytes(f.bytes),
                "credit": credit_json(f),
                "saved": v.folder.join(f.file_name()).is_file(),
            })
        })
        .collect();
    json!({
        "source": v.source.id(),
        "searchable": v.source.searchable(),
        "sources": Source::ALL.iter().map(|s| json!({ "value": s.id(), "label": s.name() })).collect::<Vec<_>>(),
        "topics": COMMONS.iter().map(|(t, _)| json!({ "value": t, "label": capital(t) })).collect::<Vec<_>>(),
        "items": items,
        "loading": v.loading,
        "error": v.error,
        "refused": v.refused,
        "more": v.more,
        "groups": GROUPS.iter().map(|g| json!({ "id": g.id, "label": g.label, "on": v.allowed.has(g.id) })).collect::<Vec<_>>(),
        "note": match v.source {
            Source::Wallhaven => "Images belong to their owners; for personal use. Each is checked before it shows, so the grid fills gradually.",
            Source::Bing => "Bing's recent images of the day, offered for use as wallpapers only.",
            Source::Apod => "NASA's Astronomy Picture of the Day, the last 30 days.",
            Source::Commons => "Wikimedia Commons' featured pictures, freely licensed; credit the author.",
        },
    })
}

// What draws the wallpaper, for the page's header and the display pop-up.
pub fn owner(owner: Option<&Owner>, restart_ok: &[String]) -> Value {
    match owner {
        None => {
            json!({ "found": false, "name": "", "note": "Nothing draws a desktop wallpaper in this session. Start your wallpaper tool, then refresh." })
        }
        Some(o) => {
            let c = o.caps();
            json!({
                "found": true,
                "name": o.tool.name(),
                "perOutput": c.per_output,
                "video": c.video,
                "themes": c.themes,
                "persists": c.persists,
                "shell": o.shell.map(|s| s.name()),
                "restartOk": restart_ok.iter().any(|t| t == o.tool.name()),
                "note": if c.themes {
                    format!("Set through {}, which can make your colours from the wallpaper", o.tool.name())
                } else if c.persists {
                    format!("Set through {}", o.tool.name())
                } else {
                    format!("Set through {}; it forgets after logging out", o.tool.name())
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darwan_core::palette::Summary;

    fn item(path: &str, modified: u64) -> Item {
        Item {
            path: PathBuf::from(path),
            kind: Kind::Image,
            bytes: 2_500_000,
            modified,
        }
    }

    fn facts(colour: &str) -> Facts {
        Facts {
            width: 3840,
            height: 2160,
            colour: colour.into(),
            summary: Summary {
                source: "#000000".into(),
                hue: 0.0,
                chroma: 0.0,
                tone: 0.0,
                luminance: 0.0,
                swatches: vec!["#112233".into()],
            },
        }
    }

    #[test]
    fn the_library_filters_by_words_and_colour_and_marks_new_and_in_use() {
        let items = [
            item("/w/misty-forest.jpg", 1_000_000),
            item("/w/sub/blue-lake.png", 900_000),
            item("/w/red_dunes.jpg", 10),
        ];
        let mut ready = HashMap::new();
        ready.insert(
            PathBuf::from("/w/misty-forest.jpg"),
            (PathBuf::from("/t/1.png"), facts("green")),
        );
        ready.insert(
            PathBuf::from("/w/sub/blue-lake.png"),
            (PathBuf::from("/t/2.png"), facts("blue")),
        );
        let credits = HashMap::new();
        let v = LibraryView {
            folder: Path::new("/w"),
            items: &items,
            ready: &ready,
            credits: &credits,
            current: Some(Path::new("/w/sub/blue-lake.png")),
            now: 1_000_100,
        };

        let all = library(&v, "", "");
        assert_eq!(all["items"].as_array().unwrap().len(), 3);
        assert_eq!(
            all["items"][0]["name"], "Misty Forest",
            "file names read as titles"
        );
        assert_eq!(all["items"][0]["isNew"], true);
        assert_eq!(all["items"][2]["isNew"], false);
        assert_eq!(all["items"][1]["folder"], "sub");
        assert_eq!(
            all["items"][2]["thumb"], "",
            "not prepared yet: the card shows a placeholder"
        );
        assert_eq!(
            all["colours"],
            json!([{"value":"green","label":"Green","count":1},{"value":"blue","label":"Blue","count":1}])
        );

        let blue = library(&v, "", "blue");
        assert_eq!(blue["items"].as_array().unwrap().len(), 1);
        assert_eq!(blue["items"][0]["inUse"], true);
        assert_eq!(library(&v, "DUNES", "")["items"][0]["name"], "Red Dunes");
    }

    #[test]
    fn online_items_carry_their_credit_and_the_groups_show_their_switches() {
        let f = darwan_core::wallpaper::online::parse_apod(
            r#"[{"copyright":"Jeff Dai","date":"2026-09-26","hdurl":"https://h.jpg","media_type":"image","title":"Meteor","url":"https://s.jpg"}]"#,
        );
        let thumbs = HashMap::new();
        let allowed = Allowed(vec!["anime".into()]);
        let v = OnlineView {
            source: Source::Apod,
            items: &f,
            thumbs: &thumbs,
            loading: false,
            error: "",
            refused: "",
            more: false,
            allowed: &allowed,
            folder: Path::new("/nowhere"),
        };
        let j = online(&v);
        assert_eq!(j["items"][0]["credit"]["licence"], "© Jeff Dai");
        assert_eq!(j["searchable"], false, "only Wallhaven has a search field");
        let groups = j["groups"].as_array().unwrap();
        assert!(groups.iter().any(|g| g["id"] == "anime" && g["on"] == true));
        assert!(
            groups.iter().all(|g| g["id"] != "sexual"),
            "sexual content has no switch"
        );
    }
}
