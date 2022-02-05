use std::{cell::RefCell, clone, rc::Rc, str::FromStr};

use anymap2::AnyMap;
use druid_shell::{
    kurbo::Line,
    piet::{Color, FontFamily, FontStyle, FontWeight, TextAlignment},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till1},
    character::complete::{alphanumeric1, char, digit1, hex_digit1, multispace0, space0},
    combinator::{map, map_res},
    error::ParseError,
    multi::{self, count, many1, separated_list0, separated_list1},
    number::complete::float,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use super::{LineBreaking, TextStyle};

pub(crate) fn parse_font_family<'a>(input: &'a str) -> IResult<&'a str, FontFamily> {
    let mut font_family = alt((
        map_res(tag_no_case("Serif"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontFamily::SERIF)
        }),
        map_res(tag_no_case("SansSerif"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontFamily::SANS_SERIF)
        }),
        map_res(tag_no_case("Monospace"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontFamily::MONOSPACE)
        }),
        map_res(tag_no_case("system"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontFamily::SYSTEM_UI)
        }),
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
    let mut font_weight = alt((
        map_res(tag_no_case("THIN"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::THIN)
        }),
        map_res(tag_no_case("HAIRLINE"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::HAIRLINE)
        }),
        map_res(tag_no_case("EXTRA_LIGHT"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::EXTRA_LIGHT)
        }),
        map_res(tag_no_case("LIGHT"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::LIGHT)
        }),
        map_res(tag_no_case("REGULAR"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::REGULAR)
        }),
        map_res(tag_no_case("NORMAL"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::NORMAL)
        }),
        map_res(tag_no_case("MEDIUM"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::MEDIUM)
        }),
        map_res(tag_no_case("SEMI_BOLD"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::SEMI_BOLD)
        }),
        map_res(tag_no_case("BOLD"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::BOLD)
        }),
        map_res(tag_no_case("EXTRA_BOLD"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::EXTRA_BOLD)
        }),
        map_res(tag_no_case("BLACK"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::BLACK)
        }),
        map_res(tag_no_case("HEAVY"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::HEAVY)
        }),
        map_res(tag_no_case("EXTRA_BLACK"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontWeight::EXTRA_BLACK)
        }),
        map_res(digit1, |s: &str| {
            let num: u16 = s.parse().map_err(|_| {
                nom::error::Error::new("invalid digit", nom::error::ErrorKind::Verify)
            })?;
            Ok::<_, nom::error::Error<&str>>(FontWeight::new(num))
        }),
    ));

    font_weight(input)
}

pub(crate) fn parse_font_style<'a>(input: &'a str) -> IResult<&'a str, FontStyle> {
    alt((
        map_res(tag_no_case("regular"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontStyle::Regular)
        }),
        map_res(tag_no_case("italic"), |_| {
            Ok::<_, nom::error::Error<&str>>(FontStyle::Italic)
        }),
    ))(input)
}

pub(crate) fn parse_line_breaking<'a>(input: &'a str) -> IResult<&'a str, LineBreaking> {
    alt((
        map_res(tag_no_case("WordWrap"), |_| {
            Ok::<_, nom::error::Error<&str>>(LineBreaking::WordWrap)
        }),
        map_res(tag_no_case("Clip"), |_| {
            Ok::<_, nom::error::Error<&str>>(LineBreaking::Clip)
        }),
        map_res(tag_no_case("Overflow"), |_| {
            Ok::<_, nom::error::Error<&str>>(LineBreaking::Overflow)
        }),
    ))(input)
}

pub(crate) fn parse_text_alignment<'a>(input: &'a str) -> IResult<&'a str, TextAlignment> {
    alt((
        map_res(tag_no_case("Start"), |_| {
            Ok::<_, nom::error::Error<&str>>(TextAlignment::Start)
        }),
        map_res(tag_no_case("End"), |_| {
            Ok::<_, nom::error::Error<&str>>(TextAlignment::End)
        }),
        map_res(tag_no_case("Center"), |_| {
            Ok::<_, nom::error::Error<&str>>(TextAlignment::Center)
        }),
        map_res(tag_no_case("Justified"), |_| {
            Ok::<_, nom::error::Error<&str>>(TextAlignment::Justified)
        }),
    ))(input)
}

fn parse_num<T: FromStr>(s: &str) -> Result<T, nom::error::Error<&str>> {
    s.parse()
        .map_err(|_| nom::error::Error::new("invalid num", nom::error::ErrorKind::Verify))
}

