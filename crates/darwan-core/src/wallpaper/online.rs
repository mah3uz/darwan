use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::filter::{self, Allowed};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Wallhaven,
    Bing,
    Apod,
    Commons,
}

impl Source {
    pub const ALL: [Source; 4] = [
        Source::Wallhaven,
        Source::Bing,
        Source::Apod,
        Source::Commons,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Source::Wallhaven => "wallhaven",
            Source::Bing => "bing",
            Source::Apod => "apod",
            Source::Commons => "commons",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Source::Wallhaven => "Wallhaven",
            Source::Bing => "Bing",
            Source::Apod => "NASA APOD",
            Source::Commons => "Wikimedia Commons",
        }
    }

    pub fn parse(s: &str) -> Option<Source> {
        Source::ALL.into_iter().find(|x| x.id() == s)
    }

    // Only Wallhaven can be searched; the others are curated collections.
    pub fn searchable(self) -> bool {
        self == Source::Wallhaven
    }
}

// One wallpaper a source offers, with what the UI must show as credit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Found {
    pub source: Source,
    pub id: String,
    pub title: String,
    pub author: String,
    // The licence or terms line the source requires shown, e.g. "CC BY-SA 4.0" or "© Getty Images".
    pub licence: String,
    pub licence_url: String,
    pub page: String,
    pub full: String,
    pub thumb: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub colours: Vec<String>,
}

impl Found {
    fn extension(&self) -> &str {
        let path = self.full.split('?').next().unwrap_or(&self.full);
        match path
            .rsplit('.')
            .next()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => "png",
            Some("webp") => "webp",
            _ => "jpg",
        }
    }

    // The name in the wallpaper folder: the source and its id, so a second download is the same file.
    pub fn file_name(&self) -> String {
        let safe: String = self
            .id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        format!("{}-{safe}.{}", self.source.id(), self.extension())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    #[default]
    Popular,
    Latest,
    Random,
}

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub text: String,
    pub sort: Sort,
    // Wallhaven's ratios: "16x9", "21x9", "landscape".
    pub ratio: Option<String>,
    pub atleast: Option<(u32, u32)>,
    // A Commons subject chip; see COMMONS.
    pub topic: Option<String>,
    pub page: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Page {
    pub items: Vec<Found>,
    // Why nothing was asked: the search text matched a refused term.
    pub refused: Option<String>,
    pub more: bool,
}

// Commons subject chips and their featured categories: subjects without people, war or predation.
pub const COMMONS: &[(&str, &[&str])] = &[
    (
        "nature",
        &[
            "Featured pictures of mountains",
            "Featured pictures of forests",
            "Featured pictures of coasts",
            "Featured pictures of bodies of water",
            "Featured pictures of islands",
            "Featured pictures of volcanoes",
            "Featured pictures of caves",
            "Featured pictures of plants",
        ],
    ),
    (
        "space",
        &[
            "Featured pictures of astronomy",
            "Featured pictures taken from satellites",
        ],
    ),
    (
        "city",
        &[
            "Featured pictures of cityscapes",
            "Featured pictures of architecture",
        ],
    ),
    ("night", &["Featured night photography"]),
];

pub const USER_AGENT: &str = concat!(
    "Darwan/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/mah3uz/darwan; mah3uz@gmail.com)"
);

// Wallhaven allows 45 requests a minute; Darwan keeps to 40.
struct Limit {
    per_minute: usize,
    sent: Mutex<VecDeque<Instant>>,
}

impl Limit {
    fn wait(&self) {
        loop {
            let mut sent = self.sent.lock().unwrap();
            let now = Instant::now();
            while sent
                .front()
                .is_some_and(|t| now.duration_since(*t) > Duration::from_secs(60))
            {
                sent.pop_front();
            }
            if sent.len() < self.per_minute {
                sent.push_back(now);
                return;
            }
            let wait = Duration::from_secs(60).saturating_sub(now.duration_since(sent[0]));
            drop(sent);
            std::thread::sleep(wait.max(Duration::from_millis(50)));
        }
    }
}

static WALLHAVEN: Limit = Limit {
    per_minute: 40,
    sent: Mutex::new(VecDeque::new()),
};

