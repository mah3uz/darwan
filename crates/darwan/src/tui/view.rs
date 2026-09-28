use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Clear, List, ListItem, Padding, Paragraph, Row, Table, Wrap,
};
use ratatui_image::StatefulImage;

use darwan_core::catalog::Theme;
use darwan_core::config::Target;
use darwan_core::form::{self, Field};
use darwan_core::manifest::Background;

use super::logic::display_value;
use super::previews::{Lookup, resize};
use super::style::Look;
use super::{App, Mode};
use darwan_core::gallery::{ListRow, display_name};

pub fn draw(f: &mut Frame, app: &mut App) {
    let [main, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Length(38), Constraint::Fill(1)]).areas(main);
    draw_list(f, app, left);
    match app.mode {
        Mode::Form { .. } => draw_form(f, app, right),
        _ => draw_browse(f, app, right),
    }
    draw_footer(f, app, footer);
    if matches!(app.mode, Mode::Help) {
        draw_help(f, app);
    }
}

fn draw_help(f: &mut Frame, app: &App) {
    let look = &app.look;
    let p = look.palette;
    let env = &app.env;
    let ok: Result<(), String> = Ok(());
    let sddm_preview = env.sddm_preview();
    let keys: [(&str, &str, &Result<(), String>); 16] = [
        ("↑ ↓  j k", "move", &ok),
        ("g G", "first / last theme", &ok),
        ("⏎ →", "settings for the theme", &ok),
        ("p", "preview as the lockscreen", &env.wayland),
        ("P", "preview with the SDDM layout", &env.wayland),
        ("a", "preview the screensaver", &env.wayland),
        ("l", "use as the lock theme", &ok),
        ("L", "lock now", &env.wayland),
        ("s", "apply to the SDDM login screen", &env.helper),
        ("S", "preview in SDDM's test mode", &sddm_preview),
        ("f", "import a missing font", &env.helper),
        ("c", "check the theme for QML errors", &ok),
        ("d", "doctor: check the system", &ok),
        ("/", "search by name or id", &ok),
        ("?", "this list", &ok),
        ("q", "quit", &ok),
    ];
    let rows: Vec<Row> = keys
        .iter()
        .map(|(k, action, available)| {
            let (key_style, text_style, note) = match available {
                Ok(()) => (
                    look.fg(p.accent).add_modifier(Modifier::BOLD),
                    look.fg(p.text),
                    String::new(),
                ),
                Err(reason) => (look.fg(p.muted), look.fg(p.muted), reason.clone()),
            };
            Row::new(vec![
                Span::styled(k.to_string(), key_style),
                Span::styled(action.to_string(), text_style),
                Span::styled(note, look.fg(p.muted).add_modifier(Modifier::ITALIC)),
            ])
        })
        .collect();
    let area = f.area();
    let width = area.width.min(96);
    let height = (keys.len() as u16 + 4).min(area.height);
    let popup = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    };
    f.render_widget(Clear, popup);
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(32),
            Constraint::Fill(1),
        ],
    )
    .block(panel(look, look.icons.details, "Keys", true).padding(Padding::uniform(1)));
    f.render_widget(table, popup);
}

