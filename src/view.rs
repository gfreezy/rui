use crate::tree::cx::Cx;
use crate::widgets::label::Label;
use crate::widgets::AnyWidget;
use druid_shell::piet::Color;
use std::fmt::Debug;
use std::panic::Location;

pub trait View: Debug {
    fn same(&self, other: &Self) -> bool;
    fn make_widget(&self) -> AnyWidget;
}

pub type AnyView = Box<dyn View + 'static>;

#[derive(Debug, PartialEq)]
struct LabelView {
    text: String,
    color: Color,
    font_size: f64,
    wrap_width: f64,
}

impl LabelView {
    pub fn new(text: impl Into<String>) -> Label {
        Label(text.into())
    }

    #[track_caller]
    pub fn build(self, cx: &mut Cx) {
        cx.leaf_view(Box::new(self), Location::caller());
    }
}

impl View for LabelView {
    fn same(&self, other: &Self) -> bool {
        self == other
    }

    fn make_widget(&self) -> AnyWidget {
        Box::new(Label::new(self.text.clone(), self.color.clone()))
    }
}
