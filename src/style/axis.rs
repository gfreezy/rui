use nom::{IResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
impl Default for Axis {
    fn default() -> Self {
        Axis::Horizontal
    }
}
enum_parser!(parse_axis, Axis => [
    Horizontal, Vertical
]);
