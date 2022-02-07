#[macro_use]
mod macros;
pub(crate) mod alignment;
pub(crate) mod arrangement;
pub(crate) mod axis;
pub(crate) mod border;
pub(crate) mod draw;
pub(crate) mod layout;
pub(crate) mod padding;
pub(crate) mod size;
pub(crate) mod text;
pub(crate) mod utils;

use nom::multi::many1;

use utils::ws;

use druid_shell::{
    kurbo::Insets,
    piet::{Color, FontFamily, FontStyle, FontWeight, TextAlignment},
};

use alignment::{
    parse_horizontal_alignment, parse_vertical_alignment, HorizontalAlignment, VerticalAlignment,
};
use arrangement::{
    parse_horizontal_arrangement, parse_vertical_arrangement, HorizontalArrangement,
    VerticalArrangement,
};
use axis::{parse_axis, Axis};
use border::{parse_border, Border};
use draw::{parse_alpha, parse_background, parse_color, Alpha, Background};
use padding::parse_insets;
use size::{
    parse_fill_max_height, parse_fill_max_width, parse_height, parse_max_height, parse_max_width,
    parse_min_height, parse_min_width, parse_spacing, parse_width, FillMaxHeight, FillMaxWidth,
    Height, MaxHeight, MaxWidth, MinHeight, MinWidth, Spacing, Width,
};
use text::{FontSize, LineBreaking};

use crate::style::{
    alignment::parse_text_alignment,
    text::{
        parse_font_family, parse_font_size, parse_font_style, parse_font_weight,
        parse_line_breaking,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct Style {
    pub name: String,
    pub axis: Axis,
    pub spacing: Spacing,
    pub vertical_alignment: VerticalAlignment,
    pub horizontal_alignment: HorizontalAlignment,
    pub vertical_arrangement: VerticalArrangement,
    pub horizontal_arrangement: HorizontalArrangement,
    pub border: Border,
    pub alpha: Alpha,
    pub background: Background,
    pub insets: Insets,
    pub width: Width,
    pub height: Height,
    pub min_width: MinWidth,
    pub max_width: MaxWidth,
    pub min_height: MinHeight,
    pub max_height: MaxHeight,
    pub fill_max_width: FillMaxWidth,
    pub fill_max_height: FillMaxHeight,
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color,
    pub line_breaking: LineBreaking,
    pub text_alignment: TextAlignment,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            name: Default::default(),
            axis: Default::default(),
            spacing: Default::default(),
            vertical_alignment: Default::default(),
            horizontal_alignment: Default::default(),
            vertical_arrangement: Default::default(),
            horizontal_arrangement: Default::default(),
            border: Default::default(),
            alpha: Default::default(),
            background: Default::default(),
            insets: Insets::ZERO,
            width: Default::default(),
            height: Default::default(),
            min_width: Default::default(),
            max_width: Default::default(),
            min_height: Default::default(),
            max_height: Default::default(),
            fill_max_width: false.into(),
            fill_max_height: false.into(),
            font_family: Default::default(),
            font_size: Default::default(),
            font_weight: Default::default(),
            font_style: Default::default(),
            color: Color::from_hex_str("#000").unwrap(),
            line_breaking: Default::default(),
            text_alignment: Default::default(),
        }
    }
}

style_parser!(
    parse_rule,
    Style,
    [
        axis => parse_axis => Axis,
        spacing => parse_spacing => Spacing,
        vertical_alignment => parse_vertical_alignment => VerticalAlignment,
        horizontal_alignment => parse_horizontal_alignment => HorizontalAlignment,
        vertical_arrangement => parse_vertical_arrangement => VerticalArrangement,
        horizontal_arrangement => parse_horizontal_arrangement => HorizontalArrangement,
        border => parse_border => Border,
        alpha => parse_alpha => Alpha,
        background => parse_background => Background,
        insets => parse_insets => Insets,
        width => parse_width => Width,
        height => parse_height => Height,
        min_width => parse_min_width => MinWidth,
        max_width => parse_max_width => MaxWidth,
        min_height => parse_min_height => MinHeight,
        max_height => parse_max_height => MaxHeight,
        fill_max_width => parse_fill_max_width => FillMaxWidth,
        fill_max_height => parse_fill_max_height => FillMaxHeight,
        font_family => parse_font_family => FontFamily,
        font_size => parse_font_size => FontSize,
        font_weight => parse_font_weight => FontWeight,
        font_style => parse_font_style => FontStyle,
        color => parse_color => Color,
        line_breaking => parse_line_breaking => LineBreaking,
        text_alignment => parse_text_alignment => TextAlignment
    ]
);

#[derive(thiserror::Error, Debug)]
#[error("parse error: {0}")]
pub struct ParseStyleError(String);

pub(crate) fn parse_style_content(input: &str) -> Result<Vec<Style>, ParseStyleError> {
    let (_, styles) =
        ws(many1(ws(parse_rule)))(input).map_err(|e| ParseStyleError(e.to_string()))?;
    Ok(styles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_style_eq() {
        let this = Style::default();
        let other = Style::default();

        assert!(this == other);
    }

    #[test]
    fn test_parse_rule() {
        let ret = parse_style_content(
            r#"
    .classname {
        axis: horizontal;
        spacing: 10;
        vertical-alignment: top;
        horizontal-alignment: center;
        vertical-arrangement: center;
        horizontal-arrangement: center;
        border: 1 solid #fff;
        alpha: 0.5;
        background: #faa;
        insets: 1 20;
        width: 10;
        height: 40;
        min-width: 0;
        max-width: 20;
        min-height: 0;
        max-height: 20;
        fill-max-width: true;
        fill-max-height: false;
        font-family: Sans_Serif;
        font-size: 14;
        font-weight: bold;
        font-style: regular;
        color: #fff;
        line-breaking: clip;
        text-alignment: center;
    }
    "#,
        );
        assert!(ret.is_ok(), "error: {:?}", ret);
    }
}
