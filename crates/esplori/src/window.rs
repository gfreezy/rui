use std::panic::Location;

use druid_shell::{
    kurbo::{Point, Rect, Size, Vec2},
    piet::{Color, PaintBrush, Piet, RenderContext},
    Region, WindowHandle,
};
use generational_indextree::{Arena, NodeId};

use crate::{
    app::{PendingWindow, WindowSizePolicy},
    app_state::{CommandQueue, Handled},
    box_constraints::BoxConstraints,
    context::{EventCtx, GlobalState, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    ext_event::ExtEventSink,
    id::WindowId,
    lifecycle::LifeCycle,
    menu::{MenuItemId, MenuManager},
    perf::FPSCounter,
    text::layout::TextLayout,
    tree::{Element, ElementId, RenderObject, UiState},
    ui::{node_to_object_mut, Ui},
    widgets::window_container::WindowContainer,
};

pub struct Window {
    id: WindowId,
    size: Size,
    size_policy: WindowSizePolicy,
    pub(crate) handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root_element: ElementId,
    invalid: Region,
    pub(crate) menu: Option<MenuManager>,
    arena: Arena<Element>,
    ext_handle: ExtEventSink,
    fps_counter: FPSCounter,
}

impl Window {
    pub fn new(
        id: WindowId,
        mut arena: Arena<Element>,
        handle: WindowHandle,
        pending: PendingWindow,
        ext_handle: ExtEventSink,
    ) -> Self {
        let root_node = arena.new_node_with(|id| {
            Element::from_widget(Location::caller().into(), WindowContainer, ElementId(id))
        });

        Window {
            id,
            size: Size::ZERO,
            size_policy: pending.size_policy,
            handle,
            app: pending.root,
            menu: pending.menu,
            root_element: ElementId(root_node),
            invalid: Region::EMPTY,
            ext_handle,
            arena,
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
        let Self {
            handle,
            ext_handle,
            root_element,
            size,
            arena,
            ..
        } = self;

        if root_element.needs_layout(arena) {
            // debug!("layout");
            self.layout(queue);
        }

        let mut global_state = GlobalState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),

            text: handle.text(),
        };

        let mut root_state = UiState::new(*root_element, Some(size.clone()));
        let mut paint_ctx = PaintCtx {
            global_state: &mut global_state,
            arena: &mut arena,
            ui_state: &mut root_state,
            region: invalid.clone(),
            render_ctx: piet,
        };

        root_element.paint(&mut paint_ctx);
        draw_fps(self.fps_counter.tick(), handle.get_size(), &mut paint_ctx);
    }

    // #[instrument(skip(self))]
    pub(crate) fn layout(&mut self, _command_queue: &mut CommandQueue) {
        let Self {
            handle,
            ext_handle,
            root_element: root_node,
            arena,
            size,
            invalid,
            ..
        } = self;

        let mut global_state = GlobalState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };
        let mut root_state = UiState::new(*root_node, Some(size.clone()));

        let mut layout_ctx = LayoutCtx {
            global_state: &mut global_state,
            arena: &mut arena,

            ui_state: &mut root_state,
        };

        let root_render_object = node_to_object_mut::<RenderObject>(arena, *root_node)
            .expect("WindowContainer not exist");

        root_state.size = root_render_object.layout(
            &mut layout_ctx,
            &(BoxConstraints::new(Size::ZERO, self.size).into()),
        );
        invalid.union_with(&root_state.invalid);
    }

    // #[instrument(skip(self))]
    pub(crate) fn event(&mut self, queue: &mut CommandQueue, event: Event) -> Handled {
        // debug!("event");
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
            root_element: root_node,
            arena,
            size,
            invalid,
            ..
        } = self;

        let mut context_state = GlobalState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };
        let mut root_state = UiState::new(*root_node, Some(size.clone()));

        let mut event_ctx = EventCtx {
            global_state: &mut context_state,
            arena: &mut arena,
            ui_state: &mut root_state,
            is_active: false,
            is_handled: false,
        };

        let root_render_object = node_to_object_mut::<RenderObject>(arena, *root_node)
            .expect("WindowContainer not exist");
        root_render_object.event(&mut event_ctx, &event);

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

    pub(crate) fn lifecycle(&mut self, _queue: &mut CommandQueue, event: &LifeCycle) {
        let Self {
            handle,
            ext_handle,
            root_element: root_node,
            arena,
            size,
            invalid,
            ..
        } = self;

        let mut context_state = GlobalState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };
        let mut root_state = UiState::new(*root_node, Some(size.clone()));

        let mut ctx = LifeCycleCtx {
            ui_state: &mut root_state,
            arena: &mut arena,
            global_state: &mut context_state,
        };

        let root_render_object = node_to_object_mut::<RenderObject>(arena, *root_node)
            .expect("WindowContainer not exist");
        root_render_object.lifecycle(&mut ctx, event);
        invalid.union_with(&root_state.invalid);
    }

    pub(crate) fn update(&mut self, _queue: &mut CommandQueue) {
        // debug!("update");
        let Self {
            handle,
            ext_handle,
            root_element: root_node,
            arena,
            app,
            ..
        } = self;

        let mut global_state = GlobalState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
        };

        let mut cx = Ui::new(*root_node, &mut global_state, arena);
        app(&mut cx);

        let root_render_object = node_to_object_mut::<RenderObject>(arena, *root_node)
            .expect("WindowContainer not exist");
        // merge children state up to root

        for node in root_node.children(arena) {
            if let Some(render_node) = node_to_object_mut::<RenderObject>(arena, node) {
                root_render_object.state.merge_up(&mut render_node.state)
            }
        }

        // println!("{:#?}", root.debug_state());
    }

    pub(crate) fn invalidate_and_finalize(&mut self) {
        let Self {
            handle,
            root_element: root_node,
            arena,
            invalid,
            ..
        } = self;

        let root_render_object = node_to_object_mut::<RenderObject>(arena, *root_node)
            .expect("WindowContainer not exist");

        if root_render_object.needs_layout() {
            tracing::debug!("needs layout");
            handle.invalidate();
        } else {
            let invalid_rect = invalid.bounding_box();
            handle.invalidate_rect(invalid_rect);
            if !invalid_rect.is_empty() {
                tracing::debug!("invalidate rect: {invalid_rect}");
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
