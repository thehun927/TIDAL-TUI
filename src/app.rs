use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::{
    config::Config,
    errors::Result,
    tui::{app::TuiApp, screens::Screen},
};

pub struct App {
    #[allow(dead_code)]
    config:  Config,
    tui:     TuiApp,
    screen:  Screen,
    running: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        App {
            config,
            tui:     TuiApp::new(),
            screen:  Screen::Home,
            running: true,
        }
    }

    /// Main loop — called from main.rs after terminal setup.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|f| self.tui.render(f, self.screen))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        // Poll with a 16 ms timeout (~60 fps), non-blocking.
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Ignore key-release events on Windows/some terminals.
                if key.kind != KeyEventKind::Press {
                    return Ok(());
                }
                self.handle_key(key.code);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) {
        match code {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,

            // Screen switching via number row
            KeyCode::Char('1') => self.screen = Screen::Home,
            KeyCode::Char('2') => self.screen = Screen::Search,
            KeyCode::Char('3') => self.screen = Screen::Library,
            KeyCode::Char('4') => self.screen = Screen::Queue,
            KeyCode::Char('5') => self.screen = Screen::Settings,

            // Cycle screens with Tab
            KeyCode::Tab => self.screen = self.screen.next(),

            _ => {}
        }
    }
}
