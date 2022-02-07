use druid_shell::piet::Color;
use nom::{
    branch::alt,
    bytes::complete::{tag},
    character::complete::{hex_digit1},
    combinator::{map, map_res},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, pair, preceded},
    IResult,
};

use super::utils::{make_error, ws};

simple_type!([Alpha, f64, 0.]);

pub(crate) fn parse_alpha(input: &str) -> IResult<&str, Alpha> {
    map(double, |v| Alpha(v))(input)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Background {
    pub(crate) color: Color,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Color::TRANSPARENT,
        }
    }
}

pub(crate) fn parse_background(input: &str) -> IResult<&str, Background> {
    map(parse_color, |color| Background { color })(input)
}

pub(crate) fn parse_color<'a>(input: &'a str) -> IResult<&'a str, Color> {
    let parse_hex_color = map_res(preceded(tag("#"), hex_digit1), |v| {
        Color::from_hex_str(v).map_err(|_| make_error("invalid hex color"))
    });
    let parse_rgb_color = map_res(
        pair(
            alt((tag("rgba"), tag("rgb"))),
            delimited(
                tag("("),
                separated_list0(ws(tag(",")), ws(double)),
                tag(")"),
            ),
        ),
        |(mode, vals)| match (mode, vals.as_slice()) {
            ("rgb", &[r, g, b]) => Ok(Color::rgb8(r as u8, g as u8, b as u8)),
            ("rgba", &[r, g, b, a]) => Ok(Color::rgba8(
                r as u8,
                g as u8,
                b as u8,
                (a.max(0.0).min(1.0) * 255.0).round() as u8,
            )),
            _ => Err(make_error("invalid rgb or rgba color")),
        },
    );
    alt((parse_hex_color, parse_rgb_color))(input)
}
