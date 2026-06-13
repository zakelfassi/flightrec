use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{App, Screen};
use super::theme;
use crate::diff::ChangeType;

const HELP_TIMELINE: &str = " j/k ↑/↓ navigate  Enter open  r refresh  g/G top/bottom  q quit";
const HELP_DETAIL: &str = " j/k ↑/↓ navigate  Enter open file diff  Esc/Backspace back  q quit";
const HELP_FILE: &str = " j/k ↑/↓ scroll  Esc/Backspace back  q quit";

pub fn render(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Timeline => render_timeline(f, app),
        Screen::DiffDetail => render_detail(f, app),
        Screen::FileDiff => render_file(f, app),
    }
}

fn help_bar<'a>(text: &'a str) -> Paragraph<'a> {
    Paragraph::new(text).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )
}

fn split_body_help(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    (chunks[0], chunks[1])
}

fn change_color(ct: &ChangeType) -> Color {
    match ct {
        ChangeType::Added => theme::ADDED,
        ChangeType::Removed => theme::REMOVED,
        ChangeType::Modified => theme::MODIFIED,
        ChangeType::Renamed => theme::RENAMED,
    }
}

/// Truncate a diff_id for compact display. Strips a leading `diff-` prefix
/// (if present) and keeps at most 15 characters of the remainder, appending
/// `…` when the remainder is longer.
fn short_diff_id(id: &str) -> String {
    const MAX: usize = 15;
    let body = id.strip_prefix("diff-").unwrap_or(id);
    if body.len() <= MAX {
        body.to_string()
    } else {
        format!("{}…", &body[..MAX])
    }
}

// ── Timeline ─────────────────────────────────────────────────────────────────

fn render_timeline(f: &mut Frame, app: &App) {
    let (body, help) = split_body_help(f.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" flightrec — timeline ");

    let inner = block.inner(body);
    f.render_widget(block, body);

    if app.timeline.is_empty() {
        let msg = Paragraph::new("no diffs yet — run `flightrec watch`")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
    } else {
        let items: Vec<ListItem> = app
            .timeline
            .iter()
            .map(|entry| {
                let count_str = format!("{} change(s)", entry.change_count);
                let id_str = short_diff_id(&entry.diff_id);
                let short = entry.short_summary.as_deref().unwrap_or("").to_string();

                let main_line = if short.is_empty() {
                    Line::from(vec![
                        Span::styled(
                            id_str,
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                        ),
                        Span::raw("  "),
                        Span::styled(entry.created_at.clone(), Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::raw(count_str),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(
                            id_str,
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                        ),
                        Span::raw("  "),
                        Span::styled(entry.created_at.clone(), Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::raw(count_str),
                        Span::raw("  "),
                        Span::styled(
                            short,
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ])
                };

                ListItem::new(main_line)
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(app.timeline_selected));

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, inner, &mut state);
    }

    f.render_widget(help_bar(HELP_TIMELINE), help);
}

// ── DiffDetail ────────────────────────────────────────────────────────────────

fn render_detail(f: &mut Frame, app: &App) {
    let (body, help) = split_body_help(f.area());

    let diff_id = app
        .timeline
        .get(app.timeline_selected)
        .map(|e| e.diff_id.as_str())
        .unwrap_or("diff");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {diff_id} — changes "));

    let inner = block.inner(body);
    f.render_widget(block, body);

    if app.detail_changes.is_empty() {
        let msg =
            Paragraph::new("no changes in this diff").style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
    } else {
        let items: Vec<ListItem> = app
            .detail_changes
            .iter()
            .map(|change| {
                let sym = App::change_symbol(&change.change_type);
                let color = change_color(&change.change_type);
                let label = if let Some(from) = &change.renamed_from {
                    format!("{from} → {}", change.path)
                } else {
                    change.path.clone()
                };
                let line = Line::from(vec![
                    Span::styled(
                        format!("{sym} "),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(label),
                ]);
                ListItem::new(line)
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(app.detail_selected));

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, inner, &mut state);
    }

    f.render_widget(help_bar(HELP_DETAIL), help);
}

// ── FileDiff ─────────────────────────────────────────────────────────────────

fn render_file(f: &mut Frame, app: &App) {
    let (body, help) = split_body_help(f.area());

    let path = app
        .detail_changes
        .get(app.detail_selected)
        .map(|c| c.path.as_str())
        .unwrap_or("file");

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {path} "));

    let inner = block.inner(body);
    f.render_widget(block, body);

    if app.file_lines.is_empty() {
        let msg =
            Paragraph::new("(no diff text available)").style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, inner);
    } else {
        let lines: Vec<Line> = app
            .file_lines
            .iter()
            .skip(app.file_scroll)
            .map(|raw| colorize_diff_line(raw.as_str()))
            .collect();

        let para = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(para, inner);
    }

    f.render_widget(help_bar(HELP_FILE), help);
}

fn colorize_diff_line(line: &str) -> Line<'static> {
    let (style, text) = if line.starts_with('+') && !line.starts_with("+++") {
        (Style::default().fg(theme::ADDED), line.to_string())
    } else if line.starts_with('-') && !line.starts_with("---") {
        (Style::default().fg(theme::REMOVED), line.to_string())
    } else if line.starts_with("@@") {
        (Style::default().fg(theme::MODIFIED), line.to_string())
    } else {
        (Style::default(), line.to_string())
    };
    Line::from(Span::styled(text, style))
}
