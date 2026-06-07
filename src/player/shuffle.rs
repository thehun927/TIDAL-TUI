#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuffleMode {
    Off,
    Random,
    Favourites,
    Discovery,
}

impl ShuffleMode {
    pub fn next(self) -> Self {
        match self {
            ShuffleMode::Off => ShuffleMode::Random,
            ShuffleMode::Random => ShuffleMode::Favourites,
            ShuffleMode::Favourites => ShuffleMode::Discovery,
            ShuffleMode::Discovery => ShuffleMode::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_covers_all_modes() {
        let mut mode = ShuffleMode::Off;
        mode = mode.next();
        assert_eq!(mode, ShuffleMode::Random);
        mode = mode.next();
        assert_eq!(mode, ShuffleMode::Favourites);
        mode = mode.next();
        assert_eq!(mode, ShuffleMode::Discovery);
        mode = mode.next();
        assert_eq!(mode, ShuffleMode::Off);
    }
}
