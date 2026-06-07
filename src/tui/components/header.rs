use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::Line,
};

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Now Playing ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let placeholder = Paragraph::new(Line::from("[ no track ]"));
    f.render_widget(placeholder, inner);
}
