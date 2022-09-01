use nom::{number::complete::double, IResult};

enum_type!(AxisDirection => [Up, Right, Down, Left], Down, parse_axis_direction);

impl AxisDirection {
    pub fn flip(&self) -> AxisDirection {
        match self {
            AxisDirection::Down => AxisDirection::Up,
            AxisDirection::Left => AxisDirection::Right,
            AxisDirection::Right => AxisDirection::Left,
            AxisDirection::Up => AxisDirection::Down,
        }
    }

    pub fn is_reversed(&self) -> bool {
        match self {
            AxisDirection::Up | AxisDirection::Left => true,
            AxisDirection::Right | AxisDirection::Down => false,
        }
    }
}

enum_type!(Layout => [Flex, Default], Default, parse_layout);

simple_type!([Flex, f64, 1.0]);
simple_attr_parser!(parse_flex, Flex, double);

enum_type!(MainAxisSize => [Min, Max], Min, parse_main_axis_size);

enum_type!(MainAxisAlignment => [
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly
], Start, parse_main_axis_alignment);

enum_type!(CrossAxisAlignment => [
    Start,
    End,
    Center,
    Stretch,
    Baseline
], Start, parse_cross_axis_alignment);

enum_type!(TextDirection => [
    Ltr,
    Rtl
], Ltr, parse_text_direction);

impl TextDirection {
    pub fn to_axis_direction(&self) -> AxisDirection {
        match self {
            TextDirection::Ltr => AxisDirection::Right,
            TextDirection::Rtl => AxisDirection::Left,
        }
    }
}

enum_type!(VerticalDirection => [
    Up,
    Down,
], Down, parse_vertical_direction);

enum_type!(FlexFit => [
    Loose,
    Tight,
], Loose, parse_flex_fit);
