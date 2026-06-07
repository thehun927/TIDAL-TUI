use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::Line,
};

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Navigation ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let placeholder = Paragraph::new(vec![
        Line::from("1 Home"),
        Line::from("2 Search"),
        Line::from("3 Library"),
        Line::from("4 Queue"),
        Line::from("5 Settings"),
    ]);
    f.render_widget(placeholder, inner);
}
