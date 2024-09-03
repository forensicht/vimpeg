use anyhow::Result;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ColorScheme {
    Dark,
    Light,
    Default,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::Default
    }
}

impl fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
            Self::Default => write!(f, "Default"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Language {
    English,
    Portuguese,
    Spanish,
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::English => write!(f, "en"),
            Self::Portuguese => write!(f, "pt"),
            Self::Spanish => write!(f, "es"),
        }
    }
}

impl FromStr for Language {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "en" => Ok(Self::English),
            "pt" => Ok(Self::Portuguese),
            "es" => Ok(Self::Spanish),
            _ => Err("Language does not exist"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Preference {
    pub color_scheme: ColorScheme,
    pub language: Language,
}

impl Default for Preference {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Default,
            language: Language::English,
        }
    }
}

impl Preference {
    pub fn new(color_scheme: ColorScheme, language: Language) -> Self {
        Self {
            color_scheme,
            language,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language() {
        let language = Language::from_str("pt").unwrap();
        assert_eq!(language, Language::Portuguese);

        let language_str = Language::English.to_string();
        assert_eq!(language_str, "en");
    }
}
