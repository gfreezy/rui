use std::str::FromStr;


use nom::{
    branch::alt,
    bytes::complete::{tag_no_case},
    character::complete::{multispace0},
    combinator::{map},
    error::ParseError,
    sequence::{delimited},
    IResult,
};

pub(crate) fn make_error(s: &'static str) -> nom::error::Error<&'static str> {
    nom::error::Error::new(s, nom::error::ErrorKind::Verify)
}

pub(crate) fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub(crate) fn parse_num<T: FromStr>(s: &str) -> Result<T, nom::error::Error<&str>> {
    s.parse().map_err(|_| make_error("invalid num"))
}

pub(crate) fn parse_bool(input: &str) -> IResult<&str, bool> {
    map(
        alt((tag_no_case("true"), tag_no_case("false"))),
        |v: &str| match v.to_ascii_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        },
    )(input)
}