pub struct Client {
    agent: ureq::Agent,
    cache: PathBuf,
}

impl Client {
    // `cache`: ~/.cache/darwan/online, for responses, tag lists and thumbnails.
    pub fn new(cache: PathBuf) -> Self {
        let config = ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .timeout_global(Some(Duration::from_secs(30)))
            .build();
        Client {
            agent: config.into(),
            cache,
        }
    }

    fn get(&self, url: &str) -> Result<String, String> {
        if url.starts_with("https://wallhaven.cc/") {
            WALLHAVEN.wait();
        }
        // Wallhaven sits behind Cloudflare, whose 52x errors pass in a moment: one retry.
        let mut resp = match self.agent.get(url).call() {
            Err(ureq::Error::StatusCode(code)) if code >= 500 => {
                std::thread::sleep(Duration::from_secs(1));
                self.agent.get(url).call()
            }
            other => other,
        }
        .map_err(|e| format!("{}: {e}", host(url)))?;
        resp.body_mut()
            .with_config()
            .limit(20 * 1024 * 1024)
            .read_to_string()
            .map_err(|e| format!("{}: {e}", host(url)))
    }

    // A response kept for `ttl`, so browsing doesn't ask the source again (Pixabay-style caching rules, and APOD's
    // small DEMO_KEY allowance).
    fn cached(&self, key: &str, ttl: Duration, url: &str) -> Result<String, String> {
        let file = self
            .cache
            .join("responses")
            .join(format!("{:016x}.json", crate::media::fnv1a(key.as_bytes())));
        if let Ok(meta) = std::fs::metadata(&file)
            && meta
                .modified()
                .ok()
                .and_then(|m| m.elapsed().ok())
                .is_some_and(|age| age < ttl)
            && let Ok(text) = std::fs::read_to_string(&file)
        {
            return Ok(text);
        }
        let text = self.get(url)?;
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
            let _ = std::fs::write(&file, &text);
        }
        Ok(text)
    }

    fn fetch_to(&self, url: &str, target: &Path) -> Result<(), String> {
        let resp = self
            .agent
            .get(url)
            .call()
            .map_err(|e| format!("{}: {e}", host(url)))?;
        let dir = target.parent().ok_or("no folder")?;
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
        std::io::copy(&mut resp.into_body().into_reader(), &mut tmp)
            .map_err(|e| format!("{}: {e}", host(url)))?;
        tmp.persist(target).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn search(&self, source: Source, q: &Query, allowed: &Allowed) -> Result<Page, String> {
        if let Some(term) = filter::refusal(&q.text, allowed) {
            return Ok(Page {
                refused: Some(term.to_string()),
                ..Page::default()
            });
        }
        match source {
            Source::Wallhaven => self.wallhaven(q, allowed),
            Source::Bing => {
                let day = Duration::from_secs(6 * 3600);
                let mut items = Vec::new();
                for idx in [0, 7] {
                    let url = format!(
                        "https://www.bing.com/HPImageArchive.aspx?format=js&idx={idx}&n=8&mkt={}",
                        market()
                    );
                    items.extend(parse_bing(&self.cached(&url, day, &url)?));
                }
                items.dedup_by(|a, b| a.id == b.id);
                Ok(keep(items, allowed))
            }
            Source::Apod => {
                let (from, to) = (date_days_ago(30), date_days_ago(0));
                let url = format!(
                    "https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY&start_date={from}&end_date={to}&thumbs=true"
                );
                let text = self.cached(
                    &format!("apod {from}"),
                    Duration::from_secs(12 * 3600),
                    &url,
                )?;
                let mut items = parse_apod(&text);
                items.reverse();
                Ok(keep(items, allowed))
            }
            Source::Commons => self.commons(q, allowed),
        }
    }

    fn wallhaven(&self, q: &Query, allowed: &Allowed) -> Result<Page, String> {
        let text = filter::clean_query(&q.text);
        let query = format!("{text} {}", filter::wallhaven_exclusions(allowed))
            .trim()
            .to_string();
        let sorting = match q.sort {
            Sort::Popular => "toplist",
            Sort::Latest => "date_added",
            Sort::Random => "random",
        };
        let (w, h) = q.atleast.unwrap_or((1920, 1080));
        let mut url = format!(
            "https://wallhaven.cc/api/v1/search?categories={}&purity=100&sorting={sorting}&topRange=1y&atleast={}x{}&page={}&q={}",
            filter::wallhaven_categories(allowed),
            w.max(1920),
            h.max(1080),
            q.page.max(1),
            encode(&query)
        );
        if let Some(r) = &q.ratio {
            url.push_str(&format!("&ratios={}", encode(r)));
        }
        let body = if q.sort == Sort::Random {
            self.get(&url)?
        } else {
            self.cached(&url, Duration::from_secs(3600), &url)?
        };
        let (found, more) = parse_wallhaven_search(&body);
        // Every image's tags are checked before it's shown; one that can't be checked isn't shown.
        let items = found
            .into_iter()
            .filter_map(|mut f| {
                let tags = self.wallhaven_tags(&f.id).ok()?;
                if !tags_allowed(&tags, allowed) {
                    return None;
                }
                name_from_tags(&mut f, &tags);
                Some(f)
            })
            .collect();
        Ok(Page {
            items,
            refused: None,
            more,
        })
    }

    // A wallpaper's tags, kept a week: moderators can change them, and the switches are applied to the list, so
    // changing a switch needs no new request.
    fn wallhaven_tags(&self, id: &str) -> Result<Vec<Tag>, String> {
        let url = format!("https://wallhaven.cc/api/v1/w/{}", encode(id));
        let text = self.cached(
            &format!("wallhaven tags {id}"),
            Duration::from_secs(7 * 86400),
            &url,
        )?;
        parse_wallhaven_tags(&text).ok_or_else(|| format!("wallhaven: no tags for {id}"))
    }

    fn commons(&self, q: &Query, allowed: &Allowed) -> Result<Page, String> {
        let topic = q.topic.as_deref().unwrap_or("nature");
        let cats = COMMONS
            .iter()
            .find(|(t, _)| *t == topic)
            .map(|(_, c)| *c)
            .ok_or_else(|| format!("no Commons subject {topic}"))?;
        let mut items = Vec::new();
        for cat in cats {
            let url = format!(
                "https://commons.wikimedia.org/w/api.php?action=query&format=json&formatversion=2&generator=categorymembers&gcmtitle={}&gcmtype=file&gcmlimit=50&prop=imageinfo|categories&cllimit=max&clshow=!hidden&iiprop=url|size|mime|extmetadata&iiurlwidth=500&iiextmetadatafilter=LicenseShortName|LicenseUrl|Artist|ObjectName",
                encode(&format!("Category:{cat}"))
            );
            // Commons asks for one request at a time, which this loop is.
            items.extend(parse_commons(
                &self.cached(&url, Duration::from_secs(86400), &url)?,
                allowed,
            ));
        }
        let mut page = keep(items, allowed);
        if q.sort == Sort::Random {
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            page.items
                .sort_by_key(|f| crate::media::fnv1a(format!("{seed}{}", f.id).as_bytes()));
        }
        Ok(page)
    }

    // The grid's picture for `found`, from the cache when it's there.
    pub fn thumbnail(&self, found: &Found) -> Result<PathBuf, String> {
        let file = self.cache.join("thumbs").join(format!(
            "{}-{}.jpg",
            found.source.id(),
            crate::media::fnv1a(found.id.as_bytes())
        ));
        if !file.is_file() {
            self.fetch_to(&found.thumb, &file)?;
        }
        Ok(file)
    }

    // Into the wallpaper folder under a stable name, with its credit kept beside Darwan's data.
    pub fn download(
        &self,
        found: &Found,
        folder: &Path,
        credits: &Path,
    ) -> Result<PathBuf, String> {
        let target = folder.join(found.file_name());
        if !target.is_file() {
            self.fetch_to(&found.full, &target)?;
        }
        save_credit(credits, &target, found);
        Ok(target)
    }
}

