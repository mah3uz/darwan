use std::collections::HashMap;
use std::path::{Path, PathBuf};

use darwan_core::wallpaper::filter::{Allowed, GROUPS};
use darwan_core::wallpaper::library::{COLOURS, Facts, Item, Kind};
use darwan_core::wallpaper::online::{COMMONS, Found, Query, Source, TOPICS};
use darwan_core::wallpaper::set::Owner;
use serde_json::{Value, json};

// One list of online results: Home's rows, Explore, a category page. It grows a page at a time until it holds
// `target` results; `ticket` counts requests, so a worker for an older one stops and its results are dropped.
// One source inside a channel, paged on its own.
#[derive(Debug, Clone)]
pub struct Feed {
    pub source: Source,
    pub query: Query,
    pub next_page: u32,
    pub more: bool,
}

pub struct Channel {
    pub feeds: Vec<Feed>,
    pub items: Vec<Found>,
    pub loading: bool,
    pub error: String,
    pub refused: String,
    pub target: usize,
    pub ticket: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Channel {
    pub fn new(feeds: Vec<Feed>, target: usize) -> Self {
        Channel {
            feeds,
            items: Vec::new(),
            loading: true,
            error: String::new(),
            refused: String::new(),
            target,
            ticket: Default::default(),
        }
    }

    pub fn more(&self) -> bool {
        self.feeds.iter().any(|f| f.more)
    }
}

// The feeds for a request. Every source unless the user picked one; search words and topics go to Wallhaven, and to
// Commons and APOD when the topic is one of their subjects.
pub fn feeds(sources: &[Source], query: &Query, topic: &str) -> Vec<Feed> {
    let feed = |source: Source, q: Query| Feed {
        source,
        query: q,
        next_page: 1,
        more: true,
    };
    let searching = !query.text.trim().is_empty() || !topic.is_empty();
    // The quick sources first; Wallhaven checks every picture before it shows, so it comes last.
    let mut sources = sources.to_vec();
    sources.sort_by_key(|s| *s == Source::Wallhaven);
    sources
        .iter()
        .filter_map(|s| match s {
            Source::Wallhaven => Some(feed(
                *s,
                Query {
                    text: if topic.is_empty() {
                        query.text.clone()
                    } else {
                        topic.to_string()
                    },
                    ..query.clone()
                },
            )),
            Source::Commons if !searching => Some(feed(*s, query.clone())),
            Source::Commons => COMMONS
                .iter()
                .find(|(t, _)| topic.contains(t) || query.text.to_lowercase().contains(t))
                .map(|(t, _)| {
                    feed(
                        *s,
                        Query {
                            topic: Some(t.to_string()),
                            ..query.clone()
                        },
                    )
                }),
            Source::Apod
                if !searching
                    || topic == "space"
                    || query.text.to_lowercase().contains("space") =>
            {
                Some(feed(*s, query.clone()))
            }
            Source::Bing if !searching => Some(feed(*s, query.clone())),
            _ => None,
        })
        .collect()
}

// The page's data between calls; the workers write into it on the GUI thread. `scan` counts folder scans, so the
// results of an older one are dropped and its workers stop early.
#[derive(Default)]
pub struct State {
    pub folder: PathBuf,
    pub items: Vec<Item>,
    pub ready: HashMap<PathBuf, (PathBuf, Facts)>,
    pub credits: HashMap<PathBuf, Found>,
    pub current: Option<PathBuf>,
    pub owner: Option<Owner>,
    pub scan: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub channels: HashMap<String, Channel>,
    // By `Found::key`: grid thumbnails and full-size previews on disk.
    pub thumbs: HashMap<String, PathBuf>,
    pub full: HashMap<String, PathBuf>,
}

impl State {
    pub fn find(&self, key: &str) -> Option<&Found> {
        self.channels
            .values()
            .flat_map(|c| c.items.iter())
            .find(|f| f.key() == key)
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

pub fn found_json(f: &Found, thumbs: &HashMap<String, PathBuf>, folder: &Path) -> Value {
    json!({
        "key": f.key(),
        "title": if f.title.is_empty() { f.source.name().to_string() } else { f.title.clone() },
        "thumb": thumbs.get(&f.key()).map(|p| format!("file://{}", p.display())).unwrap_or_default(),
        "size": size_text(f.width, f.height),
        "bytes": megabytes(f.bytes),
        "credit": credit_json(f),
        "downloaded": folder.join(f.file_name()).is_file(),
    })
}

// One from each source in turn, each keeping its own order, so "every source" reads as a mix, not blocks.
fn mixed(items: &[Found]) -> Vec<&Found> {
    let mut by_source: Vec<(Source, std::collections::VecDeque<&Found>)> = Vec::new();
    for f in items {
        match by_source.iter_mut().find(|(s, _)| *s == f.source) {
            Some((_, q)) => q.push_back(f),
            None => by_source.push((f.source, std::collections::VecDeque::from([f]))),
        }
    }
    let mut out = Vec::with_capacity(items.len());
    while by_source.iter().any(|(_, q)| !q.is_empty()) {
        for (_, q) in &mut by_source {
            if let Some(f) = q.pop_front() {
                out.push(f);
            }
        }
    }
    out
}

// A channel as the page shows it: at most `target` results, and whether there are more to load.
pub fn online(c: Option<&Channel>, thumbs: &HashMap<String, PathBuf>, folder: &Path) -> Value {
    let Some(c) = c else {
        return json!({ "items": [], "loading": false, "error": "", "refused": "", "more": false, "source": "" });
    };
    let shown: Vec<Value> = mixed(&c.items)
        .into_iter()
        .take(c.target)
        .map(|f| found_json(f, thumbs, folder))
        .collect();
    json!({
        "sources": c.feeds.iter().map(|f| f.source.id()).collect::<Vec<_>>(),
        "items": shown,
        "loading": c.loading,
        "error": c.error,
        "refused": c.refused,
        "more": c.more() || c.items.len() > c.target,
    })
}

// Explore's chips and the Filter's switches, following what the user allows.
pub fn explore(allowed: &Allowed) -> Value {
    json!({
        "topics": TOPICS
            .iter()
            .filter(|(_, _, group)| group.is_none_or(|g| allowed.has(g)))
            .map(|(label, query, _)| json!({ "label": label, "query": query }))
            .collect::<Vec<_>>(),
        "sources": Source::ALL.iter().map(|s| json!({ "value": s.id(), "label": s.name() })).collect::<Vec<_>>(),
        "groups": GROUPS.iter().map(|g| json!({ "id": g.id, "label": g.label, "on": allowed.has(g.id) })).collect::<Vec<_>>(),
    })
}

// What draws the wallpaper, for the page's header and the display pop-up; with the colour generators found, each
// with its switch or the reason it can't run here.
pub fn owner(
    owner: Option<&Owner>,
    restart_ok: &[String],
    generators: &[darwan_core::wallpaper::colours::Found],
    enabled: &[String],
) -> Value {
    let gens: Vec<Value> = generators
        .iter()
        .map(|g| json!({ "id": g.generator.id(), "on": enabled.iter().any(|e| e == g.generator.id()), "reason": g.reason }))
        .collect();
    match owner {
        None => json!({
            "found": false,
            "name": "",
            "generators": gens,
            "note": "Nothing draws a desktop wallpaper in this session. Start your wallpaper tool, then refresh.",
        }),
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
                "generators": gens,
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
    fn a_channel_shows_its_target_and_the_filter_shows_every_optional_group() {
        let f = darwan_core::wallpaper::online::parse_apod(
            r#"[{"copyright":"Jeff Dai","date":"2026-09-26","hdurl":"https://h.jpg","media_type":"image","title":"Meteor","url":"https://s.jpg"},
                {"date":"2026-09-27","hdurl":"https://h2.jpg","media_type":"image","title":"Nebula","url":"https://s2.jpg"}]"#,
        );
        let mut c = Channel::new(feeds(&[Source::Apod], &Query::default(), ""), 1);
        c.items = f;
        c.feeds[0].more = false;
        let j = online(Some(&c), &HashMap::new(), Path::new("/nowhere"));
        assert_eq!(
            j["items"].as_array().unwrap().len(),
            1,
            "only the target is shown"
        );
        assert_eq!(j["more"], true, "the rest is there for scrolling");
        assert_eq!(j["items"][0]["credit"]["licence"], "© Jeff Dai");
        assert_eq!(j["items"][0]["key"], "apod:2026-09-26");

        let allowed = Allowed(vec!["anime".into()]);
        let e = explore(&allowed);
        let topics: Vec<&str> = e["topics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["label"].as_str().unwrap())
            .collect();
        assert!(
            topics.contains(&"Anime") && !topics.contains(&"Video games"),
            "a topic shows once its group is allowed"
        );
        assert!(
            e["groups"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g["id"] != "sexual"),
            "sexual content has no switch"
        );

        let every = feeds(&Source::ALL, &Query::default(), "");
        assert_eq!(every.len(), 4, "every source, until the user picks one");
        let space = feeds(&Source::ALL, &Query::default(), "space");
        let sources: Vec<Source> = space.iter().map(|f| f.source).collect();
        assert_eq!(
            sources,
            [Source::Apod, Source::Commons, Source::Wallhaven],
            "Bing can't be searched; APOD and Commons have space; Wallhaven, the slowest, comes last"
        );
        assert_eq!(space[2].query.text, "space");

        let mut two = Channel::new(
            feeds(&[Source::Apod, Source::Bing], &Query::default(), ""),
            10,
        );
        let bing = darwan_core::wallpaper::online::parse_bing(
            r#"{"images":[{"urlbase":"/a","copyright":"x (© y)","title":"A","wp":true,"hsh":"1"},{"urlbase":"/b","copyright":"x (© y)","title":"B","wp":true,"hsh":"2"}]}"#,
        );
        two.items = [c.items.clone(), bing].concat();
        let shown = online(Some(&two), &HashMap::new(), Path::new("/nowhere"));
        let order: Vec<&str> = shown["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["key"].as_str().unwrap())
            .collect();
        assert_eq!(
            order,
            ["apod:2026-09-26", "bing:1", "apod:2026-09-27", "bing:2"],
            "every source reads as a mix"
        );
    }
}