pub(crate) fn parse_color<'a>(input: &'a str) -> IResult<&'a str, Color> {
    let parse_hex_color = map_res(preceded(tag("#"), hex_digit1), |v| {
        Color::from_hex_str(v)
            .map_err(|_| nom::error::Error::new("invalid hex color", nom::error::ErrorKind::Verify))
    });
    let parse_rgb_color = map_res(
        pair(
            alt((tag("rgba"), tag("rgb"))),
            delimited(tag("("), separated_list0(ws(tag(",")), ws(float)), tag(")")),
        ),
        |(mode, vals)| match (mode, vals.as_slice()) {
            ("rgb", &[r, g, b]) => Ok(Color::rgb8(r as u8, g as u8, b as u8)),
            ("rgba", &[r, g, b, a]) => Ok(Color::rgba8(
                r as u8,
                g as u8,
                b as u8,
                (a.max(0.0).min(1.0) * 255.0).round() as u8,
            )),
            _ => Err(nom::error::Error::new(
                "invalid rgb or rgba color",
                nom::error::ErrorKind::Verify,
            )),
        },
    );
    alt((parse_hex_color, parse_rgb_color))(input)
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Size(f32);

impl From<Size> for f64 {
    fn from(s: Size) -> Self {
        s.0 as f64
    }
}

pub(crate) fn parse_rule(input: &str) -> IResult<&str, TextStyle> {
    let attrs = Rc::new(RefCell::new(AnyMap::new()));
    let family = tuple((
        ws(tag("family")),
        ws(tag(":")),
        ws(map(
            parse_font_family,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));
    let size = tuple((
        ws(tag("size")),
        ws(tag(":")),
        ws(map(
            float,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(Size(v));
            }),
        )),
    ));
    let weight = tuple((
        ws(tag("weight")),
        ws(tag(":")),
        ws(map(
            parse_font_weight,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));
    let style = tuple((
        ws(tag("style")),
        ws(tag(":")),
        ws(map(
            parse_font_style,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));
    let color = tuple((
        ws(tag("color")),
        ws(tag(":")),
        ws(map(
            parse_color,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));
    let line_breaking = tuple((
        ws(tag("line_breaking")),
        ws(tag(":")),
        ws(map(
            parse_line_breaking,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));
    let alignment = tuple((
        ws(tag("alignment")),
        ws(tag(":")),
        ws(map(
            parse_text_alignment,
            clone!([attrs] move |v| {
                attrs.borrow_mut().insert(v);
            }),
        )),
    ));

    let parse_attrs = many1(ws(terminated(
        ws(alt((
            family,
            size,
            weight,
            style,
            color,
            line_breaking,
            alignment,
        ))),
        ws(tag(";")),
    )));
    let parse_attrs_block = delimited(ws(tag("{")), ws(parse_attrs), ws(tag("}")));
    let parse_class_name = map(pair(tag("."), alphanumeric1), |(_, name)| {
        format!(".{name}")
    });
    let (left, class_name) = terminated(ws(parse_class_name), ws(parse_attrs_block))(input)?;
    let mut style_attrs = attrs.take();
    let mut style = TextStyle::default();
    style.name = class_name;

    macro_rules! assign {
        ($($i:tt => $ty:ty),*) => {
            $(
                if let Some(v) = style_attrs.remove::<$ty>() {
                    style.$i = v.into();
                }
            )+
        };
    }

    assign!(
        family => FontFamily,
        size => Size,
        weight => FontWeight,
        style => FontStyle,
        color => Color,
        line_breaking => LineBreaking,
        alignment => TextAlignment
    );

    Ok((left, style))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_family() {
        let fixtures = [
            ("serif", FontFamily::SERIF),
            ("Serif", FontFamily::SERIF),
            ("SansSerif", FontFamily::SANS_SERIF),
            ("sansSerif", FontFamily::SANS_SERIF),
            ("Monospace", FontFamily::MONOSPACE),
            ("monospace", FontFamily::MONOSPACE),
            ("system", FontFamily::SYSTEM_UI),
            ("System", FontFamily::SYSTEM_UI),
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

    #[test]
    fn test_parse_rule() {
        let ret = parse_rule(
            r#"
        .classname {
            family: Sans;
            size: 14;
            weight: bold;
            style: regular;
            color: #fff;
            line_breaking: clip;
            alignment: center;
        }
        "#,
        );
        dbg!(&ret);
        assert!(ret.is_ok(), "{:?}", ret);
    }
}
