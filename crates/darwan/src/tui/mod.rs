mod logic;
mod previews;
mod style;
mod view;

use std::io::{BufRead, Write};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use darwan_core::catalog::{Catalog, Theme};
use darwan_core::config::UserConfig;
use darwan_core::environment::Environment;
use darwan_core::form::{self, FieldKind};
use darwan_core::gallery::{ListRow, list_rows};
use darwan_core::paths::{self, Paths};
use darwan_core::settings::{self, Key};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui_image::picker::Picker;

use crate::settings_cmd;
use logic::step;
use previews::Previews;
use style::Look;

pub enum Mode {
    Browse,
    Form {
        selected: usize,
        editing: Option<String>,
    },
    Input {
        prompt: String,
        buffer: String,
    },
    Help,
    Search,
}

const SETTLE: Duration = Duration::from_millis(120);
const SAVED_FOR: Duration = Duration::from_secs(15);

pub struct App {
    catalog: Catalog,
    rows: Vec<ListRow>,
    query: String,
    look: Look,
    config: UserConfig,
    env: Environment,
    list: ListState,
    mode: Mode,
    status: String,
    status_at: Instant,
    picker: Picker,
    previews: Previews,
    moved_at: Instant,
    quit: bool,
    // R was pressed once in the settings; a second R resets the whole theme.
    reset_armed: bool,
}

enum Move {
    Next,
    Prev,
    First,
    Last,
}

fn theme_at<'a>(catalog: &'a Catalog, rows: &[ListRow], row: Option<usize>) -> Option<&'a Theme> {
    match rows.get(row?)? {
        ListRow::Theme(j) => catalog.themes().get(*j),
        ListRow::Section { .. } => None,
    }
}

enum Outcome {
    Stay,
    Run { args: Vec<String>, pause: bool },
}

impl App {
    fn new(
        catalog: Catalog,
        config: UserConfig,
        env: Environment,
        picker: Picker,
        previews: Previews,
        look: Look,
    ) -> Self {
        let rows = list_rows(&catalog, "");
        let mut list = ListState::default();
        list.select(rows.iter().position(|r| matches!(r, ListRow::Theme(_))));
        Self {
            rows,
            query: String::new(),
            look,
            catalog,
            config,
            env,
            list,
            mode: Mode::Browse,
            status: String::new(),
            status_at: Instant::now(),
            picker,
            previews,
            moved_at: Instant::now(),
            quit: false,
            reset_armed: false,
        }
    }

    fn selected(&self) -> Option<&Theme> {
        theme_at(&self.catalog, &self.rows, self.list.selected())
    }

    fn move_selection(&mut self, to: Move) {
        let themes: Vec<usize> = (0..self.rows.len())
            .filter(|&i| matches!(self.rows[i], ListRow::Theme(_)))
            .collect();
        let Some(&first) = themes.first() else { return };
        let current = self.list.selected().unwrap_or(first);
        let target = match to {
            Move::Next => themes
                .iter()
                .copied()
                .find(|&i| i > current)
                .unwrap_or(current),
            Move::Prev => themes
                .iter()
                .rev()
                .copied()
                .find(|&i| i < current)
                .unwrap_or(current),
            Move::First => first,
            Move::Last => *themes.last().unwrap_or(&first),
        };
        if self.list.selected() != Some(target) {
            self.moved_at = Instant::now();
        }
        self.list.select(Some(target));
    }

    fn refilter(&mut self) {
        let keep = self.selected().map(|t| t.id.clone());
        self.rows = list_rows(&self.catalog, &self.query);
        let theme_row = |id: Option<&str>| {
            self.rows.iter().position(|r| match r {
                ListRow::Theme(j) => id.is_none_or(|id| self.catalog.themes()[*j].id == id),
                ListRow::Section { .. } => false,
            })
        };
        let row = theme_row(keep.as_deref()).or_else(|| theme_row(None));
        if row != self.list.selected() {
            self.moved_at = Instant::now();
        }
        self.list.select(row);
    }

