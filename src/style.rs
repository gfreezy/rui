mod text_style_parser;

use std::{num::ParseIntError, str::FromStr, string::ParseError};

use druid_shell::piet::{self, Color, FontFamily, FontStyle, FontWeight, TextAlignment};
use serde::Deserialize;

use self::text_style_parser::parse_rule;

mod de {
    use druid_shell::{
        kurbo::Insets,
        piet::{self, TextAlignment},
    };
    use serde::Deserializer;

    pub(crate) fn de_color<'de, D>(deserializer: D) -> Result<piet::Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        piet::Color::from_hex_str(s).map_err(|_| serde::de::Error::custom("invalid color"))
    }

    pub(crate) fn de_insets<'de, D>(deserializer: D) -> Result<Insets, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let values: Vec<_> = s.split_ascii_whitespace().collect();
        let insets = match values.as_slice() {
            &[v] => {
                let n: f64 = v
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                Ok(Insets::uniform(n))
            }
            &[x, y] => {
                let x: f64 = x
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                let y: f64 = y
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                Ok(Insets::uniform_xy(x, y))
            }
            &[top, right, bottom, left] => {
                let x0: f64 = left
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                let y0: f64 = top
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                let x1: f64 = right
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                let y1: f64 = bottom
                    .parse()
                    .map_err(|_| serde::de::Error::custom("invalid insets"))?;
                Ok(Insets::new(x0, y0, x1, y1))
            }
            _ => {
                return Err(serde::de::Error::custom("invalid insets"));
            }
        };
        insets
    }

    pub(crate) fn de_text_alignment<'de, D>(
        deserializer: D,
    ) -> Result<piet::TextAlignment, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        match s {
            "Start" => Ok(TextAlignment::Start),
            "End" => Ok(TextAlignment::End),
            "Center" => Ok(TextAlignment::Center),
            "Justified" => Ok(TextAlignment::Justified),
            _ => Err(serde::de::Error::custom("invalid color")),
        }
    }
}

/// Options for handling lines that are too wide for the label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineBreaking {
    /// Lines are broken at word boundaries.
    WordWrap,
    /// Lines are truncated to the width of the label.
    Clip,
    /// Lines overflow the label.
    Overflow,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TextStyle {
    pub name: String,
    /// The font's [`FontFamily`](struct.FontFamily.html).
    pub family: FontFamily,
    /// The font's size.
    pub size: f64,
    /// The font's [`FontWeight`](struct.FontWeight.html).
    pub weight: FontWeight,
    /// The font's [`FontStyle`](struct.FontStyle.html).
    pub style: FontStyle,
    pub color: Color,
    pub line_breaking: LineBreaking,
    pub alignment: TextAlignment,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            family: Default::default(),
            size: Default::default(),
            weight: Default::default(),
            style: Default::default(),
            color: Color::from_hex_str("#000").unwrap(),
            line_breaking: LineBreaking::WordWrap,
            alignment: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseTextStyleError;

impl std::fmt::Display for ParseTextStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseTextStyleError")
    }
}

impl std::error::Error for ParseTextStyleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl FromStr for TextStyle {
    type Err = ParseTextStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_rule(s)
            .map(|(_, v)| v)
            .map_err(|_| ParseTextStyleError)
    }
}
