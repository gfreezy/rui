use nom::IResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn flip(&self) -> Axis {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

impl Default for Axis {
    fn default() -> Self {
        Axis::Horizontal
    }
}
enum_parser!(parse_axis, Axis => [
    Horizontal, Vertical
]);