fn host(url: &str) -> &str {
    url.split('/').nth(2).unwrap_or(url)
}

fn keep(items: Vec<Found>, allowed: &Allowed) -> Page {
    Page {
        items: items
            .into_iter()
            .filter(|f| filter::refusal(&format!("{} {}", f.title, f.licence), allowed).is_none())
            .collect(),
        refused: None,
        more: false,
    }
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn market() -> String {
    let lang = std::env::var("LC_ALL")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default();
    let tag = lang.split('.').next().unwrap_or("").replace('_', "-");
    if tag.len() == 5 { tag } else { "en-US".into() }
}

fn date_days_ago(days: u64) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        - days * 86400;
    let (y, m, d) = civil(secs / 86400);
    format!("{y:04}-{m:02}-{d:02}")
}

// Days since 1970-01-01 to a calendar date (Howard Hinnant's civil_from_days).
fn civil(days: u64) -> (i64, u32, u32) {
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

fn n(v: &Value, k: &str) -> u64 {
    v.get(k).and_then(Value::as_u64).unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub alias: String,
    pub category: String,
    pub purity: String,
}

// Only SFW, general-category results (the category bit is the uploader's; tags decide the rest).
pub fn parse_wallhaven_search(text: &str) -> (Vec<Found>, bool) {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return (Vec::new(), false);
    };
    let more = n(&v["meta"], "current_page") < n(&v["meta"], "last_page");
    let items = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| {
            s(d, "purity") == "sfw"
                && matches!(s(d, "category").as_str(), "general" | "anime" | "people")
        })
        .map(|d| Found {
            source: Source::Wallhaven,
            id: s(d, "id"),
            title: String::new(),
            author: String::new(),
            licence: "Images belong to their owners; for personal use".into(),
            licence_url: "https://wallhaven.cc/terms".into(),
            page: s(d, "url"),
            full: s(d, "path"),
            thumb: s(&d["thumbs"], "large"),
            width: n(d, "dimension_x") as u32,
            height: n(d, "dimension_y") as u32,
            bytes: n(d, "file_size"),
            colours: d["colors"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|c| c.as_str().map(str::to_string))
                .collect(),
        })
        .collect();
    (items, more)
}

