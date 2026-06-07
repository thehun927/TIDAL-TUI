// TODO: Phase 5 — Playlist detail view.
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}, text::Line};

#[allow(dead_code)]
pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default().title(" Playlist ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(Line::from("[ Playlist — Phase 5 ]")), inner);
}
