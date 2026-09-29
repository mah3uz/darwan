use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use darwan_core::config::UserConfig;
use darwan_core::paths::{self, Paths};
use darwan_core::wallpaper::Env;
use darwan_core::wallpaper::library::{self, Item, Size};
use darwan_core::wallpaper::set::{self, Request, Target};

use crate::style;

pub fn status() -> Result<ExitCode, String> {
    let env = Env::system();
    let Some(owner) = set::owner(&env) else {
        println!(
            "{}",
            style::warn("Nothing draws a desktop wallpaper in this session.")
        );
        return Ok(ExitCode::FAILURE);
    };
    let caps = owner.caps();
    let yes = |b: bool| if b { "yes" } else { "no" };
    println!(
        "{} {}",
        style::bold("Wallpaper by"),
        style::id(owner.tool.name())
    );
    if let Some(shell) = owner.shell {
        println!(
            "  {} also runs and makes colours from the wallpaper; Darwan sets the drawer",
            shell.name()
        );
    }
    println!(
        "  per screen: {}, video: {}, can make colours from the wallpaper: {}, kept after a restart: {}",
        yes(caps.per_output),
        yes(caps.video),
        yes(caps.themes),
        yes(caps.persists)
    );
    let outputs = outputs().unwrap_or_default();
    let shown = |o: Option<&str>| {
        set::current(&env, &owner, o).map_or_else(
            || style::dim("(can't be asked)"),
            |p| p.display().to_string(),
        )
    };
    if caps.per_output && !outputs.is_empty() {
        for o in &outputs {
            println!("  {}: {}", style::bold(o), shown(Some(o)));
        }
    } else {
        println!("  now: {}", shown(None));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn set_file(file: &Path, chosen: &[String], allow_restart: bool) -> Result<ExitCode, String> {
    let path = std::fs::canonicalize(file).map_err(|e| format!("{}: {e}", file.display()))?;
    if !path.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    let env = Env::system();
    let owner = set::owner(&env).ok_or(
        "nothing draws a desktop wallpaper in this session; start your wallpaper tool first",
    )?;
    let outputs = outputs()?;
    for c in chosen {
        if !outputs.contains(c) {
            return Err(format!(
                "no screen named {c}; screens: {}",
                outputs.join(", ")
            ));
        }
    }
    let req = Request {
        path,
        target: if chosen.is_empty() {
            Target::All
        } else {
            Target::Outputs(chosen.to_vec())
        },
        outputs,
    };
    let plan = set::plan(&env, &owner, &req)?;
    let remembered = UserConfig::load(&paths::config_file())
        .map(|c| c.wallpaper_restart())
        .unwrap_or_default();
    if let Some(tool) = plan.restarts
        && !allow_restart
        && !remembered.iter().any(|t| t == tool.name())
    {
        return Err(format!(
            "{} can only change the wallpaper by being restarted, and Darwan didn't start it; run again with --allow-restart, or allow it for good with `darwan set wallpaper.restart {}`",
            tool.name(),
            tool.name()
        ));
    }
    for note in &plan.notes {
        println!("{}", style::dim(note));
    }
    match set::carry_out(&env, &owner, &req, &plan, &set::System)? {
        set::Outcome::Shown => println!(
            "{} {} with {}",
            style::ok("Set"),
            req.path.display(),
            owner.tool.name()
        ),
        set::Outcome::Sent => println!(
            "{} {} with {} {}",
            style::ok("Sent"),
            req.path.display(),
            owner.tool.name(),
            style::dim("(it can't be asked what it shows)")
        ),
    }
    let enabled = UserConfig::load(&paths::config_file())
        .map(|c| c.wallpaper_colours())
        .unwrap_or_default();
    for (name, result) in darwan_core::wallpaper::colours::run_after(
        &env,
        Some(&owner),
        &enabled,
        &req.path,
        &paths::cache_home(),
    ) {
        match result {
            Ok(()) => println!("{} with {name}", style::ok("Colours made")),
            Err(e) => println!("{} {name}: {e}", style::warn("Colours not made:")),
        }
    }
    Ok(ExitCode::SUCCESS)
}

// Connector names from the compositor itself (wl_output v4), so any compositor works.
fn outputs() -> Result<Vec<String>, String> {
    use wayland_client::protocol::{wl_output, wl_registry};
    use wayland_client::{Connection, Dispatch, QueueHandle};

    #[derive(Default)]
    struct State {
        names: Vec<String>,
    }
    impl Dispatch<wl_registry::WlRegistry, ()> for State {
        fn event(
            _: &mut Self,
            reg: &wl_registry::WlRegistry,
            ev: wl_registry::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            if let wl_registry::Event::Global {
                name,
                interface,
                version,
            } = ev
                && interface == "wl_output"
                && version >= 4
            {
                reg.bind::<wl_output::WlOutput, _, _>(name, 4, qh, ());
            }
        }
    }
    impl Dispatch<wl_output::WlOutput, ()> for State {
        fn event(
            s: &mut Self,
            _: &wl_output::WlOutput,
            ev: wl_output::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            if let wl_output::Event::Name { name } = ev {
                s.names.push(name);
            }
        }
    }

    let conn = Connection::connect_to_env()
        .map_err(|e| format!("can't reach the Wayland session ({e}); run this inside it"))?;
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());
    let mut state = State::default();
    for _ in 0..2 {
        queue.roundtrip(&mut state).map_err(|e| e.to_string())?;
    }
    Ok(state.names)
}

fn folder_and_items() -> Result<(std::path::PathBuf, Vec<Item>), String> {
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let folder = library::folder(&config, &home, &paths::config_home());
    if !folder.is_dir() {
        return Err(format!(
            "{} doesn't exist; set another with `darwan set wallpaper.folder <path>`",
            folder.display()
        ));
    }
    let items = library::scan(&folder);
    Ok((folder, items))
}

pub fn list(_paths: &Paths) -> Result<ExitCode, String> {
    let (folder, items) = folder_and_items()?;
    println!(
        "{} {} {}",
        style::bold("Wallpapers in"),
        folder.display(),
        style::dim(format!("({})", items.len()))
    );
    for i in &items {
        let name = i
            .path
            .strip_prefix(&folder)
            .unwrap_or(&i.path)
            .display()
            .to_string();
        let kind = match i.kind {
            library::Kind::Image => "",
            library::Kind::Animated => " animated",
            library::Kind::Video => " video",
        };
        println!(
            "  {name}{}  {}",
            style::dim(kind),
            style::dim(format!("{:.1} MB", i.bytes as f64 / 1e6))
        );
    }
    Ok(ExitCode::SUCCESS)
}

// Thumbnails and colours for every wallpaper, a few at a time; what is already cached costs a file read.
pub fn prepare(_paths: &Paths) -> Result<ExitCode, String> {
    let (folder, items) = folder_and_items()?;
    let started = Instant::now();
    let cache_home = paths::cache_home();
    let facts_dir = paths::cache_dir().join("wallpapers");
    let workers = std::thread::available_parallelism().map_or(2, |n| (n.get() / 2).clamp(1, 4));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failed = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(item) = items.get(i) else { break };
                    let done = library::thumbnail(&cache_home, item, Size::XLarge)
                        .and_then(|thumb| library::facts(&facts_dir, item, &thumb));
                    if let Err(e) = done {
                        failed.lock().unwrap().push(e);
                    }
                }
            });
        }
    });
    let failed = failed.into_inner().unwrap();
    println!(
        "{} {} wallpapers in {} in {:.1} s {}",
        style::ok("Prepared"),
        items.len() - failed.len(),
        folder.display(),
        started.elapsed().as_secs_f64(),
        style::dim(format!("({workers} at a time)"))
    );
    for e in &failed {
        println!("  {} {e}", style::warn("skipped"));
    }
    Ok(ExitCode::SUCCESS)
}

