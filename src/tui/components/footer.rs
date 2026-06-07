use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::Line,
};

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Controls ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let placeholder = Paragraph::new(Line::from(
        "Space: play/pause  n: next  p: prev  s: shuffle  r: repeat  q: quit",
    ));
    f.render_widget(placeholder, inner);
}
