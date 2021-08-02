use std::any::Any;

use druid_shell::kurbo::{Point, Size};
use druid_shell::piet::Piet;
use druid_shell::{
    Application, HotKey, Menu, MouseEvent, Region, SysMods, WinHandler, WindowBuilder, WindowHandle,
};
use tracing::{debug, instrument};

use crate::box_constraints::BoxConstraints;
use crate::context::{ContextState, EventCtx, LayoutCtx, PaintCtx};
use crate::event::Event;
use crate::id::{ChildCounter, ChildId};
use crate::tree::Children;
use crate::ui::Ui;
use crate::widgets::sized_box::SizedBox;

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

    fn run_app(&mut self) {
        let Self {
            handle,
            app,
            root,
            child_counter,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle,
            text: handle.text(),
        };
        let mut cx = Ui::new(root, &mut context_state, child_counter);
        app(&mut cx);
    }

    #[instrument(skip(self))]
    fn layout(&mut self, bc: &BoxConstraints) {
        debug!("AppWidget::layout called");
        let Self { handle, root, .. } = self;

        let mut context_state = ContextState {
            window: handle,
            text: handle.text(),
        };
        let child = &mut root.renders[0];
        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: &mut child.state,
        };

        child.state.size = child
            .object
            .layout(&mut layout_ctx, bc, &mut child.children);
    }

    #[instrument(skip(self))]
    fn event(&mut self, event: Event) {
        let Self {
            handle,
            root,
            mouse_pos,
            ..
        } = self;

        match &event {
            Event::MouseMove(mouse_event)
            | Event::MouseUp(mouse_event)
            | Event::MouseDown(mouse_event)
            | Event::Wheel(mouse_event) => {
                *mouse_pos = Some(mouse_event.pos);
            }
            _ => {}
        };

        let mut context_state = ContextState {
            window: handle,
            text: handle.text(),
        };

        let child = &mut root.renders[0];
        let mut event_ctx = EventCtx {
            context_state: &mut context_state,
            child_state: &mut child.state,
            is_handled: false,
        };

        child
            .object
            .event(&mut event_ctx, &event, &mut child.children);
    }
}

impl WinHandler for AppWidget {
    #[instrument(skip(self, handle))]
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    #[instrument(skip(self))]
    fn size(&mut self, size: Size) {
        debug!("WinHandler::size");
        if self.root.is_empty() {
            self.run_app();
        } else {
            self.event(Event::WindowSize(size));
        }
    }

    fn prepare_paint(&mut self) {
        debug!("WinHandler::prepeare_paint");
    }

    #[instrument(skip(self, piet))]
    fn paint(&mut self, piet: &mut Piet, invalid: &Region) {
        debug!("WinHandler::paint");
        self.layout(&BoxConstraints::new(Size::ZERO, self.handle.get_size()));

        let Self { handle, root, .. } = self;

        let mut context_state = ContextState {
            window: handle,
            text: handle.text(),
        };

        let root = &mut root.renders[0];
        let mut paint_ctx = PaintCtx {
            context_state: &mut context_state,
            child_state: &mut root.state,
            region: invalid.clone(),
            render_ctx: piet,
        };

        root.object.paint(&mut paint_ctx, &mut root.children);
    }

    #[instrument(skip(self))]
    fn mouse_move(&mut self, mouse_event: &MouseEvent) {
        self.event(Event::MouseMove(mouse_event.clone()));
    }

    #[instrument(skip(self))]
    fn mouse_down(&mut self, mouse_event: &MouseEvent) {
        self.event(Event::MouseDown(mouse_event.clone()));
    }

    #[instrument(skip(self))]
    fn mouse_up(&mut self, mouse_event: &MouseEvent) {
        self.event(Event::MouseUp(mouse_event.clone()));
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