    // Holding a key must not queue a preview for every theme it passes.
    fn settled(&self) -> bool {
        self.moved_at.elapsed() >= SETTLE
    }

    fn change(&mut self, key: &Key, value: Option<&str>) {
        let result = match value {
            Some(v) => settings::set(&mut self.config, &self.catalog, key, v)
                .map(|()| format!("{key} = {v}")),
            None => Ok(if settings::unset(&mut self.config, key) {
                format!("{key} is back to its default")
            } else {
                format!("{key} was already the default")
            }),
        };
        self.status = match result.and_then(|msg| settings_cmd::save(&self.config).map(|()| msg)) {
            Ok(msg) => {
                settings_cmd::prepare_media(key, &self.config);
                format!("saved: {msg}")
            }
            Err(e) => e,
        };
    }

    fn reset_theme(&mut self, id: &str) {
        self.status = if !self.config.remove_theme(id) {
            format!("{id} has no settings to reset")
        } else {
            match settings_cmd::save(&self.config) {
                Ok(()) => format!("saved: every setting of {id} is back to its default"),
                Err(e) => e,
            }
        };
    }

    fn status_is_saved(&self) -> bool {
        self.status.starts_with("saved")
    }

    // A confirmation fades; a warning stays until the user reads it and moves on.
    fn expire_status(&mut self) -> bool {
        let expired = self.status_is_saved() && self.status_at.elapsed() >= SAVED_FOR;
        if expired {
            self.status.clear();
        }
        expired
    }

    fn place(&self) -> (Option<usize>, Option<usize>, std::mem::Discriminant<Mode>) {
        let field = match self.mode {
            Mode::Form { selected, .. } => Some(selected),
            _ => None,
        };
        (
            self.list.selected(),
            field,
            std::mem::discriminant(&self.mode),
        )
    }

    fn on_key(&mut self, k: KeyEvent) -> Outcome {
        let place = self.place();
        let shown = std::mem::take(&mut self.status);
        let outcome = self.handle(k);
        if !self.status.is_empty() {
            self.status_at = Instant::now();
        } else if self.place() == place {
            self.status = shown;
        }
        outcome
    }

