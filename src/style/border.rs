use druid_shell::piet::{Color, LineJoin, StrokeStyle};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::{map, map_res},
    number::complete::double,
    sequence::tuple,
    IResult,
};

use super::{
    draw::parse_color,
    size::Width,
    utils::{make_error, ws},
};

const DASH: StrokeStyle = StrokeStyle::new()
    .dash_pattern(&[4.0, 2.0])
    .dash_offset(8.0)
    .line_join(LineJoin::Round);
const SOLID: StrokeStyle = StrokeStyle::new()
    .dash_pattern(&[])
    .line_join(LineJoin::Round);

#[derive(Debug, PartialEq, Clone)]
pub struct Border {
    width: Width,
    color: Color,
    style: StrokeStyle,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: Width(0.),
            color: Color::rgba8(0, 0, 0, 0),
            style: SOLID,
        }
    }
}

pub(crate) fn parse_border(input: &str) -> IResult<&str, Border> {
    let parse_border = map_res(
        alt((tag_no_case("solid"), tag_no_case("dash"))),
        |v: &str| match v.to_ascii_lowercase().as_str() {
            "solid" => Ok(SOLID),
            "dash" => Ok(DASH),
            _ => Err(make_error("invalid border style")),
        },
    );
    map(
        tuple((ws(double), ws(parse_border), ws(parse_color))),
        |(width, style, color)| Border {
            width: width.into(),
            color,
            style,
        },
    )(input)
}
