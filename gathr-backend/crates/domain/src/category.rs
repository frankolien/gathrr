use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Birthday,
    Party,
    Meetup,
    Dinner,
    GameNight,
    Wedding,
    Other,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::Birthday,
        Self::Party,
        Self::Meetup,
        Self::Dinner,
        Self::GameNight,
        Self::Wedding,
        Self::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Birthday => "birthday",
            Self::Party => "party",
            Self::Meetup => "meetup",
            Self::Dinner => "dinner",
            Self::GameNight => "game_night",
            Self::Wedding => "wedding",
            Self::Other => "other",
        }
    }

    pub fn parse_or_other(input: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == input.trim().to_ascii_lowercase())
            .unwrap_or(Self::Other)
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_round_trips() {
        for category in Category::ALL {
            assert_eq!(Category::parse_or_other(category.as_str()), category);
        }
    }

    #[test]
    fn unknown_categories_render_rather_than_fail() {
        assert_eq!(Category::parse_or_other("hackathon"), Category::Other);
        assert_eq!(Category::parse_or_other(""), Category::Other);
    }

    #[test]
    fn parsing_tolerates_case_and_padding_from_older_clients() {
        assert_eq!(Category::parse_or_other("  BIRTHDAY "), Category::Birthday);
        assert_eq!(Category::parse_or_other("Game_Night"), Category::GameNight);
    }
}
