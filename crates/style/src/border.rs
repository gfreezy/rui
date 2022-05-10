use druid_shell::piet::{Color};
use nom::{
    combinator::{map},
    number::complete::double,
    sequence::tuple,
    IResult,
};

use super::{draw::parse_color, size::Width, utils::ws};

enum_type!(BorderStyle => [
        Solid, Dash
], Solid, parse_border_style);

#[derive(Debug, PartialEq, Clone)]
pub struct Border {
    width: Width,
    color: Color,
    style: BorderStyle,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: Width(0.),
            color: Color::rgba8(0, 0, 0, 0),
            style: BorderStyle::Solid,
        }
    }
}

pub(crate) fn parse_border(input: &str) -> IResult<&str, Border> {
    map(
        tuple((ws(double), ws(parse_border_style), ws(parse_color))),
        |(width, style, color)| Border {
            width: width.into(),
            color,
            style,
        },
    )(input)
}
