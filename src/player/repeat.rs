#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_covers_all_modes() {
        let mut mode = RepeatMode::Off;
        mode = mode.next();
        assert_eq!(mode, RepeatMode::One);
        mode = mode.next();
        assert_eq!(mode, RepeatMode::All);
        mode = mode.next();
        assert_eq!(mode, RepeatMode::Off);
    }
}
