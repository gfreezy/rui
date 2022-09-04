use druid_shell::kurbo::Insets;
use nom::{combinator::map_res, multi::many1, number::complete::double, IResult};

use super::utils::{make_error, ws};

pub(crate) fn parse_insets(input: &str) -> IResult<&str, Insets> {
    map_res(many1(ws(double)), |v| match v.as_slice() {
        &[v] => Ok(Insets::uniform(v)),
        &[y, x] => Ok(Insets::uniform_xy(x, y)),
        &[top, right, bottom, left] => Ok(Insets::new(left, top, right, bottom)),
        _ => Err(make_error("invalid hex color")),
    })(input)
}
