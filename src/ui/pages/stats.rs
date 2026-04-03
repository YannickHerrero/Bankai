use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::centered_rect;

pub fn render(_app: &App, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Stats ");

    let text = Text::from(vec![
        Line::from(""),
        Line::from("Stats"),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    let popup = centered_rect(40, 5, area);
    frame.render_widget(paragraph, popup);
}