fn panel<'a>(look: &Look, icon: &str, title: &str, focused: bool) -> Block<'a> {
    let p = look.palette;
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(look.fg(if focused { p.accent } else { p.border }))
        .title(Line::from(format!(" {icon} {title} ")).style(look.heading()))
        .padding(Padding::horizontal(1))
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let look = &app.look;
    let (p, i) = (look.palette, look.icons);
    let lock = app
        .config
        .theme(Target::Lock)
        .ok()
        .flatten()
        .map(str::to_string);
    let sddm = app
        .config
        .theme(Target::Sddm)
        .ok()
        .flatten()
        .map(str::to_string);
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .enumerate()
        .map(|(n, row)| match row {
            ListRow::Section { title, count } => {
                let header = Line::from(vec![
                    Span::styled(format!("{} {title}", i.section), look.heading()),
                    Span::styled(format!("  {count}"), look.fg(p.muted)),
                ]);
                if n == 0 {
                    ListItem::new(header)
                } else {
                    ListItem::new(vec![Line::raw(""), header])
                }
            }
            ListRow::Theme(j) => {
                let t = &app.catalog.themes()[*j];
                let mark = |on: bool, icon: &'static str, color| {
                    if on {
                        Span::styled(icon, look.fg(color))
                    } else {
                        Span::raw(" ")
                    }
                };
                let name = if t.manifest.family.is_some() {
                    t.manifest.name.clone()
                } else {
                    display_name(t)
                };
                ListItem::new(Line::from(vec![
                    mark(lock.as_deref() == Some(t.id.as_str()), i.lock, p.lock),
                    mark(sddm.as_deref() == Some(t.id.as_str()), i.sddm, p.sddm),
                    Span::raw(" "),
                    Span::styled(name, look.fg(p.text)),
                ]))
            }
        })
        .collect();
    let focused = matches!(app.mode, Mode::Browse | Mode::Input { .. } | Mode::Search);
    let total = app.catalog.themes().len();
    let shown = app
        .rows
        .iter()
        .filter(|r| matches!(r, ListRow::Theme(_)))
        .count();
    let searching = matches!(app.mode, Mode::Search);
    let count = if app.query.is_empty() {
        format!(" {total} themes ")
    } else {
        format!(" {shown} of {total} ")
    };
    let count = Line::from(count)
        .style(look.fg(p.muted))
        .alignment(Alignment::Right);
    let mut block = panel(look, i.themes, "Themes", focused).title_bottom(count);
    if searching || !app.query.is_empty() {
        let cursor = if searching { "▏" } else { "" };
        block = block.title(
            Line::from(format!(" / {}{cursor} ", app.query))
                .style(look.fg(p.warn))
                .alignment(Alignment::Right),
        );
    }
    if shown == 0 {
        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new("no themes match").style(look.fg(p.muted)),
            inner,
        );
        return;
    }
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::new()
                .bg(p.surface)
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(format!("{} ", i.pointer));
    f.render_stateful_widget(list, area, &mut app.list);
}

// The preview keeps 16:9 in pixels, which depends on the terminal's cell size.
fn preview_rows(app: &App, width: u16) -> u16 {
    let fs = app.picker.font_size();
    let (fw, fh) = (fs.width.max(1) as u32, fs.height.max(1) as u32);
    ((width as u32 * fw * 9) / (16 * fh)).clamp(4, u16::MAX as u32) as u16
}

fn draw_browse(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(theme) = app.selected() else { return };
    let (id, path) = (theme.id.clone(), theme.preview.clone());
    let lines = details(&app.look, theme, app);

    // The preview takes whatever the details don't need, up to a full-width 16:9 image.
    let natural = preview_rows(app, area.width.saturating_sub(4)) + 2;
    let details_rows = lines.len() as u16 + 2;
    let preview_height = area
        .height
        .saturating_sub(details_rows)
        .clamp(8.min(area.height), natural.max(8));
    let [top, bottom] =
        Layout::vertical([Constraint::Length(preview_height), Constraint::Fill(1)]).areas(area);

    let block = panel(&app.look, app.look.icons.preview, "Preview", false);
    let inner = block.inner(top);
    f.render_widget(block, top);
    let muted = app.look.fg(app.look.palette.muted);
    let image_area = centered_16_9(app, inner);
    let size = image_area.as_size();
    let settled = app.settled();
    // Terminal graphics sit above text, so an overlay can't cover the image; leave it out instead.
    let overlay = matches!(app.mode, Mode::Help);
    let placeholder = match app.previews.get(&id, path, size, settled) {
        _ if overlay => None,
        Lookup::Ready(state) => {
            f.render_stateful_widget(StatefulImage::default().resize(resize()), image_area, state);
            None
        }
        Lookup::Loading => Some("loading preview…"),
        Lookup::Missing => Some("no preview"),
    };
    if let Some(text) = placeholder {
        let [_, middle, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);
        f.render_widget(
            Paragraph::new(text)
                .style(muted)
                .alignment(Alignment::Center),
            middle,
        );
    }

    let block = panel(&app.look, app.look.icons.details, "Details", false);
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        bottom,
    );
}