    fn handle(&mut self, k: KeyEvent) -> Outcome {
        match std::mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Browse => self.on_browse(k),
            // Terminals turn the scroll wheel into arrow keys, so only Esc closes the list.
            Mode::Help if k.code != KeyCode::Esc => {
                self.mode = Mode::Help;
                Outcome::Stay
            }
            Mode::Help => Outcome::Stay,
            Mode::Search => {
                self.mode = Mode::Search;
                match k.code {
                    KeyCode::Enter => self.mode = Mode::Browse,
                    KeyCode::Esc => {
                        self.mode = Mode::Browse;
                        self.query.clear();
                        self.refilter();
                    }
                    KeyCode::Backspace => {
                        self.query.pop();
                        self.refilter();
                    }
                    KeyCode::Down => self.move_selection(Move::Next),
                    KeyCode::Up => self.move_selection(Move::Prev),
                    KeyCode::Char(c) => {
                        self.query.push(c);
                        self.refilter();
                    }
                    _ => {}
                }
                Outcome::Stay
            }
            Mode::Form { selected, editing } => {
                self.mode = Mode::Form { selected, editing };
                self.on_form(k);
                Outcome::Stay
            }
            Mode::Input { prompt, mut buffer } => match k.code {
                KeyCode::Esc => Outcome::Stay,
                KeyCode::Enter => {
                    let id = self.selected().map(|t| t.id.clone()).unwrap_or_default();
                    let path = buffer.trim();
                    let path = path.strip_prefix("~/").map_or(path.to_string(), |rest| {
                        format!("{}/{rest}", std::env::var("HOME").unwrap_or_default())
                    });
                    Outcome::Run {
                        args: vec!["font".into(), "import".into(), id, path],
                        pause: false,
                    }
                }
                KeyCode::Backspace => {
                    buffer.pop();
                    self.mode = Mode::Input { prompt, buffer };
                    Outcome::Stay
                }
                KeyCode::Char(c) => {
                    buffer.push(c);
                    self.mode = Mode::Input { prompt, buffer };
                    Outcome::Stay
                }
                _ => {
                    self.mode = Mode::Input { prompt, buffer };
                    Outcome::Stay
                }
            },
        }
    }

    fn needs(&mut self, available: Result<(), String>, args: &[&str]) -> Outcome {
        match available {
            Ok(()) => Outcome::Run {
                args: args.iter().map(|s| s.to_string()).collect(),
                pause: false,
            },
            Err(reason) => {
                self.status = reason;
                Outcome::Stay
            }
        }
    }

    fn on_browse(&mut self, k: KeyEvent) -> Outcome {
        match k.code {
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                return Outcome::Stay;
            }
            KeyCode::Esc if !self.query.is_empty() => {
                self.query.clear();
                self.refilter();
                return Outcome::Stay;
            }
            _ => {}
        }
        let Some(theme) = self.selected() else {
            self.quit = k.code == KeyCode::Char('q');
            return Outcome::Stay;
        };
        let id = theme.id.clone();
        let missing_fonts = theme.missing_fonts();
        let env = &self.env;
        let (wayland, helper) = (env.wayland.clone(), env.helper.clone());
        let sddm_preview = env.sddm_preview();
        match k.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(Move::Next),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(Move::Prev),
            KeyCode::Home | KeyCode::Char('g') => self.move_selection(Move::First),
            KeyCode::End | KeyCode::Char('G') => self.move_selection(Move::Last),
            KeyCode::Enter | KeyCode::Right => {
                self.mode = Mode::Form {
                    selected: 0,
                    editing: None,
                }
            }
            KeyCode::Char('l') => {
                self.change(&Key::Theme(darwan_core::config::Target::Lock), Some(&id))
            }
            KeyCode::Char('p') => return self.needs(wayland, &["preview", &id]),
            KeyCode::Char('P') => return self.needs(wayland, &["preview", &id, "--sddm"]),
            KeyCode::Char('a') => return self.needs(wayland, &["preview", &id, "--saver"]),
            KeyCode::Char('L') => return self.needs(wayland, &["lock", &id]),
            KeyCode::Char('s') => return self.needs(helper, &["sddm", "apply", &id]),
            KeyCode::Char('S') => return self.needs(sddm_preview, &["sddm", "preview", &id]),
            KeyCode::Char('c') => {
                return Outcome::Run {
                    args: vec!["check".into(), id],
                    pause: true,
                };
            }
            KeyCode::Char('d') => {
                return Outcome::Run {
                    args: vec!["doctor".into()],
                    pause: true,
                };
            }
            KeyCode::Char('f') if missing_fonts == 0 => {
                self.status = format!("{id} has no missing fonts")
            }
            KeyCode::Char('f') => match helper {
                Ok(()) => {
                    self.mode = Mode::Input {
                        prompt: format!("Font file for {id}"),
                        buffer: String::new(),
                    }
                }
                Err(reason) => {
                    self.status =
                        format!("{reason}; in a checkout, put the file in themes/{id}/font/")
                }
            },
            _ => {}
        }
        Outcome::Stay
    }

    fn on_form(&mut self, k: KeyEvent) {
        let armed = std::mem::take(&mut self.reset_armed);
        let Mode::Form { selected, editing } = &mut self.mode else {
            return;
        };
        let Some(theme) = theme_at(&self.catalog, &self.rows, self.list.selected()) else {
            return;
        };
        let id = theme.id.clone();
        let fields = form::fields(theme, &self.config);
        let Some(field) = fields.get(*selected).cloned() else {
            return;
        };

        if let Some(buf) = editing {
            match k.code {
                KeyCode::Esc => *editing = None,
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char(c) => buf.push(c),
                KeyCode::Enter => {
                    let value = std::mem::take(buf);
                    *editing = None;
                    let empty_text = value.is_empty()
                        && matches!(
                            field.kind,
                            FieldKind::Text
                                | FieldKind::Media(_)
                                | FieldKind::Font
                                | FieldKind::Color { .. }
                        );
                    self.change(&field.key, if empty_text { None } else { Some(&value) });
                }
                _ => {}
            }
            return;
        }

        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Browse,
            KeyCode::Down | KeyCode::Char('j') => {
                *selected = (*selected + 1).min(fields.len().saturating_sub(1))
            }
            KeyCode::Up | KeyCode::Char('k') => *selected = selected.saturating_sub(1),
            // Everything saves at once here, so a whole-theme reset asks for a second press.
            KeyCode::Char('R') if armed => self.reset_theme(&id),
            KeyCode::Char('R') => {
                self.reset_armed = true;
                self.status = format!("press R again to reset every setting of {id}");
            }
            _ if field.disabled.is_some() => {
                if !matches!(k.code, KeyCode::Down | KeyCode::Up) {
                    self.status =
                        format!("{}: {}", field.label, field.disabled.unwrap_or_default());
                }
            }
            KeyCode::Char('r') => self.change(&field.key, None),
            KeyCode::Right
            | KeyCode::Char('l')
            | KeyCode::Char(' ')
            | KeyCode::Left
            | KeyCode::Char('h') => {
                let forward = !matches!(k.code, KeyCode::Left | KeyCode::Char('h'));
                if let Some(v) = step(&field, forward) {
                    self.change(&field.key, Some(&v));
                }
            }
            // Colours, backgrounds and fonts are typed: a hex value, a path or a family.
            KeyCode::Enter
                if matches!(
                    field.kind,
                    FieldKind::Color { .. } | FieldKind::Media(_) | FieldKind::Font
                ) =>
            {
                *editing = Some(field.value.clone())
            }
            KeyCode::Enter => match step(&field, true) {
                Some(v) => self.change(&field.key, Some(&v)),
                None => *editing = Some(field.value.clone()),
            },
            _ => {}
        }
    }
}

