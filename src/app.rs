use std::any::Any;

use druid_shell::kurbo::{Insets, Point, Rect, Size, Vec2};
use druid_shell::piet::{Color, PaintBrush, Piet, RenderContext};
use druid_shell::{
    Application, HotKey, KeyEvent, Menu, Monitor, MouseEvent, Region, Screen, SysMods, WinHandler,
    WindowBuilder, WindowHandle,
};
use tracing::{debug, instrument};

use crate::box_constraints::BoxConstraints;
use crate::context::{ContextState, EventCtx, LayoutCtx, PaintCtx};
use crate::event::Event;
use crate::id::{ChildCounter, ChildId};
use crate::perf::FPSCounter;
use crate::text::layout::{self, TextLayout};
use crate::tree::{ChildState, Children};
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

        let primary_monitor = Screen::get_monitors()
            .into_iter()
            .find(Monitor::is_primary)
            .unwrap();
        let window_size = Size::new(800., 600.);
        let virtual_rect = primary_monitor.virtual_rect();
        let x = (virtual_rect.x1 - window_size.width + virtual_rect.x0) / 2.;
        let y = (virtual_rect.y1 - window_size.height + virtual_rect.y0) / 2.;
        let window_offset = Point::new(x, y);
        builder.set_size(window_size);
        builder.set_position(window_offset);

        let window = builder.build().unwrap();
        window.show();

        app.run(None);
    }
}

struct AppWidget {
    handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root: Children,
    root_state: ChildState,
    child_counter: ChildCounter,
    mouse_pos: Option<Point>,
    fps_counter: FPSCounter,
}

impl AppWidget {
    pub fn new(app: impl FnMut(&mut Ui) + 'static) -> Self {
        let mut counter = ChildCounter::new();
        AppWidget {
            handle: WindowHandle::default(),
            app: Box::new(app),
            root: Children::new(),
            root_state: ChildState::new(counter.generate_id(), None),
            child_counter: counter,
            mouse_pos: None,
            fps_counter: FPSCounter::new(),
        }
    }

    fn run_app(
        handle: &WindowHandle,
        app: &mut Box<dyn FnMut(&mut Ui)>,
        root: &mut Children,
        child_counter: &mut ChildCounter,
    ) {
        let mut context_state = ContextState {
            window: handle.clone(),
            text: handle.text(),
        };
        let mut cx = Ui::new(root, &mut context_state, child_counter);
        app(&mut cx);
    }

    #[instrument(skip(self))]
    fn layout(&mut self, bc: &BoxConstraints) {
        let Self {
            handle,
            root,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            text: handle.text(),
        };
        let child = &mut root.renders[0];
        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: root_state,
        };

        root_state.size = child.layout(&mut layout_ctx, &bc.into());
    }

    #[instrument(skip(self))]
    fn event(&mut self, event: Event) -> bool {
        let Self {
            handle,
            root,
            mouse_pos,
            app,
            child_counter,
            root_state,
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
            window: handle.clone(),
            text: handle.text(),
        };

        let child = &mut root.renders[0];
        let mut event_ctx = EventCtx {
            context_state: &mut context_state,
            child_state: root_state,
            is_active: false,
            is_handled: false,
        };

        child.event(&mut event_ctx, &event);

        let is_handled = event_ctx.is_handled;
        if is_handled {
            AppWidget::run_app(&handle, app, root, child_counter);
        }

        let child = &mut root.renders[0];

        if child.needs_layout() {
            handle.invalidate();
        } else {
            let invalid_rect = root_state.invalid.bounding_box();
            root_state.invalid.clear();
            handle.invalidate_rect(invalid_rect);
        }
        is_handled
    }
}

fn draw_fps(fps: usize, window_size: Size, paint_ctx: &mut PaintCtx) {
    let mut layout: TextLayout<String> = TextLayout::from_text(format!("{}", fps));
    layout.rebuild_if_needed(&mut paint_ctx.text());
    let text_size = layout.size();
    let win_size = window_size;
    let origin = Point::new(win_size.width - text_size.width, 0.);
    let text_rect = Rect::from_origin_size(origin, text_size) - Vec2::new(5., 0.);
    let bg_rect = text_rect.inset(5.);
    paint_ctx.fill(
        bg_rect,
        &PaintBrush::Color(Color::from_hex_str("#fff").unwrap()),
    );
    paint_ctx.draw_text(layout.layout().unwrap(), text_rect.origin());
}

impl WinHandler for AppWidget {
    #[instrument(skip(self, handle))]
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    #[instrument(skip(self))]
    fn size(&mut self, size: Size) {
        if self.root.is_empty() {
            let Self {
                handle,
                root,

                app,
                child_counter,
                ..
            } = self;
            AppWidget::run_app(&handle, app, root, child_counter);
        } else {
            self.event(Event::WindowSize(size));
        }
    }

    fn prepare_paint(&mut self) {}

    #[instrument(skip(self, piet))]
    fn paint(&mut self, piet: &mut Piet, invalid: &Region) {
        self.layout(&BoxConstraints::new(Size::ZERO, self.handle.get_size()));

        let Self {
            handle,
            root,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            text: handle.text(),
        };

        let child = &mut root.renders[0];
        let mut paint_ctx = PaintCtx {
            context_state: &mut context_state,
            child_state: &root_state,
            region: invalid.clone(),
            render_ctx: piet,
        };
        child.paint(&mut paint_ctx);
        draw_fps(self.fps_counter.tick(), handle.get_size(), &mut paint_ctx);
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        self.event(Event::KeyUp(event))
    }

    fn key_up(&mut self, event: KeyEvent) {
        self.event(Event::KeyDown(event));
    }

    #[instrument(skip(self))]
    fn wheel(&mut self, mouse_event: &MouseEvent) {
        self.event(Event::Wheel(mouse_event.clone()));
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