pub fn parse_wallhaven_tags(text: &str) -> Option<Vec<Tag>> {
    let v: Value = serde_json::from_str(text).ok()?;
    let data = &v["data"];
    if s(data, "purity") != "sfw" {
        return Some(vec![Tag {
            name: "not sfw".into(),
            alias: String::new(),
            category: String::new(),
            purity: s(data, "purity"),
        }]);
    }
    Some(
        data["tags"]
            .as_array()?
            .iter()
            .map(|t| Tag {
                name: s(t, "name"),
                alias: s(t, "alias"),
                category: s(t, "category"),
                purity: s(t, "purity"),
            })
            .collect(),
    )
}

// Wallhaven has no titles: the first few subject tags make one, and a photographer or artist tag is the author.
pub fn name_from_tags(f: &mut Found, tags: &[Tag]) {
    let credit = |t: &&Tag| {
        matches!(
            t.category.as_str(),
            "Photographers" | "Artists" | "Artist" | "Photographer"
        )
    };
    f.author = tags
        .iter()
        .filter(credit)
        .map(|t| t.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let subjects: Vec<&str> = tags
        .iter()
        .filter(|t| !credit(t))
        .take(3)
        .map(|t| t.name.as_str())
        .collect();
    if !subjects.is_empty() {
        f.title = subjects.join(", ");
    }
}

// Any non-SFW tag, a tag category the user hasn't allowed, or a refused word in a tag's name or aliases hides it.
pub fn tags_allowed(tags: &[Tag], allowed: &Allowed) -> bool {
    tags.iter().all(|t| {
        t.purity == "sfw"
            && !filter::refused_category(&t.category, allowed)
            && filter::refusal(
                &format!("{} {}", t.name, t.alias.replace(',', " ")),
                allowed,
            )
            .is_none()
    })
}

// Bing marks the images it offers as wallpapers (`wp`); the others aren't Darwan's to set.
pub fn parse_bing(text: &str) -> Vec<Found> {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    v["images"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|i| i["wp"].as_bool() == Some(true))
        .map(|i| {
            let base = format!("https://www.bing.com{}", s(i, "urlbase"));
            let copyright = s(i, "copyright");
            let (what, holder) = match copyright.rsplit_once(" (©") {
                Some((w, h)) => (w.to_string(), format!("©{}", h.trim_end_matches(')'))),
                None => (copyright.clone(), String::new()),
            };
            Found {
                source: Source::Bing,
                id: s(i, "hsh"),
                title: if s(i, "title").is_empty() {
                    what.clone()
                } else {
                    format!("{}: {what}", s(i, "title"))
                },
                author: holder.clone(),
                licence: format!("{holder}. Bing image of the day, for use as a wallpaper only"),
                licence_url: s(i, "copyrightlink"),
                page: s(i, "copyrightlink"),
                full: format!("{base}_UHD.jpg"),
                thumb: format!("{base}_800x480.jpg"),
                width: 3840,
                height: 2160,
                bytes: 0,
                colours: Vec::new(),
            }
        })
        .collect()
}

pub fn parse_apod(text: &str) -> Vec<Found> {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    v.as_array()
        .into_iter()
        .flatten()
        .filter(|d| s(d, "media_type") == "image")
        .filter_map(|d| {
            let full = [s(d, "hdurl"), s(d, "url")]
                .into_iter()
                .find(|u| !u.is_empty())?;
            let date = s(d, "date");
            let copyright = s(d, "copyright").trim().replace('\n', " ");
            Some(Found {
                source: Source::Apod,
                id: date.clone(),
                title: s(d, "title"),
                author: copyright.clone(),
                licence: if copyright.is_empty() {
                    "Public domain (NASA)".into()
                } else {
                    format!("© {copyright}")
                },
                licence_url: "https://apod.nasa.gov/apod/lib/about_apod.html".into(),
                page: format!(
                    "https://apod.nasa.gov/apod/ap{}.html",
                    date.replace('-', "").get(2..).unwrap_or("")
                ),
                full,
                thumb: s(d, "url"),
                width: 0,
                height: 0,
                bytes: 0,
                colours: Vec::new(),
            })
        })
        .collect()
}

fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

// Wide enough for a desktop and landscape-shaped (no panoramas or portraits), in a still format, with no refused
// word in its categories or title.
pub fn parse_commons(text: &str, allowed: &Allowed) -> Vec<Found> {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    v["query"]["pages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let ii = p["imageinfo"].get(0)?;
            let (w, h) = (n(ii, "width") as u32, n(ii, "height") as u32);
            let ratio = f64::from(w) / f64::from(h.max(1));
            if w < 1920
                || !(1.3..=2.5).contains(&ratio)
                || !matches!(s(ii, "mime").as_str(), "image/jpeg" | "image/png")
            {
                return None;
            }
            let cats: Vec<String> = p["categories"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|c| s(c, "title"))
                .collect();
            let meta = &ii["extmetadata"];
            let m = |k: &str| strip_html(&s(&meta[k], "value"));
            let title = if m("ObjectName").is_empty() {
                s(p, "title").trim_start_matches("File:").to_string()
            } else {
                m("ObjectName")
            };
            if filter::refusal(&format!("{title} {}", cats.join(" ")), allowed).is_some() {
                return None;
            }
            let thumb = s(ii, "thumburl");
            // Commons serves standard widths only; 3840 is one, and a smaller original is used as it is.
            let full = if w > 3840 {
                thumb.replacen("/500px-", "/3840px-", 1)
            } else {
                s(ii, "url")
            };
            Some(Found {
                source: Source::Commons,
                id: n(p, "pageid").to_string(),
                title,
                author: m("Artist"),
                licence: m("LicenseShortName"),
                licence_url: m("LicenseUrl"),
                page: s(ii, "descriptionurl"),
                full,
                thumb,
                width: w,
                height: h,
                bytes: n(ii, "size"),
                colours: Vec::new(),
            })
        })
        .collect()
}