// Commands that only show a window return straight to the TUI unless they fail.
fn run_cli(args: &[String], pause: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    println!("$ darwan {}\n", args.join(" "));
    let status = Command::new(exe)
        .args(args)
        .status()
        .map_err(|e| e.to_string())?;
    if pause || !status.success() {
        print!(
            "\n{} Press Enter to go back.",
            if status.success() { "Done." } else { "Failed." }
        );
        std::io::stdout().flush().ok();
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line).ok();
    }
    Ok(())
}

fn load(paths: &Paths) -> Result<(Catalog, UserConfig), String> {
    let (catalog, _) =
        Catalog::load(&paths.themes()).map_err(|e| format!("{}: {e}", paths.themes().display()))?;
    let path = paths::config_file();
    let config = UserConfig::load(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((catalog, config))
}

pub fn run(paths: &Paths) -> Result<ExitCode, String> {
    let (catalog, config) = load(paths)?;
    let mut terminal = ratatui::init();
    let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
    let previews = Previews::start(picker.clone());
    let mut app = App::new(
        catalog,
        config,
        Environment::detect(crate::session::WaylandSession::discover().map(drop)),
        picker,
        previews,
        Look::detect(),
    );

    let mut dirty = true;
    let mut was_settled = true;
    let result = loop {
        if dirty {
            if let Err(e) = terminal.draw(|f| view::draw(f, &mut app)) {
                break Err(e.to_string());
            }
            dirty = false;
        }
        dirty |= app.previews.poll();
        dirty |= app.expire_status();
        let settled = app.settled();
        dirty |= settled && !was_settled;
        was_settled = settled;

        let key = match event::poll(Duration::from_millis(30)) {
            Ok(false) => continue,
            Ok(true) => match event::read() {
                Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => k,
                Ok(Event::Resize(..)) => {
                    dirty = true;
                    continue;
                }
                Ok(_) => continue,
                Err(e) => break Err(e.to_string()),
            },
            Err(e) => break Err(e.to_string()),
        };
        dirty = true;
        if let Outcome::Run { args, pause } = app.on_key(key) {
            ratatui::restore();
            let ran = run_cli(&args, pause);
            terminal = ratatui::init();
            app.previews.clear();
            app.env = Environment::detect(crate::session::WaylandSession::discover().map(drop));
            match (ran, load(paths)) {
                (Ok(()), Ok((catalog, config))) => {
                    (app.catalog, app.config) = (catalog, config);
                    app.refilter();
                }
                (Err(e), _) | (_, Err(e)) => app.status = e,
            }
        }
        if app.quit {
            break Ok(ExitCode::SUCCESS);
        }
    };
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::KeyModifiers;
    use std::path::Path;

    fn app(env: Environment) -> App {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let catalog = Catalog::load(&root).unwrap().0;
        App::new(
            catalog,
            UserConfig::default(),
            env,
            Picker::halfblocks(),
            Previews::disabled(),
            Look::plain(),
        )
    }

    fn no_sddm() -> Environment {
        Environment {
            wayland: Ok(()),
            sddm: Err("SDDM (Qt 6) is not installed".into()),
            helper: Err("SDDM (Qt 6) is not installed".into()),
        }
    }

    fn screen(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
        terminal.draw(|f| view::draw(f, app)).unwrap();
        let buf = terminal.backend().buffer().clone();
        buf.content()
            .iter()
            .map(|c| c.symbol())
            .collect::<Vec<_>>()
            .chunks(140)
            .map(|r| r.concat())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn press(app: &mut App, code: KeyCode) -> Outcome {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn select(app: &mut App, id: &str) {
        let i = app
            .rows
            .iter()
            .position(|r| matches!(r, ListRow::Theme(j) if app.catalog.themes()[*j].id == id))
            .unwrap();
        app.list.select(Some(i));
    }

    #[test]
    fn unavailable_actions_say_why_instead_of_running() {
        let mut a = app(no_sddm());
        assert!(screen(&mut a).contains("SDDM (Qt 6) is not installed"));
        assert!(matches!(press(&mut a, KeyCode::Char('s')), Outcome::Stay));
        assert_eq!(a.status, "SDDM (Qt 6) is not installed");
    }

    #[test]
    fn available_actions_run_the_matching_cli_command() {
        let mut a = app(no_sddm());
        select(&mut a, "osu");
        match press(&mut a, KeyCode::Char('p')) {
            Outcome::Run { args, pause } => {
                assert_eq!(args, ["preview", "osu"]);
                assert!(!pause, "a preview returns straight to the TUI");
            }
            Outcome::Stay => panic!("preview should run"),
        }
    }

    #[test]
    fn the_settings_form_shows_why_an_option_is_disabled() {
        let mut a = app(no_sddm());
        select(&mut a, "terraria");
        press(&mut a, KeyCode::Enter);
        let s = screen(&mut a);
        assert!(s.contains("Settings · Terraria"), "{s}");
        assert!(s.contains("only when Background is Fixed"), "{s}");
        assert!(s.contains("Terraria doesn't support clock format"), "{s}");
    }

    #[test]
    fn marks_show_the_lock_and_sddm_themes() {
        let mut a = app(no_sddm());
        a.config = UserConfig::parse("[lock]\ntheme = \"osu\"\n[sddm]\ntheme = \"osu\"\n").unwrap();
        select(&mut a, "osu");
        assert!(screen(&mut a).contains("LS osu!"));
    }

    #[test]
    fn help_lists_the_keys_moved_off_the_bar_with_their_reasons() {
        let mut a = app(no_sddm());
        press(&mut a, KeyCode::Char('?'));
        let s = screen(&mut a);
        assert!(s.contains("preview in SDDM's test mode"), "{s}");
        assert!(s.contains("doctor: check the system"), "{s}");
        assert!(s.contains("SDDM (Qt 6) is not installed"), "{s}");
        for code in [
            KeyCode::Down,
            KeyCode::Up,
            KeyCode::Char('q'),
            KeyCode::Enter,
        ] {
            press(&mut a, code);
            assert!(
                matches!(a.mode, Mode::Help),
                "{code:?} must not close the key list"
            );
        }
        assert!(!a.quit);
        press(&mut a, KeyCode::Esc);
        assert!(matches!(a.mode, Mode::Browse));
    }

    // One stray key must not wipe a theme's settings: R asks, a second R does it.
    #[test]
    fn resetting_a_whole_theme_takes_two_presses_of_r() {
        let mut a = app(no_sddm());
        select(&mut a, "material-you");
        press(&mut a, KeyCode::Enter);
        press(&mut a, KeyCode::Char('R'));
        assert!(a.status.starts_with("press R again"), "{}", a.status);
        press(&mut a, KeyCode::Down);
        press(&mut a, KeyCode::Char('R'));
        assert!(
            a.status.starts_with("press R again"),
            "another key in between disarms it"
        );
        press(&mut a, KeyCode::Char('R'));
        assert_eq!(a.status, "material-you has no settings to reset");
    }

    #[test]
    fn right_arrow_opens_the_settings_like_enter() {
        let mut a = app(no_sddm());
        select(&mut a, "terraria");
        press(&mut a, KeyCode::Right);
        assert!(matches!(
            a.mode,
            Mode::Form {
                selected: 0,
                editing: None
            }
        ));
    }

    #[test]
    fn a_warning_stays_until_the_user_moves_on() {
        let mut a = app(no_sddm());
        select(&mut a, "osu");
        press(&mut a, KeyCode::Char('s'));
        press(&mut a, KeyCode::Char('x'));
        a.status_at = Instant::now().checked_sub(SAVED_FOR * 2).unwrap();
        a.expire_status();
        assert_eq!(
            a.status, "SDDM (Qt 6) is not installed",
            "it outlives keys and time"
        );
        press(&mut a, KeyCode::Down);
        assert!(a.status.is_empty(), "moving to another theme clears it");

        press(&mut a, KeyCode::Char('s'));
        press(&mut a, KeyCode::Enter);
        assert!(a.status.is_empty(), "opening the settings clears it");
    }

    #[test]
    fn a_saved_confirmation_fades_on_its_own() {
        let mut a = app(no_sddm());
        a.status = "saved: lock.theme = osu".into();
        a.status_at = Instant::now();
        assert!(!a.expire_status(), "it stays long enough to read");
        press(&mut a, KeyCode::Char('x'));
        assert!(
            a.status.starts_with("saved"),
            "a key that does nothing keeps it"
        );
        a.status_at = Instant::now().checked_sub(SAVED_FOR).unwrap();
        assert!(a.expire_status());
        assert!(a.status.is_empty());
    }

    #[test]
    fn reports_pause_so_their_output_can_be_read() {
        let mut a = app(no_sddm());
        match press(&mut a, KeyCode::Char('d')) {
            Outcome::Run { args, pause } => assert!(pause && args == ["doctor"]),
            Outcome::Stay => panic!("doctor should run"),
        }
    }

    #[test]
    fn settings_show_customisation_headings_and_unsupported_reasons() {
        let mut a = app(no_sddm());
        select(&mut a, "pixel-rainyroom");
        press(&mut a, KeyCode::Right);
        let s = screen(&mut a);
        for heading in ["BACKGROUND", "APPEARANCE", "COLOURS"] {
            assert!(s.contains(heading), "{heading} missing:\n{s}");
        }
        assert!(
            s.contains("has one look"),
            "Rainy Room has no light/dark variant:\n{s}"
        );
    }

    #[test]
    fn search_filters_live_and_escape_clears_it_but_never_quits() {
        let mut a = app(no_sddm());
        press(&mut a, KeyCode::Char('/'));
        for c in "tape".chars() {
            press(&mut a, KeyCode::Char(c));
        }
        assert_eq!(a.selected().map(|t| t.id.as_str()), Some("clockwork/tape"));
        press(&mut a, KeyCode::Enter);
        assert!(screen(&mut a).contains("1 of 40"));
        press(&mut a, KeyCode::Esc);
        assert!(a.query.is_empty(), "Esc clears the search");
        press(&mut a, KeyCode::Esc);
        assert!(!a.quit, "Esc on the main list does nothing; only q quits");
        press(&mut a, KeyCode::Char('q'));
        assert!(a.quit);
    }
}
