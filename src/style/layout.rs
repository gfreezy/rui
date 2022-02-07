use nom::IResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Layout {
    Flex,
}

enum_parser!(parse_layout, Layout => [Flex]);

simple_type!([Flex, f64, f64::NAN]);
