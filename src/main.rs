#![allow(unused)]
mod atom;
mod box_constraints;
mod context;
mod id;
mod key;
mod state;
mod tree;
mod view;
mod widget;
mod widgets;

use druid_shell::kurbo::Insets;
use druid_shell::piet::{Color, Piet, RenderContext};
use widgets::button::Button;

use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, GlobalCtx, LayoutCtx, PaintCtx};
use crate::widgets::box_container::BoxContainer;
use crate::widgets::label::Label;
use crate::widgets::{Event, Widget};
use druid_shell::{
    Application, HotKey, Menu, MouseEvent, Region, SysMods, WinHandler, WindowBuilder, WindowHandle,
};
use std::any::Any;
use tap::Tap;

fn view() -> Box<dyn Widget> {
    let label =
        Label::new("hello", Color::rgb8(40, 200, 145)).tap_mut(|label| label.set_font_size(60.));
    let button = Button::new(label, || println!("clicked"));
    let container =
        BoxContainer::new(Box::new(button)).tap_mut(|c| c.set_padding(Insets::uniform(20.)));
    let container2 =
        BoxContainer::new(Box::new(container)).tap_mut(|c| c.set_padding(Insets::uniform(20.)));
    Box::new(container2)
}

struct Handler {
    handle: WindowHandle,
    widget: Box<dyn Widget>,
}

impl Handler {
    pub fn new(widget: Box<dyn Widget>) -> Self {
        Handler {
            handle: WindowHandle::default(),
            widget,
        }
    }
}

impl WinHandler for Handler {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {}

    fn paint(&mut self, piet: &mut Piet, _invalid: &Region) {
        let g_ctx = GlobalCtx {
            window: self.handle.clone(),
            text: piet.text().clone(),
        };
        let mut layout_ctx = LayoutCtx {
            global_ctx: g_ctx.clone(),
        };
        let mut paint_ctx = PaintCtx {
            global_ctx: g_ctx.clone(),
            render_ctx: piet,
        };
        self.widget.layout(
            &mut layout_ctx,
            BoxConstraints::tight(self.handle.get_size()),
        );
        self.widget.paint(&mut paint_ctx);
    }

    fn mouse_down(&mut self, event: &MouseEvent) {
        let mut event_ctx = EventCtx {};
        self.widget
            .event(&mut event_ctx, &Event::MouseDown(event.clone()));
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        Application::global().quit();
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

fn run_app(widget: Box<dyn Widget>) {
    let mut file_menu = Menu::new();
    file_menu.add_item(
        0x100,
        "E&xit",
        Some(&HotKey::new(SysMods::Cmd, "q")),
        true,
        false,
    );
    file_menu.add_item(
        0x101,
        "O&pen",
        Some(&HotKey::new(SysMods::Cmd, "o")),
        true,
        false,
    );
    let mut menubar = Menu::new();
    menubar.add_dropdown(Menu::new(), "Application", true);
    menubar.add_dropdown(file_menu, "&File", true);

    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());
    builder.set_handler(Box::new(Handler::new(widget)));
    builder.set_title("Test");
    builder.set_menu(menubar);

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}

fn main() {
    run_app(view())
}
