macro_rules! alt {
    ($a:expr) => {
        $a
    };
    ($a:expr, $b:expr) => {
        nom::branch::alt(($a, $b))
    };
    ($a:expr, $b:expr, $c:expr) => {
        nom::branch::alt(($a, $b, $c))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        nom::branch::alt(($a, $b, $c, $d))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e, $f))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e, $f, $g))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e, $f, $g, $h))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e, $f, $g, $h, $i))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr) => {
        nom::branch::alt(($a, $b, $c, $d, $e, $f, $g, $h, $i, $j))
    };
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr, $($more:expr),*) => {
        alt!(nom::branch::alt(($a, $b, $c, $d, $e, $f, $g, $h, $i, $j)), $($more),*)
    };
}
///
/// ```rust
/// style_parser!(
/// parse_text_style_rule,
/// TextStyle,
/// [
///     font_family => parse_font_family,
///     font_size => parse_font_size,
/// ],
/// [
///     font_family => FontFamily,
///     font_size => FontSize,
/// ]
/// );
/// ```
macro_rules! style_parser {
    ($fun:tt, $ty:ty, [$($attr_name:tt => $parse_fun:tt => $field_ty:ty),*]) => {

        pub(crate) fn $fun(input: &str) -> nom::IResult<&str, $ty> {
            let attrs = std::rc::Rc::new(std::cell::RefCell::new(anymap2::AnyMap::new()));

            fn parse_attr_name(name: &str) -> impl FnMut(&str) -> nom::IResult<&str, &str> {
                use convert_case::Casing;
                let kebab_case = name.to_case(convert_case::Case::Kebab);

                let original = name.to_string();
                move |input: &str| {
                    nom::branch::alt((
                        nom::bytes::complete::tag_no_case(original.as_str()),
                        nom::bytes::complete::tag_no_case(kebab_case.as_str()),
                    ))(input)
                }
            }

            let parser_tuple = alt!($(
                nom::sequence::tuple((
                    crate::style::utils::ws(parse_attr_name(stringify!($attr_name))),
                    crate::style::utils::ws(nom::bytes::complete::tag(":")),
                    crate::style::utils::ws(nom::combinator::map(
                        $parse_fun,
                        clone!([attrs] move |v| {
                            attrs.borrow_mut().insert(v);
                        }),
                    )),
                ))
            ),*);

            let parse_attrs = nom::multi::many1(crate::style::utils::ws(nom::sequence::terminated(
                crate::style::utils::ws(parser_tuple),
                crate::style::utils::ws(nom::bytes::complete::tag(";")),
            )));
            let parse_attrs_block = nom::sequence::delimited(crate::style::utils::ws(nom::bytes::complete::tag("{")), crate::style::utils::ws(parse_attrs), crate::style::utils::ws(nom::bytes::complete::tag("}")));
            let parse_class_name = nom::combinator::map(nom::sequence::pair(nom::bytes::complete::tag("."), nom::character::complete::alphanumeric1), |(_, name)| {
                format!(".{name}")
            });
            let mut parse_rule = nom::sequence::terminated(crate::style::utils::ws(parse_class_name), crate::style::utils::ws(parse_attrs_block));

            let (left, class_name) = parse_rule(input)?;
            let mut style_attrs = attrs.take();
            let mut style: $ty = Default::default();
            style.name = class_name;


            $(
                if let Some(v) = style_attrs.remove::<$field_ty>() {
                    style.$attr_name = v.into();
                }
            )*

            Ok((left, style))
        }
    };
}

///
/// ```rust
/// simple_attr_parser!(parse_width, Width, double)
/// ```
macro_rules! simple_attr_parser {
    ($fn_name:tt, $ty:ident, $parser:ident) => {
        pub(crate) fn $fn_name(input: &str) -> nom::IResult<&str, $ty> {
            nom::combinator::map($parser, |v| $ty(v))(input)
        }
    };
}

///
/// ```rust
/// enum_parser!(parse_layout, Layout => [Flex])
/// ```
macro_rules! enum_parser {
    ($parser_name:tt, $enum_ty:tt => [$($variant:tt),*]) => {
        pub(crate) fn $parser_name<'a>(input: &'a str) -> IResult<&'a str, $enum_ty> {
            fn parse_kebab_case(name: &str) -> impl FnMut(&str) -> nom::IResult<&str, &str> {
                use convert_case::Casing;
                let kebab_case = name.to_case(convert_case::Case::Kebab);

                let original = name.to_string();
                move |input: &str| {
                    nom::branch::alt((
                        nom::bytes::complete::tag_no_case(original.as_str()),
                        nom::bytes::complete::tag_no_case(kebab_case.as_str()),
                    ))(input)
                }
            }

            alt!(
                $(
                    nom::combinator::map(parse_kebab_case(std::stringify!($variant)), |_| $enum_ty::$variant)
                ),*
            )(input)
        }
    };
}

///
/// ```rust
/// simple_type!([StructType, f64, f64::NAN])
/// simple_type!([Struct, bool, false], [StructType, f64, f64::NAN])
/// ```
///
macro_rules! simple_type {
    ([$ident:tt, f64, $default:expr]) => {
        #[derive(Debug, PartialOrd, Clone, Copy)]
        pub struct $ident(pub(crate) f64);

        impl $ident {
            pub(crate) fn value(&self) -> f64 {
                self.0
            }

            pub(crate) fn is_normal(&self) -> bool {
                self.0.is_normal()
            }

            pub(crate) fn is_infinite(&self) -> bool {
                self.0.is_infinite()
            }

            pub(crate) fn is_nan(&self) -> bool {
                self.0.is_nan()
            }
        }

        impl From<$ident> for f64 {
            fn from(a: $ident) -> Self {
                a.0
            }
        }

        impl From<f64> for $ident {
            fn from(a: f64) -> Self {
                $ident(a)
            }
        }

        impl PartialEq<$ident> for $ident {
            fn eq(&self, other: &$ident) -> bool {
                match (self.0.classify(), other.0.classify()) {
                    (std::num::FpCategory::Nan, std::num::FpCategory::Nan) => true,
                    (std::num::FpCategory::Infinite, std::num::FpCategory::Infinite) => true,
                    (std::num::FpCategory::Zero, std::num::FpCategory::Zero) => true,
                    (std::num::FpCategory::Normal, std::num::FpCategory::Normal) => (self.0 - other.0).abs() < 0.1,
                    _ => false
                }

            }
        }

        impl Default for $ident {
            fn default() -> Self {
                Self($default)
            }
        }
    };
    ([$ident:tt, $inner_ty:tt, $default:expr]) => {
        #[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
        pub struct $ident(pub(crate) $inner_ty);

        impl $ident {
            pub(crate) fn value(&self) -> $inner_ty {
                self.0
            }
        }

        impl From<$ident> for $inner_ty {
            fn from(a: $ident) -> Self {
                a.0
            }
        }

        impl From<$inner_ty> for $ident {
            fn from(a: $inner_ty) -> Self {
                $ident(a)
            }
        }

        impl Default for $ident {
            fn default() -> Self {
                Self($default)
            }
        }
    };
    ($([$ident:tt, $inner_ty:tt, $default:expr]),*) => {
        $(
            simple_type!([$ident, $inner_ty, $default]);
        )*
    };
}
