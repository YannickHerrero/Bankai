use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, LoginState};

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}

pub fn render(app: &App, frame: &mut Frame) {
    match &app.login_state {
        LoginState::Prompt => render_prompt(app, frame),
        LoginState::WaitingForToken { auth_url } => {
            render_token_input(app, frame, auth_url.clone())
        }
    }
}

fn render_prompt(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 8, frame.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" bankai ");

    let mut lines = vec![
        Line::from(""),
        Line::from("AniList Terminal Client"),
        Line::from(""),
        Line::from("Press Enter to start login"),
        Line::from("Press q to quit"),
    ];

    if let Some(ref msg) = app.status_message {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow),
        ));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_token_input(app: &App, frame: &mut Frame, auth_url: String) {
    let area = centered_rect(70, 14, frame.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" bankai - Login ");

    let mut lines = vec![
        Line::from(""),
        Line::from("1. Open this URL in your browser:"),
        Line::from(""),
        Line::styled(auth_url, Style::default().fg(Color::Green)),
        Line::from(""),
        Line::from("2. Authorize, then copy the token from the URL"),
        Line::from(""),
        Line::from("3. Paste your access token below and press Enter:"),
        Line::from(""),
    ];

    let input_display = if app.token_input.is_empty() {
        Line::styled(
            "_ ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::SLOW_BLINK),
        )
    } else {
        let display: String = "*".repeat(app.token_input.len().min(50));
        Line::styled(display, Style::default().fg(Color::White))
    };
    lines.push(input_display);

    if let Some(ref msg) = app.status_message {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow),
        ));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}
