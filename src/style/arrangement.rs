use nom::{IResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalArrangement {
    Bottom,
    Center,
    Top,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl Default for VerticalArrangement {
    fn default() -> Self {
        VerticalArrangement::Top
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HorizontalArrangement {
    Leading,
    Trailing,
    Center,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl Default for HorizontalArrangement {
    fn default() -> Self {
        HorizontalArrangement::Leading
    }
}
enum_parser!(parse_horizontal_arrangement, HorizontalArrangement => [
    Leading, Trailing, Center, SpaceBetween, SpaceEvenly, SpaceAround
]);

enum_parser!(parse_vertical_arrangement, VerticalArrangement => [
    Bottom, Center, Top, SpaceBetween, SpaceEvenly, SpaceAround
]);
