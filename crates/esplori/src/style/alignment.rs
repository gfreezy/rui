use super::utils::{make_error, parse_kebab_case, ws};
use druid_shell::piet::TextAlignment;
use nom::branch::alt;
use nom::bytes::streaming::tag;
use nom::combinator::map;
use nom::{combinator::map_res, multi::many1, number::complete::double, IResult};

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

#[derive(Debug, PartialEq, Clone)]
pub struct Alignment {
    x: f64,
    y: f64,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::center()
    }
}

impl Alignment {
    pub const fn top_left() -> Alignment {
        Alignment { x: -1., y: -1. }
    }
    pub const fn top_center() -> Alignment {
        Alignment { x: 0., y: -1. }
    }
    pub const fn top_right() -> Alignment {
        Alignment { x: 1., y: -1. }
    }
    pub const fn center_left() -> Alignment {
        Alignment { x: -1., y: 0. }
    }
    pub const fn center() -> Alignment {
        Alignment { x: 0., y: 0. }
    }
    pub const fn center_right() -> Alignment {
        Alignment { x: 1., y: 0. }
    }
    pub const fn bottom_left() -> Alignment {
        Alignment { x: -1., y: 1. }
    }
    pub const fn bottom_center() -> Alignment {
        Alignment { x: 0., y: 1. }
    }
    pub const fn bottom_right() -> Alignment {
        Alignment { x: 1., y: 1. }
    }
}

pub(crate) fn parse_alignment(input: &str) -> IResult<&str, Alignment> {
    alt!(
        map_res(many1(ws(double)), |v| match v.as_slice() {
            &[x, y] => Ok(Alignment { x, y }),
            _ => Err(make_error("invalid alignment")),
        }),
        ws(alt!(
            map(parse_kebab_case("TopLeft"), |_| Alignment::top_left()),
            map(parse_kebab_case("TopCenter"), |_| Alignment::top_center()),
            map(parse_kebab_case("TopRight"), |_| Alignment::top_right()),
            map(parse_kebab_case("CenterLeft"), |_| Alignment::center_left()),
            map(parse_kebab_case("Center"), |_| Alignment::center()),
            map(
                parse_kebab_case("CenterRight"),
                |_| Alignment::center_right()
            ),
            map(parse_kebab_case("BottomLeft"), |_| Alignment::bottom_left()),
            map(parse_kebab_case("BottomCenter"), |_| {
                Alignment::bottom_center()
            }),
            map(
                parse_kebab_case("BottomRight"),
                |_| Alignment::bottom_right()
            ),
        )),
    )(input)
}
