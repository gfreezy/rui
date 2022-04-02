use std::{panic::Location, time::Instant};

use druid_shell::{
    kurbo::{Point, Rect, Size, Vec2},
    piet::{Color, PaintBrush, Piet, RenderContext},
    Region, WindowHandle,
};

use crate::{
    app::{PendingWindow, WindowSizePolicy},
    app_state::{CommandQueue, Handled},
    box_constraints::BoxConstraints,
    context::{ContextState, EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    ext_event::ExtEventSink,
    id::{ChildId, WindowId},
    lifecycle::{InternalLifeCycle, LifeCycle},
    menu::{MenuItemId, MenuManager},
    perf::{measure_time, FPSCounter},
    text::layout::TextLayout,
    tree::{Element, ElementState},
    ui::Ui,
    widgets::window_container::WindowContainer,
};

pub struct Window {
    id: WindowId,
    size: Size,
    size_policy: WindowSizePolicy,
    pub(crate) handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root: Element,
    phatom_root_id: ChildId,
    invalid: Region,
    pub(crate) menu: Option<MenuManager>,
    ext_handle: ExtEventSink,
    fps_counter: FPSCounter,
}

impl Window {
    pub fn new(
        id: WindowId,
        handle: WindowHandle,
        pending: PendingWindow,
        ext_handle: ExtEventSink,
    ) -> Self {
        Window {
            id,
            size: Size::ZERO,
            size_policy: pending.size_policy,
            handle,
            app: pending.root,
            menu: pending.menu,
            phatom_root_id: ChildId::next(),
            root: Element::new(Location::caller().into(), WindowContainer),
            invalid: Region::EMPTY,
            ext_handle,
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

    // #[instrument(skip(self, piet))]
    pub(crate) fn paint(&mut self, piet: &mut Piet, invalid: &Region, queue: &mut CommandQueue) {
        if self.root.needs_layout() {
            self.layout(queue);
        }

        let Self {
            handle,
            ext_handle,
            phatom_root_id,
            root,
            size,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),

            text: handle.text(),
            command_queue: queue,
        };

        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));
        let mut paint_ctx = PaintCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            region: invalid.clone(),
            render_ctx: piet,
        };
        root.paint(&mut paint_ctx);
        let fps = self.fps_counter.tick();
        draw_fps(fps, handle.get_size(), &mut paint_ctx);
    }

    // #[instrument(skip(self))]
    pub(crate) fn layout(&mut self, command_queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root,
            phatom_root_id,
            size,
            invalid,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
        };

        root_state.size = root.layout_box(
            &mut layout_ctx,
            &(BoxConstraints::new(Size::ZERO, self.size).into()),
        );
        let mut ctx = LifeCycleCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
        };
        root.lifecycle(
            &mut ctx,
            &LifeCycle::Internal(InternalLifeCycle::ParentWindowOrigin),
        );
        invalid.union_with(&root_state.invalid);
    }

    // #[instrument(skip(self))]
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
            phatom_root_id,
            size,
            invalid,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue: queue,
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut event_ctx = EventCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            is_active: false,
            is_handled: false,
        };

        self.root.event(&mut event_ctx, &event);
        let is_handled = event_ctx.is_handled;
        invalid.union_with(&root_state.invalid);

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

    pub(crate) fn lifecycle(&mut self, command_queue: &mut CommandQueue, event: &LifeCycle) {
        let Self {
            handle,
            ext_handle,
            root,
            phatom_root_id,
            size,
            invalid,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut ctx = LifeCycleCtx {
            child_state: &mut root_state,
            context_state: &mut context_state,
        };

        root.lifecycle(&mut ctx, event);
        invalid.union_with(&root_state.invalid);
    }

    pub(crate) fn update(&mut self, command_queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root,
            app,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
        };
        let mut cx = Ui::new(&mut root.children, &mut context_state);
        measure_time("app::update", || {
            app(&mut cx);
        });
        cx.cleanup_tree();
        root.merge_child_states_up();
    }

    pub(crate) fn invalidate_and_finalize(&mut self) {
        let Self {
            handle,
            root,
            invalid,
            ..
        } = self;

        if root.needs_layout() {
            handle.invalidate();
        } else {
            let invalid_rect = invalid.bounding_box();
            if !invalid_rect.is_empty() {
                handle.invalidate_rect(invalid_rect);
            }
        }
        invalid.clear();
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
