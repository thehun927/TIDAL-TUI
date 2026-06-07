use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use crate::tui::{screens::Screen, components};

pub struct TuiApp;

impl TuiApp {
    pub fn new() -> Self {
        TuiApp
    }

    pub fn render(&self, f: &mut Frame, screen: Screen) {
        // Outer vertical split: header | body | footer
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header / now-playing bar
                Constraint::Min(0),     // body
                Constraint::Length(3),  // footer / controls bar
            ])
            .split(f.area());

        components::header::render(f, outer[0]);
        components::footer::render(f, outer[2]);

        // Body horizontal split: sidebar | content
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // sidebar
                Constraint::Min(0),     // main content
            ])
            .split(outer[1]);

        components::sidebar::render(f, body[0]);

        // Render the active screen in the content area.
        match screen {
            Screen::Home     => components::content::render(f, body[1]),
            Screen::Search   => components::search::render(f, body[1]),
            Screen::Library  => components::library::render(f, body[1]),
            Screen::Queue    => components::queue::render(f, body[1]),
            Screen::Settings => components::settings::render(f, body[1]),
        }
    }
}
