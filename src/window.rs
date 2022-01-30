use druid_shell::{
    kurbo::{Point, Rect, Size, Vec2},
    piet::{Color, PaintBrush, Piet, RenderContext},
    Region, WindowHandle,
};
use tracing::instrument;

use crate::{
    app::{PendingWindow, WindowSizePolicy},
    app_state::{CommandQueue, Handled},
    box_constraints::BoxConstraints,
    context::{ContextState, EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    ext_event::ExtEventSink,
    id::{ChildCounter, WindowId},
    lifecycle::LifeCycle,
    menu::{MenuItemId, MenuManager},
    perf::FPSCounter,
    text::layout::TextLayout,
    tree::{ChildState, Children},
    ui::Ui,
};

pub struct Window {
    id: WindowId,
    size: Size,
    size_policy: WindowSizePolicy,
    pub(crate) handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root: Children,
    pub(crate) menu: Option<MenuManager>,
    root_state: ChildState,
    child_counter: ChildCounter,
    ext_handle: ExtEventSink,
    fps_counter: FPSCounter,
}

impl Window {
    pub fn new(
        id: WindowId,
        handle: WindowHandle,
        pending: PendingWindow,
        ext_handle: ExtEventSink,
        counter: ChildCounter,
    ) -> Self {
        Window {
            id,
            size: Size::ZERO,
            size_policy: pending.size_policy,
            handle: handle,
            app: pending.root,
            menu: pending.menu,
            root: Children::new(),
            ext_handle,
            root_state: ChildState::new(counter.generate_id(), None),
            child_counter: counter,
            fps_counter: FPSCounter::new(),
        }
    }

    pub(crate) fn menu_cmd(&mut self, queue: &mut CommandQueue, cmd_id: MenuItemId) {
        if let Some(menu) = &mut self.menu {
            menu.event(queue, Some(self.id), cmd_id);
        }
        // if let Some((menu, _)) = &mut self.context_menu {
        //     menu.event(queue, Some(self.id), cmd_id, data, env);
        // }
    }

    pub(crate) fn prepare_paint(&mut self) {}

    #[instrument(skip(self, piet))]
    pub(crate) fn paint(&mut self, piet: &mut Piet, invalid: &Region, queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),

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

    #[instrument(skip(self))]
    pub(crate) fn layout(&mut self, command_queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };
        let child = &mut root.renders[0];
        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: root_state,
        };

        root_state.size = child.layout(
            &mut layout_ctx,
            &(BoxConstraints::new(Size::ZERO, self.size).into()),
        );
    }

    #[instrument(skip(self))]
    pub(crate) fn event(&mut self, queue: &mut CommandQueue, event: Event) -> Handled {
        match &event {
            Event::WindowSize(size) => self.size = *size,
            _ => (),
        }

        if let Event::WindowConnected = event {
            self.lifecycle(queue, &LifeCycle::Other);
        }

        let Self {
            handle,
            ext_handle,
            root,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
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

        if matches!(
            (event, self.size_policy),
            (Event::WindowSize(_), WindowSizePolicy::Content)
        ) {
            // Because our initial size can be zero, the window system won't ask us to paint.
            // So layout ourselves and hopefully we resize
            self.layout(queue);
        }

        is_handled.into()
    }

    pub(crate) fn lifecycle(&mut self, _queue: &mut CommandQueue, event: &LifeCycle) {
        let Self {
            handle,
            ext_handle,
            root,
            app: _,
            child_counter: _,
            root_state,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };

        let mut ctx = LifeCycleCtx {
            child_state: root_state,
            context_state: &mut context_state,
        };
        let child = &mut root.renders[0];

        child.lifecycle(&mut ctx, event);
    }

    pub(crate) fn update(&mut self, _queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root,
            app,
            child_counter,
            ..
        } = self;
        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };
        let mut cx = Ui::new(root, &mut context_state, child_counter);
        app(&mut cx);
    }

    pub(crate) fn invalidate_and_finalize(&mut self) {
        let Self {
            handle,
            root,
            root_state,
            ..
        } = self;
        let child = &mut root.renders[0];

        if child.needs_layout() {
            // debug!("needs layout");
            handle.invalidate();
        } else {
            let invalid_rect = root_state.invalid.bounding_box();
            root_state.invalid.clear();
            handle.invalidate_rect(invalid_rect);
            // debug!("invalidate rect: {invalid_rect}");
        }
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
