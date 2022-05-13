use style::layout::AxisDirection;
use style::Style;

use style::alignment::Alignment;

use crate::sliver_constraints::CacheExtent;
use crate::ui::Ui;
use crate::widgets::sliver_list::SliverChildDelegate;

pub fn flex(ui: &mut Ui, mut style: Style, content: impl FnMut(&mut Ui)) {
    style.widget_name = "flex".to_string();
    crate::widgets::flex::Flex::new(
        style.axis,
        style.main_axis_size,
        style.main_axis_alignment,
        style.cross_axis_alignment,
        style.text_direction,
        style.vertical_direction,
    )
    .build(ui, content);
}

pub fn column(ui: &mut Ui, mut style: Style, content: impl FnMut(&mut Ui)) {
    style.axis = style::axis::Axis::Vertical;
    style.widget_name = "column".to_string();
    flex(ui, style, content);
}

pub fn row(ui: &mut Ui, mut style: Style, content: impl FnMut(&mut Ui)) {
    style.axis = style::axis::Axis::Horizontal;
    style.widget_name = "row".to_string();
    flex(ui, style, content);
}

pub fn debug(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    crate::widgets::debug::Debug.build(ui, content);
}

pub fn flexible(ui: &mut Ui, mut style: Style, content: impl FnMut(&mut Ui)) {
    let flex = style.flex.value();
    let flex_fit = style.flex_fit;
    crate::widgets::flex::Flexible::new(flex, flex_fit).build(ui, content);
}

pub fn expand(ui: &mut Ui, content: impl FnMut(&mut Ui)) {
    let style = Style {
        flex: 1.0.into(),
        flex_fit: style::layout::FlexFit::Loose,
        ..Default::default()
    };
    flexible(ui, style, content);
}

pub fn text(ui: &mut Ui, text: &str, style: Style) {
    crate::widgets::text::Text::new(text).style(style).build(ui);
}

pub fn button(ui: &mut Ui, text: &str, click: impl FnMut() + 'static) {
    crate::widgets::button::Button::new()
        .text_align(druid_shell::piet::TextAlignment::Start)
        .labeled(ui, text, click);
}

pub fn viewport(ui: &mut Ui, style: Style, content: impl FnMut(&mut Ui)) {
    crate::widgets::viewport::Viewport::new(
        style.axis_direction,
        style.cross_axis_direction,
        0.0,
        None,
        CacheExtent::Viewport(1.),
    )
    .build(ui, content)
}

pub fn sliver_to_box(ui: &mut Ui, local_key: String, content: impl FnMut(&mut Ui)) {
    crate::widgets::sliver_to_box::SliverToBox.build(ui, local_key, content);
}

pub fn sliver_list(ui: &mut Ui, delegate: impl SliverChildDelegate + 'static) {
    crate::widgets::sliver_list::SliverList::new(Box::new(delegate)).build(ui)
}