fn centered_16_9(app: &App, area: Rect) -> Rect {
    let fs = app.picker.font_size();
    let (fw, fh) = (fs.width.max(1) as u32, fs.height.max(1) as u32);
    let width = ((area.height as u32 * fh * 16) / (9 * fw)).min(area.width as u32) as u16;
    let height = preview_rows(app, width).min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

fn section<'a>(look: &Look, icon: &str, title: &str) -> Vec<Line<'a>> {
    vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!("{icon} {title} "), look.heading()),
            Span::styled("─".repeat(40), look.fg(look.palette.border)),
        ]),
    ]
}

fn details<'a>(look: &Look, theme: &Theme, app: &App) -> Vec<Line<'a>> {
    let (p, i) = (look.palette, look.icons);
    let m = &theme.manifest;
    let (bg_icon, bg) = match m.background {
        Background::Video => (i.video, "video"),
        Background::Image => (i.image, "image"),
        Background::Color => (i.color, "colour"),
    };
    let mut lines = vec![
        Line::from(Span::styled(
            display_name(theme),
            look.fg(p.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(format!("{} {}", i.author, m.author), look.fg(p.subtext)),
            Span::raw("   "),
            Span::styled(format!("{bg_icon} {bg} background"), look.fg(p.subtext)),
            Span::styled(format!("   {}", theme.id), look.fg(p.muted)),
        ]),
    ];
    let mut badges = Vec::new();
    if app.config.theme(Target::Lock).ok().flatten() == Some(theme.id.as_str()) {
        badges.push(Span::styled(
            format!("{} lock theme", i.lock),
            look.fg(p.lock).add_modifier(Modifier::BOLD),
        ));
        badges.push(Span::raw("   "));
    }
    if app.config.theme(Target::Sddm).ok().flatten() == Some(theme.id.as_str()) {
        badges.push(Span::styled(
            format!("{} login theme", i.sddm),
            look.fg(p.sddm).add_modifier(Modifier::BOLD),
        ));
    }
    if !badges.is_empty() {
        lines.push(Line::from(badges));
    }

    if !m.fonts.is_empty() {
        lines.extend(section(look, i.fonts, "Fonts"));
        for font in &m.fonts {
            let installed = theme.dir.join("font").join(&font.file).is_file();
            lines.push(if installed {
                Line::from(vec![
                    Span::styled(format!("{} ", i.ok), look.fg(p.ok)),
                    Span::styled(font.family.clone(), look.fg(p.text)),
                    Span::styled("  installed", look.fg(p.muted)),
                ])
            } else {
                let source = if font.url.is_empty() {
                    "no public source".to_string()
                } else {
                    font.url.clone()
                };
                Line::from(vec![
                    Span::styled(format!("{} ", i.warn), look.fg(p.warn)),
                    Span::styled(font.family.clone(), look.fg(p.warn)),
                    Span::styled(
                        format!("  missing · {} · {source} · press f", font.license),
                        look.fg(p.subtext),
                    ),
                ])
            });
        }
    }

    lines.extend(section(look, i.settings, "Settings"));
    for field in form::fields(theme, &app.config) {
        lines.push(setting_line(look, &field));
    }
    lines
}

fn setting_line<'a>(look: &Look, field: &Field) -> Line<'a> {
    let p = look.palette;
    match &field.disabled {
        Some(reason) => Line::from(vec![
            Span::styled(format!("{:<22}", field.label), look.fg(p.muted)),
            Span::styled(
                format!("{} {reason}", look.icons.disabled),
                look.fg(p.muted).add_modifier(Modifier::ITALIC),
            ),
        ]),
        None => Line::from(vec![
            Span::styled(format!("{:<22}", field.label), look.fg(p.subtext)),
            Span::styled(
                display_value(field),
                look.fg(if field.is_set { p.accent } else { p.text }),
            ),
            Span::styled(
                if field.is_set { "" } else { "  default" },
                look.fg(p.muted),
            ),
        ]),
    }
}

fn draw_form(f: &mut Frame, app: &mut App, area: Rect) {
    let Mode::Form {
        selected,
        ref editing,
    } = app.mode
    else {
        return;
    };
    let Some(theme) = app.selected() else { return };
    let look = &app.look;
    let p = look.palette;
    let fields = form::fields(theme, &app.config);
    let mut rows: Vec<Row> = Vec::new();
    for (n, field) in fields.iter().enumerate() {
        if field.group.is_some() && (n == 0 || fields[n - 1].group != field.group) {
            rows.push(Row::new(vec![Span::styled(
                field.group.unwrap_or_default().to_uppercase(),
                look.fg(p.accent).add_modifier(Modifier::BOLD),
            )]));
        }
        rows.push({
            let value = match editing {
                Some(buf) if n == selected => format!("{buf}▏"),
                _ => display_value(field),
            };
            let (note, note_style) = match &field.disabled {
                Some(reason) => (
                    format!("{} {reason}", look.icons.disabled),
                    look.fg(p.muted).add_modifier(Modifier::ITALIC),
                ),
                None if field.is_set => ("set".into(), look.fg(p.accent)),
                None => ("default".into(), look.fg(p.muted)),
            };
            let dim = field.disabled.is_some();
            let value_color = if dim {
                p.muted
            } else if field.is_set {
                p.accent
            } else {
                p.text
            };
            let row = Row::new(vec![
                Span::styled(
                    field.label.clone(),
                    look.fg(if dim { p.muted } else { p.subtext }),
                ),
                Span::styled(value, look.fg(value_color)),
                Span::styled(note, note_style),
            ]);
            if n == selected {
                row.style(Style::new().bg(p.surface).add_modifier(Modifier::BOLD))
            } else {
                row
            }
        });
    }
    let title = format!("Settings · {}", display_name(theme));
    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Length(26),
            Constraint::Fill(1),
        ],
    )
    .header(
        Row::new(vec!["Setting", "Value", ""])
            .style(look.fg(p.muted))
            .bottom_margin(1),
    )
    .block(panel(look, look.icons.settings, &title, true).padding(Padding::uniform(1)));
    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let look = &app.look;
    let p = look.palette;
    let key = |k: &'static str,
               label: &'static str,
               available: &Result<(), String>|
     -> Vec<Span<'static>> {
        let (ks, ls) = match available {
            Ok(()) => (
                look.fg(p.accent).add_modifier(Modifier::BOLD),
                look.fg(p.subtext),
            ),
            Err(_) => (look.fg(p.muted), look.fg(p.muted)),
        };
        vec![
            Span::styled(format!(" {k} "), ks),
            Span::styled(format!("{label}  "), ls),
        ]
    };
    let ok = Ok(());
    let env = &app.env;
    let hints: Vec<Span> = match &app.mode {
        Mode::Browse => [
            key("↑↓", "move", &ok),
            key("⏎ →", "settings", &ok),
            key("p", "preview", &env.wayland),
            key("l", "use for lock", &ok),
            key("L", "lock now", &env.wayland),
            key("s", "apply to SDDM", &env.helper),
            key("f", "import font", &env.helper),
            key("/", "search", &ok),
            key("?", "all keys", &ok),
            key("q", "quit", &ok),
        ]
        .concat(),
        Mode::Help => key("esc", "close", &ok),
        Mode::Search => [
            key("type", "to filter", &ok),
            key("↑↓", "move", &ok),
            key("⏎", "keep", &ok),
            key("esc", "clear", &ok),
        ]
        .concat(),
        Mode::Form {
            editing: Some(_), ..
        } => [key("⏎", "save", &ok), key("esc", "cancel", &ok)].concat(),
        Mode::Form { .. } => [
            key("↑↓", "move", &ok),
            key("←→ space", "change", &ok),
            key("⏎", "edit", &ok),
            key("r", "reset to default", &ok),
            key("R R", "reset the whole theme", &ok),
            key("esc", "back", &ok),
        ]
        .concat(),
        Mode::Input { prompt, buffer } => vec![
            Span::styled(format!(" {prompt}: "), look.heading()),
            Span::styled(format!("{buffer}▏"), look.fg(p.text)),
        ],
    };
    let status = if app.status.is_empty() {
        let unavailable: Vec<&str> = [&env.wayland, &env.helper]
            .into_iter()
            .filter_map(|r| r.as_ref().err().map(String::as_str))
            .collect();
        Line::from(format!(" {}", unavailable.join(" · "))).style(look.fg(p.muted))
    } else {
        let color = if app.status_is_saved() { p.ok } else { p.warn };
        Line::from(format!(" {}", app.status)).style(look.fg(color))
    };
    let [keys, status_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    f.render_widget(
        Paragraph::new(Line::from(hints)).wrap(Wrap { trim: true }),
        keys,
    );
    f.render_widget(Paragraph::new(status), status_area);
}
