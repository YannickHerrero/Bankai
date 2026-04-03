use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::centered_rect;
use crate::app::PageSelectorState;

pub fn render(state: &PageSelectorState, frame: &mut Frame) {
    let height = (state.filtered.len() as u16 + 4).min(15);
    let area = centered_rect(40, height, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Go to page ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Input line
    let input_line = if state.query.is_empty() {
        Line::from(vec![
            Span::styled(
                "Type to filter...",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(&state.query, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::DarkGray).add_modifier(Modifier::SLOW_BLINK)),
        ])
    };

    let input_area = ratatui::layout::Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(input_line), input_area);

    // Separator
    let sep_area = ratatui::layout::Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from("─".repeat(inner.width as usize))
            .style(Style::default().fg(Color::DarkGray))),
        sep_area,
    );

    // Page list
    let list_area = ratatui::layout::Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };

    let items: Vec<ListItem> = state
        .filtered
        .iter()
        .map(|page| ListItem::new(Line::from(format!("  {}", page.label()))))
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !state.filtered.is_empty() {
        list_state.select(Some(state.selected));
    }

    frame.render_stateful_widget(list, list_area, &mut list_state);
}
