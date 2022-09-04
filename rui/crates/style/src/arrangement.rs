use nom::IResult;

enum_type!(HorizontalArrangement => [
    Leading, Trailing, Center, SpaceBetween, SpaceEvenly, SpaceAround
], Leading, parse_horizontal_arrangement);

enum_type!(VerticalArrangement => [
    Bottom, Center, Top, SpaceBetween, SpaceEvenly, SpaceAround
], Top, parse_vertical_arrangement);
