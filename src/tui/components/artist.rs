// TODO: Phase 5 — Artist view with discography.
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}, text::Line};

#[allow(dead_code)]
pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default().title(" Artist ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(Line::from("[ Artist — Phase 5 ]")), inner);
}