fn credit_file(credits: &Path, file: &Path) -> PathBuf {
    credits.join(format!(
        "{:016x}.json",
        crate::media::fnv1a(file.as_os_str().as_encoded_bytes())
    ))
}

fn save_credit(credits: &Path, file: &Path, found: &Found) {
    if std::fs::create_dir_all(credits).is_ok()
        && let Ok(text) = serde_json::to_string(found)
    {
        let _ = std::fs::write(credit_file(credits, file), text);
    }
}

// Where a downloaded wallpaper came from, for the Library's credit line.
pub fn credit(credits: &Path, file: &Path) -> Option<Found> {
    serde_json::from_str(&std::fs::read_to_string(credit_file(credits, file)).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> Allowed {
        Allowed::default()
    }

    #[test]
    fn wallhaven_results_are_sfw_only_and_the_uploaders_category_is_only_a_hint() {
        let text = r##"{"data":[
            {"id":"a1","purity":"sfw","category":"general","path":"https://w.wallhaven.cc/full/a1/wallhaven-a1.jpg","thumbs":{"large":"https://th/a1.jpg"},"dimension_x":3840,"dimension_y":2160,"file_size":5,"url":"https://wallhaven.cc/w/a1","colors":["#424153"]},
            {"id":"b2","purity":"sketchy","category":"general","path":"x","thumbs":{"large":"y"}},
            {"id":"c3","purity":"nsfw","category":"people","path":"x","thumbs":{"large":"y"}}
        ],"meta":{"current_page":1,"last_page":4}}"##;
        let (items, more) = parse_wallhaven_search(text);
        assert_eq!(
            items.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
            ["a1"]
        );
        assert!(more);
        assert_eq!(items[0].file_name(), "wallhaven-a1.jpg");
    }

    fn tag(name: &str, category: &str, purity: &str) -> Tag {
        Tag {
            name: name.into(),
            alias: String::new(),
            category: category.into(),
            purity: purity.into(),
        }
    }

    #[test]
    fn a_wallhaven_image_is_shown_only_when_every_tag_passes() {
        let ok = [
            tag("Alex Mesmer", "Photographers", "sfw"),
            tag("Seiganto-ji", "Architecture", "sfw"),
        ];
        assert!(tags_allowed(&ok, &none()));
        // The research's probe: an SFW, "general" upload tagged women and school uniform, filed under Anime & Manga.
        let probe = [
            tag("women", "People", "sfw"),
            tag("school uniform", "Clothing", "sfw"),
            tag("anime girls", "Anime & Manga", "sfw"),
        ];
        assert!(!tags_allowed(&probe, &none()));
        assert!(
            tags_allowed(&probe, &Allowed(vec!["people".into(), "anime".into()])),
            "with people and anime switched on nothing in it is refused"
        );
        let sketchy_tag = [tag("sunset", "Nature", "sketchy")];
        assert!(
            !tags_allowed(
                &sketchy_tag,
                &Allowed(filter::GROUPS.iter().map(|g| g.id.to_string()).collect())
            ),
            "a tag's own purity wins"
        );
        let alias = [Tag {
            name: "Model".into(),
            alias: "Sexy Woman, hot girls".into(),
            category: "People".into(),
            purity: "sfw".into(),
        }];
        assert!(
            !tags_allowed(&alias, &Allowed(vec!["people".into()])),
            "aliases are words too, and sexual ones never pass"
        );
    }

    #[test]
    fn a_wallhaven_title_and_author_come_from_its_tags() {
        let (mut items, _) = parse_wallhaven_search(
            r#"{"data":[{"id":"w5","purity":"sfw","category":"general","path":"p","thumbs":{"large":"t"}}],"meta":{}}"#,
        );
        let tags = [
            tag("Alex Mesmer", "Photographers", "sfw"),
            tag("Seiganto-ji", "Architecture", "sfw"),
            tag("waterfall", "Nature", "sfw"),
        ];
        name_from_tags(&mut items[0], &tags);
        assert_eq!(
            (items[0].title.as_str(), items[0].author.as_str()),
            ("Seiganto-ji, waterfall", "Alex Mesmer")
        );
    }

    #[test]
    fn bing_keeps_only_what_it_offers_as_wallpaper_with_its_credit() {
        let text = r#"{"images":[
            {"startdate":"20260929","urlbase":"/th?id=OHR.KasilofRiver_EN-US0047556055","copyright":"The blue waters of the Kasilof River, Alaska, USA (© jared lloyd/Getty Images)","copyrightlink":"https://www.bing.com/search?q=Kasilof","title":"Born of glaciers","wp":true,"hsh":"6321"},
            {"urlbase":"/th?id=OHR.Other","copyright":"x (© y)","title":"t","wp":false,"hsh":"77"}
        ]}"#;
        let items = parse_bing(text);
        assert_eq!(items.len(), 1);
        let f = &items[0];
        assert_eq!(
            f.full,
            "https://www.bing.com/th?id=OHR.KasilofRiver_EN-US0047556055_UHD.jpg"
        );
        assert_eq!(f.author, "© jared lloyd/Getty Images");
        assert!(f.licence.contains("wallpaper only"));
    }

    #[test]
    fn apod_skips_videos_and_names_the_copyright_holder_or_public_domain() {
        let text = r#"[
            {"copyright":"Jeff Dai","date":"2026-09-26","hdurl":"https://apod.nasa.gov/apod/image/2609/Big.jpg","media_type":"image","title":"Mirrored Meteor","url":"https://apod.nasa.gov/apod/image/2609/Small.jpg"},
            {"date":"2026-09-27","media_type":"video","title":"A video","url":"https://youtube"},
            {"date":"2026-09-28","hdurl":"https://apod/h.jpg","media_type":"image","title":"Nebula","url":"https://apod/s.jpg"}
        ]"#;
        let items = parse_apod(text);
        assert_eq!(items.len(), 2);
        assert_eq!(
            (items[0].licence.as_str(), items[0].page.as_str()),
            ("© Jeff Dai", "https://apod.nasa.gov/apod/ap260926.html")
        );
        assert_eq!(items[1].licence, "Public domain (NASA)");
    }

    #[test]
    fn commons_keeps_desktop_shaped_stills_with_their_licence_and_refuses_by_category() {
        let page = |id: u64, w: u64, h: u64, cat: &str| {
            format!(
                r#"{{"pageid":{id},"title":"File:P{id}.jpg","categories":[{{"title":"Category:{cat}"}}],"imageinfo":[{{"width":{w},"height":{h},"size":9,"mime":"image/jpeg","url":"https://upload/P{id}.jpg","thumburl":"https://thumb/P{id}.jpg/500px-P{id}.jpg","descriptionurl":"https://commons/File:P{id}.jpg","extmetadata":{{"Artist":{{"value":"<a href=\"//x\">Ggia</a>"}},"LicenseShortName":{{"value":"CC BY-SA 3.0"}},"LicenseUrl":{{"value":"https://cc/by-sa/3.0"}}}}}}]}}"#
            )
        };
        let text = format!(
            r#"{{"query":{{"pages":[{},{},{},{}]}}}}"#,
            page(1, 6000, 4000, "Mountains of Nepal"),
            page(2, 13000, 1486, "Panoramics"),
            page(3, 1200, 800, "Small"),
            page(4, 4000, 2667, "Nude photography")
        );
        let items = parse_commons(&text, &none());
        assert_eq!(
            items.iter().map(|f| f.title.as_str()).collect::<Vec<_>>(),
            ["P1.jpg"],
            "a panorama, a small one and a refused category are left out"
        );
        let f = &items[0];
        assert_eq!(
            (f.author.as_str(), f.licence.as_str()),
            ("Ggia", "CC BY-SA 3.0"),
            "CC BY-SA needs the author and licence shown"
        );
        assert_eq!(
            f.full, "https://thumb/P1.jpg/3840px-P1.jpg",
            "a standard width, not the 6000 px original"
        );
    }

    #[test]
    fn dates_and_markets_are_what_the_apis_take() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(20725), (2026, 9, 29));
        assert_eq!(encode("mountain -women"), "mountain%20-women");
    }

    #[test]
    fn credits_are_kept_for_downloaded_files() {
        let d = tempfile::tempdir().unwrap();
        let f = parse_apod(r#"[{"date":"2026-09-26","hdurl":"https://h.jpg","media_type":"image","title":"T","url":"https://s.jpg"}]"#).remove(0);
        let file = d.path().join("apod-2026-09-26.jpg");
        save_credit(&d.path().join("credits"), &file, &f);
        assert_eq!(credit(&d.path().join("credits"), &file), Some(f));
    }
}
