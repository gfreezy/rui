use crate::box_constraints::BoxConstraints;
use crate::context::{ContextState, LayoutCtx, PaintCtx};
use crate::id::{ChildCounter, ChildId};
use crate::tree::{Child, Children};
use crate::ui::Ui;
use crate::widgets::sized_box::SizedBox;
use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::{Piet, RenderContext};
use druid_shell::{
    Application, HotKey, Menu, MouseEvent, Region, SysMods, WinHandler, WindowBuilder, WindowHandle,
};
use std::any::Any;

pub struct App {
    name: String,
}

impl App {
    pub fn new(name: impl Into<String>) -> Self {
        App { name: name.into() }
    }

    pub fn run(self, mut fun: impl FnMut(&mut Ui) + 'static) {
        let app = Application::new().unwrap();

        let mut file_menu = Menu::new();
        file_menu.add_item(
            0x100,
            "E&xit",
            Some(&HotKey::new(SysMods::Cmd, "q")),
            true,
            false,
        );

        let mut menubar = Menu::new();
        menubar.add_dropdown(file_menu, "Application", true);

        let mut builder = WindowBuilder::new(app.clone());
        builder.set_handler(Box::new(AppWidget::new(move |ui| {
            SizedBox::new().build(ui, |ui| fun(ui))
        })));
        builder.set_title(self.name);
        builder.set_menu(menubar);

        let window = builder.build().unwrap();
        window.show();

        app.run(None);
    }
}

struct AppWidget {
    handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root: Children,
    child_counter: ChildCounter,
    focus_widget: Option<ChildId>,
    mouse_pos: Option<Point>,
}

impl AppWidget {
    pub fn new(app: impl FnMut(&mut Ui) + 'static) -> Self {
        AppWidget {
            handle: WindowHandle::default(),
            app: Box::new(app),
            root: Children::new(),
            child_counter: ChildCounter::new(),
            focus_widget: None,
            mouse_pos: None,
        }
    }

    fn layout(&mut self, size: Size) {
        let mut context_state = ContextState {
            window: &self.handle,
            text: self.handle.text(),
        };
        let mut cx = Ui::new(&mut self.root, &mut context_state, &mut self.child_counter);
        (self.app)(&mut cx);

        let mouse_pos = self.mouse_pos;

        let mut context_state = ContextState {
            window: &self.handle,
            text: self.handle.text(),
        };

        let root = &mut self.root.renders[0];
        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: &mut root.state,
            mouse_pos,
        };

        root.state.size = root.object.layout(
            &mut layout_ctx,
            &BoxConstraints::tight(size),
            &mut root.children,
        );
    }
}

impl WinHandler for AppWidget {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn size(&mut self, size: Size) {
        self.layout(size)
    }

    fn prepare_paint(&mut self) {
        self.layout(self.handle.get_size())
    }

    fn paint(&mut self, piet: &mut Piet, invalid: &Region) {
        println!("paint");

        let mut context_state = ContextState {
            window: &self.handle,
            text: self.handle.text(),
        };

        let root = &mut self.root.renders[0];
        let mut paint_ctx = PaintCtx {
            context_state: &mut context_state,
            child_state: &mut root.state,
            region: invalid.clone(),
            render_ctx: piet,
        };

        root.object.paint(&mut paint_ctx, &mut root.children);
    }

    fn mouse_down(&mut self, event: &MouseEvent) {}

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
