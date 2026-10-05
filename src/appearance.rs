use std::path::PathBuf;

use tao::window::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    System,
    Light,
    Dark,
}

impl Appearance {
    pub const ALL: [Appearance; 3] = [Appearance::System, Appearance::Light, Appearance::Dark];

    pub fn theme(self) -> Option<Theme> {
        match self {
            Appearance::System => None,
            Appearance::Light => Some(Theme::Light),
            Appearance::Dark => Some(Theme::Dark),
        }
    }

    pub fn opposite_of(current: Theme) -> Self {
        match current {
            Theme::Dark => Appearance::Light,
            _ => Appearance::Dark,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Appearance::System => "system",
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        }
    }

    pub fn parse(text: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|appearance| appearance.name() == text.trim())
            .unwrap_or(Appearance::System)
    }

    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Appearance::System;
        };
        std::fs::read_to_string(path).map(|text| Self::parse(&text)).unwrap_or(Appearance::System)
    }

    pub fn save(self) {
        let Some(path) = settings_path() else {
            return;
        };
        let Some(directory) = path.parent() else {
            return;
        };
        std::fs::create_dir_all(directory).ok();
        std::fs::write(path, self.name()).ok();
    }
}

fn settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Application Support/mdview/appearance"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_the_current_theme() {
        assert_eq!(Appearance::opposite_of(Theme::Dark), Appearance::Light);
        assert_eq!(Appearance::opposite_of(Theme::Light), Appearance::Dark);
    }

    #[test]
    fn names_round_trip_and_unknown_falls_back_to_system() {
        Appearance::ALL.into_iter().for_each(|appearance| {
            assert_eq!(Appearance::parse(appearance.name()), appearance);
        });
        assert_eq!(Appearance::parse("dark\n"), Appearance::Dark);
        assert_eq!(Appearance::parse("sepia"), Appearance::System);
    }
}
