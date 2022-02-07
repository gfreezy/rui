use druid_shell::piet::TextAlignment;
use nom::{IResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlignment {
    Bottom,
    Center,
    // FirstTextBaseline,
    // LastTextBaseline,
    Top,
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        VerticalAlignment::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HorizontalAlignment {
    Start,
    End,
    Center,
}

impl Default for HorizontalAlignment {
    fn default() -> Self {
        HorizontalAlignment::Start
    }
}

enum_parser!(parse_text_alignment, TextAlignment => [
    Start, End, Center, Justified
]);

enum_parser!(parse_horizontal_alignment, HorizontalAlignment => [
    Start, End, Center
]);

enum_parser!(parse_vertical_alignment, VerticalAlignment => [
    Bottom, Center, Top
]);
