use nom::number::complete::double;

use super::utils::parse_bool;

simple_type!(
    [Width, f64, f64::NAN],
    [Height, f64, f64::NAN],
    [MinWidth, f64, f64::NAN],
    [MaxWidth, f64, f64::NAN],
    [MinHeight, f64, f64::NAN],
    [MaxHeight, f64, f64::NAN],
    [AspectRatio, f64, f64::NAN],
    [Spacing, f64, 0.],
    [FillMaxWidth, bool, false],
    [FillMaxHeight, bool, false]
);

simple_attr_parser!(parse_spacing, Spacing, double);
simple_attr_parser!(parse_width, Width, double);
simple_attr_parser!(parse_min_width, MinWidth, double);
simple_attr_parser!(parse_max_width, MaxWidth, double);
simple_attr_parser!(parse_height, Height, double);
simple_attr_parser!(parse_min_height, MinHeight, double);
simple_attr_parser!(parse_max_height, MaxHeight, double);
simple_attr_parser!(parse_fill_max_width, FillMaxWidth, parse_bool);
simple_attr_parser!(parse_fill_max_height, FillMaxHeight, parse_bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width() {
        assert_eq!(&Width(10.), &Width(10.));
        assert_eq!(Width(0.), Width(0.));
        assert_eq!(Width(30.), Width(30.));
        assert_eq!(&Width(f64::NAN), &Width(f64::NAN));
    }
}
