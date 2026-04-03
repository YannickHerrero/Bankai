use chrono::{DateTime, Datelike, Local, TimeZone, Weekday};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, DashboardSection};

pub fn render(app: &App, frame: &mut Frame) {
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(frame.area());

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    render_watching(app, frame, outer[0]);
    render_calendar(app, frame, right[0]);
    render_updates(app, frame, right[1]);
}

fn section_block<'a>(app: &'a App, section: DashboardSection, title: &'a str) -> Block<'a> {
    let focused = app.dashboard_section == section;
    if focused {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(title)
    }
}

fn render_watching(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, DashboardSection::Watching, " Currently Watching ");

    if app.watching_list.is_empty() {
        let paragraph = Paragraph::new("No anime in watching list").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .watching_list
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = i == app.watching_scroll;
            let title = &entry.media.title.romaji;
            let total = entry
                .media
                .episodes
                .map(|e| e.to_string())
                .unwrap_or_else(|| "?".to_string());
            let score_str = if entry.score > 0.0 {
                format!("  ★ {}", entry.score as u8)
            } else {
                String::new()
            };
            let secondary_color = if selected { Color::Gray } else { Color::DarkGray };
            let score_color = if selected { Color::Yellow } else { Color::Yellow };
            let line = Line::from(vec![
                Span::styled(
                    format!(" {title}"),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("  ({}/{})", entry.progress, total),
                    Style::default().fg(secondary_color),
                ),
                Span::styled(score_str, Style::default().fg(score_color)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.watching_scroll));
    frame.render_stateful_widget(list, area, &mut state);
}

const WEEKDAYS: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

const DAY_NAMES: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

fn render_calendar(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, DashboardSection::Calendar, " Weekly Calendar ");

    let now = Local::now();
    let today_weekday = now.weekday();
    let days_since_monday = today_weekday.num_days_from_monday() as i64;
    let monday = (now - chrono::Duration::days(days_since_monday))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let sunday_end = monday + chrono::Duration::days(7);

    let monday_ts = Local.from_local_datetime(&monday).unwrap().timestamp();
    let sunday_ts = Local.from_local_datetime(&sunday_end).unwrap().timestamp();

    // Group airing shows by weekday
    let mut by_day: [Vec<(String, String)>; 7] = Default::default();

    for entry in &app.watching_list {
        if let Some(ref airing) = entry.media.next_airing_episode {
            if airing.airing_at >= monday_ts && airing.airing_at < sunday_ts {
                let dt: DateTime<Local> = Local.timestamp_opt(airing.airing_at, 0).unwrap();
                let day_idx = dt.weekday().num_days_from_monday() as usize;
                let time_str = dt.format("%H:%M").to_string();
                let label = format!(
                    "{} ep.{}",
                    entry.media.title.romaji, airing.episode
                );
                by_day[day_idx].push((label, time_str));
            }
        }
    }

    let has_any = by_day.iter().any(|d| !d.is_empty());

    if !has_any {
        let paragraph = Paragraph::new("No shows airing this week").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    let mut item_idx: usize = 0;
    for (i, day_shows) in by_day.iter().enumerate() {
        let is_today = WEEKDAYS[i] == today_weekday;
        let day_style = if is_today {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!(" {}", DAY_NAMES[i]),
            day_style,
        ))));
        item_idx += 1;

        if day_shows.is_empty() {
            let selected = item_idx == app.calendar_scroll;
            let color = if selected { Color::Gray } else { Color::DarkGray };
            items.push(ListItem::new(Line::from(Span::styled(
                "   --",
                Style::default().fg(color),
            ))));
            item_idx += 1;
        } else {
            for (label, time) in day_shows {
                let selected = item_idx == app.calendar_scroll;
                let time_color = if selected { Color::Gray } else { Color::DarkGray };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("   {label}"), Style::default().fg(Color::White)),
                    Span::styled(format!("  {time}"), Style::default().fg(time_color)),
                ])));
                item_idx += 1;
            }
        }
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    state.select(Some(app.calendar_scroll));
    frame.render_stateful_widget(list, area, &mut state);
}

fn relative_time(timestamp: i64) -> String {
    let now = Local::now().timestamp();
    let diff = now - timestamp;
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn render_updates(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, DashboardSection::Updates, " Last Updates ");

    if app.recent_activity.is_empty() {
        let paragraph = Paragraph::new("No recent activity").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .recent_activity
        .iter()
        .enumerate()
        .map(|(i, activity)| {
            let selected = i == app.updates_scroll;
            let progress_str = activity
                .progress
                .as_ref()
                .map(|p| format!(" {p} of"))
                .unwrap_or_default();
            let title = &activity.media.title.romaji;
            let ago = relative_time(activity.created_at);
            let secondary_color = if selected { Color::Gray } else { Color::DarkGray };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {}{} {title}", activity.status, progress_str),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("  · {ago}"),
                    Style::default().fg(secondary_color),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.updates_scroll));
    frame.render_stateful_widget(list, area, &mut state);
}
