#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Search,
    Library,
    Queue,
    Settings,
}

impl Screen {
    /// Cycle to the next screen (Tab key).
    pub fn next(self) -> Self {
        match self {
            Screen::Home     => Screen::Search,
            Screen::Search   => Screen::Library,
            Screen::Library  => Screen::Queue,
            Screen::Queue    => Screen::Settings,
            Screen::Settings => Screen::Home,
        }
    }

    /// Map number keys 1–5 to screens.
    #[allow(dead_code)]
    pub fn from_key(n: u8) -> Option<Self> {
        match n {
            1 => Some(Screen::Home),
            2 => Some(Screen::Search),
            3 => Some(Screen::Library),
            4 => Some(Screen::Queue),
            5 => Some(Screen::Settings),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn title(self) -> &'static str {
        match self {
            Screen::Home     => "Home",
            Screen::Search   => "Search",
            Screen::Library  => "Library",
            Screen::Queue    => "Queue",
            Screen::Settings => "Settings",
        }
    }
}
