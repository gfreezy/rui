use druid_shell::piet::{FontFamily, FontStyle, FontWeight};

use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::complete::{alphanumeric1, char, digit1},
    combinator::map_res,
    number::complete::double,
    sequence::delimited,
    IResult,
};

use crate::style::utils::parse_num;

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

impl Default for LineBreaking {
    fn default() -> Self {
        LineBreaking::WordWrap
    }
}

pub(crate) fn parse_font_family<'a>(input: &'a str) -> IResult<&'a str, FontFamily> {
    enum_parser!(parse_common_font, FontFamily => [
       SERIF, SANS_SERIF, MONOSPACE, SYSTEM_UI
    ]);
    let mut font_family = alt((
        parse_common_font,
        map_res(alphanumeric1, |s| {
            Ok::<_, nom::error::Error<&str>>(FontFamily::new_unchecked(s))
        }),
        map_res(
            delimited(char('"'), take_till1(|c| "\"\n\r".contains(c)), char('"')),
            |s| Ok::<_, nom::error::Error<&str>>(FontFamily::new_unchecked(s)),
        ),
    ));
    font_family(input)
}

pub(crate) fn parse_font_weight<'a>(input: &'a str) -> IResult<&'a str, FontWeight> {
    enum_parser!(parse_common_weight, FontWeight => [
       THIN, HAIRLINE, EXTRA_LIGHT, LIGHT, REGULAR, NORMAL, MEDIUM, SEMI_BOLD, BOLD, EXTRA_BOLD, BLACK, HEAVY, EXTRA_BLACK
    ]);
    let mut font_weight = alt((
        parse_common_weight,
        map_res(digit1, |s: &str| {
            let num: u16 = parse_num(s)?;
            Ok::<_, nom::error::Error<&str>>(FontWeight::new(num))
        }),
    ));

    font_weight(input)
}

enum_parser!(parse_font_style, FontStyle => [Regular, Italic]);
enum_parser!(parse_line_breaking, LineBreaking => [WordWrap, Clip, Overflow]);

simple_type!([FontSize, f64, 0.]);

simple_attr_parser!(parse_font_size, FontSize, double);

#[cfg(test)]
mod tests {
    use druid_shell::piet::Color;

    use crate::style::draw::parse_color;

    use super::*;

    #[test]
    fn test_parse_font_family() {
        let fixtures = [
            ("serif", FontFamily::SERIF),
            ("Serif", FontFamily::SERIF),
            ("Sans_Serif", FontFamily::SANS_SERIF),
            ("sans_Serif", FontFamily::SANS_SERIF),
            ("Monospace", FontFamily::MONOSPACE),
            ("monospace", FontFamily::MONOSPACE),
            ("system_ui", FontFamily::SYSTEM_UI),
            ("System_ui", FontFamily::SYSTEM_UI),
            ("apple", FontFamily::new_unchecked("apple")),
            ("SanFrancisco", FontFamily::new_unchecked("SanFrancisco")),
            (
                "\"San Francisco\"",
                FontFamily::new_unchecked("San Francisco"),
            ),
        ];
        for (input, expected) in fixtures {
            assert!(
                matches!(parse_font_family(input), Ok((_, v)) if v == expected),
                "input: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_font_weight() {
        let fixtures = [
            ("THIN", FontWeight::THIN),
            ("HAIRLINE", FontWeight::HAIRLINE),
            ("EXTRA_LIGHT", FontWeight::EXTRA_LIGHT),
            ("LIGHT", FontWeight::LIGHT),
            ("REGULAR", FontWeight::REGULAR),
            ("NORMAL", FontWeight::NORMAL),
            ("MEDIUM", FontWeight::MEDIUM),
            ("SEMI_BOLD", FontWeight::SEMI_BOLD),
            ("BOLD", FontWeight::BOLD),
            ("EXTRA_BOLD", FontWeight::EXTRA_BOLD),
            ("BLACK", FontWeight::BLACK),
            ("HEAVY", FontWeight::HEAVY),
            ("EXTRA_BLACK", FontWeight::EXTRA_BLACK),
            ("130", FontWeight::new(130)),
            ("431", FontWeight::new(431)),
        ];
        for (input, expected) in fixtures {
            assert!(
                matches!(parse_font_weight(input), Ok((_, v)) if v == expected),
                "input: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_color() {
        let fixtures = [
            ("#fff", Color::from_hex_str("fff").unwrap()),
            ("#af1", Color::from_hex_str("af1").unwrap()),
            ("#223123", Color::from_hex_str("223123").unwrap()),
            ("rgb(10, 20, 15)", Color::rgb8(10, 20, 15)),
            ("rgb( 10 , 20 , 15 )", Color::rgb8(10, 20, 15)),
            ("rgba( 10 , 20 , 15, 0.4 )", Color::rgba8(10, 20, 15, 102)),
        ];
        for (input, expected) in fixtures {
            let ret = parse_color(input);
            assert!(
                matches!(&ret, Ok((_, v)) if v == &expected),
                "input: {}, result: {:?}",
                input,
                ret
            );
        }
    }
}