pub fn online(
    source: &str,
    text: &str,
    sort: &str,
    topic: Option<String>,
    download: Option<usize>,
    set_it: bool,
) -> Result<ExitCode, String> {
    use darwan_core::wallpaper::filter::Allowed;
    use darwan_core::wallpaper::online::{Client, Query, Sort, Source};

    let src = Source::parse(source)
        .ok_or_else(|| format!("no source {source:?}; use wallhaven, bing, apod or commons"))?;
    let sort = match sort {
        "popular" => Sort::Popular,
        "latest" => Sort::Latest,
        "random" => Sort::Random,
        other => return Err(format!("sort is popular, latest or random, not {other:?}")),
    };
    let config_path = paths::config_file();
    let config =
        UserConfig::load(&config_path).map_err(|e| format!("{}: {e}", config_path.display()))?;
    let allowed = Allowed(
        config
            .wallpaper_allow()
            .map_err(|e| format!("{}: {e}", config_path.display()))?
            .unwrap_or_default(),
    );
    let client = Client::new(paths::cache_dir().join("online"));
    let query = Query {
        text: text.to_string(),
        sort,
        topic,
        page: 1,
        ..Query::default()
    };
    let page = client.search(src, &query, &allowed)?;
    if let Some(term) = &page.refused {
        println!(
            "{} the search has a refused word ({term})",
            style::warn("Nothing asked:")
        );
        return Ok(ExitCode::FAILURE);
    }
    println!(
        "{} {} {}",
        style::bold(src.name()),
        style::dim(format!("({} shown)", page.items.len())),
        if page.more {
            style::dim("more pages")
        } else {
            String::new()
        }
    );
    for (i, f) in page.items.iter().enumerate() {
        let size = if f.width > 0 {
            format!("{}×{}", f.width, f.height)
        } else {
            String::new()
        };
        let title = if f.title.is_empty() {
            f.id.clone()
        } else {
            f.title.clone()
        };
        println!(
            "{:>3}. {title}  {}  {}",
            i + 1,
            style::dim(size),
            style::dim(&f.licence)
        );
    }
    let Some(n) = download else {
        return Ok(ExitCode::SUCCESS);
    };
    let found = page
        .items
        .get(n.wrapping_sub(1))
        .ok_or_else(|| format!("no result {n}"))?;
    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let folder = library::folder(&config, &home, &paths::config_home());
    let file = client.download(found, &folder, &paths::data_dir().join("credits"))?;
    println!("{} {}", style::ok("Saved"), file.display());
    if set_it {
        return set_file(&file, &[], false);
    }
    Ok(ExitCode::SUCCESS)
}
