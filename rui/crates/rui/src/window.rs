use std::{
    cell::RefCell,
    collections::HashMap,
    panic::Location,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use bumpalo::Bump;
use druid_shell::{
    kurbo::{Point, Rect, Size, Vec2},
    piet::{Color, PaintBrush, Piet, RenderContext},
    MouseEvent, Region, WindowHandle,
};

use crate::{
    app::{PendingWindow, WindowSizePolicy},
    app_state::{CommandQueue, Handled},
    box_constraints::BoxConstraints,
    context::{ContextState, EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx},
    event::Event,
    ext_event::ExtEventSink,
    id::{ElementId, WindowId},
    key::{Key, EMPTY_LOCAL_KEY},
    lifecycle::{InternalLifeCycle, LifeCycle},
    menu::{MenuItemId, MenuManager},
    perf::{measure_time, FPSCounter},
    text::layout::TextLayout,
    tree::{Element, ElementState, InnerElement},
    ui::Ui,
    widgets::window_container::WindowContainer,
};

use self::hit_test::HitTestResult;

mod hit_test {
    use std::{
        cell::RefCell,
        rc::{Rc, Weak},
    };

    use crate::tree::InnerElement;

    #[derive(Clone)]
    pub(crate) struct HitTestEntry {
        target: Weak<RefCell<InnerElement>>,
    }

    impl HitTestEntry {
        pub(crate) fn target(&self) -> Option<Rc<RefCell<InnerElement>>> {
            self.target.upgrade()
        }
    }

    #[derive(Clone)]
    pub(crate) struct HitTestResult {
        path: Vec<HitTestEntry>,
    }

    impl HitTestResult {
        pub(crate) fn new() -> Self {
            HitTestResult { path: vec![] }
        }

        pub(crate) fn iter(&self) -> impl Iterator<Item = &HitTestEntry> {
            self.path.iter()
        }
    }
}

type PointerIdentifier = usize;

pub struct Window {
    id: WindowId,
    size: Size,
    size_policy: WindowSizePolicy,
    pub(crate) handle: WindowHandle,
    app: Box<dyn FnMut(&mut Ui)>,
    root: Element,
    phatom_root_id: ElementId,
    invalid: Region,
    pub(crate) menu: Option<MenuManager>,
    ext_handle: ExtEventSink,
    dirty_elements: Rc<RefCell<Vec<Weak<RefCell<InnerElement>>>>>,
    bump: Bump,
    hit_tests: HashMap<PointerIdentifier, hit_test::HitTestResult>,
}

impl Window {
    pub(crate) fn new(
        id: WindowId,
        handle: WindowHandle,
        pending: PendingWindow,
        ext_handle: ExtEventSink,
        dirty_elements: Rc<RefCell<Vec<Weak<RefCell<InnerElement>>>>>,
    ) -> Self {
        Window {
            id,
            size: Size::ZERO,
            size_policy: pending.size_policy,
            handle,
            app: pending.root,
            menu: pending.menu,
            phatom_root_id: ElementId::next(),
            root: Element::new(
                Key::current(),
                EMPTY_LOCAL_KEY.into(),
                WindowContainer::new(),
                None,
            ),
            invalid: Region::EMPTY,
            ext_handle,
            dirty_elements,
            hit_tests: HashMap::new(),
            bump: Bump::new(),
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

    /// On macos we need to update the global application menu to be the menu
    /// for the current window.
    #[cfg(target_os = "macos")]
    pub(crate) fn macos_update_app_menu(&mut self) {
        if let Some(menu) = self.menu.as_mut() {
            self.handle.set_menu(menu.refresh());
        }
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
            bump,
            dirty_elements,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),

            text: handle.text(),
            command_queue: queue,
            dirty_elements: dirty_elements.clone(),
            bump,
        };

        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));
        let mut paint_ctx = PaintCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            region: invalid.clone(),
            render_ctx: piet,
            parent: None,
        };
        root.paint(&mut paint_ctx);
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
            bump,
            dirty_elements,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
            bump,
            dirty_elements: dirty_elements.clone(),
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut layout_ctx = LayoutCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            parent: None,
        };

        root_state.size = root.layout_box(
            &mut layout_ctx,
            &BoxConstraints::new(self.size, self.size),
            false,
        );
        let mut ctx = LifeCycleCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            parent: None,
        };
        root.lifecycle(
            &mut ctx,
            &LifeCycle::Internal(InternalLifeCycle::ParentWindowOrigin),
        );
        invalid.union_with(&root_state.invalid);
    }

    fn hit_test(&mut self, hit_test_result: &mut HitTestResult, event: Point) {}

    fn handle_pointer_event(&mut self, event: &Event) {
        static POINTER_IDENTIFIER: AtomicUsize = AtomicUsize::new(0);

        let identifier = if matches!(event, Event::MouseDown(_)) {
            POINTER_IDENTIFIER.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            POINTER_IDENTIFIER.load(Ordering::Relaxed)
        };

        let hit_test_result = match event {
            Event::MouseDown(e) | Event::Wheel(e) | Event::MouseMove(e) if e.buttons.is_empty() => {
                assert!(!self.hit_tests.contains_key(&identifier));
                let mut hit_test_result = HitTestResult::new();
                self.hit_test(&mut hit_test_result, e.pos);
                if matches!(event, Event::MouseDown(_)) {
                    self.hit_tests.insert(identifier, hit_test_result.clone());
                }
                hit_test_result
            }
            Event::MouseUp(e) => self.hit_tests.remove(&identifier).unwrap(),
            Event::MouseMove(e) if e.buttons.has_left() => {
                self.hit_tests.get(&identifier).unwrap().clone()
            }
            _ => unreachable!(),
        };
        self.dispatch_event(event, hit_test_result);
    }

    fn dispatch_event(&mut self, event: &Event, hit_test_result: HitTestResult) {
        for el in hit_test_result.iter().filter_map(|p| p.target()) {
            // todo: xxx
            // let el = el.borrow_mut().event(ctx, event);
        }
    }

    // #[instrument(skip(self))]
    pub(crate) fn event(&mut self, queue: &mut CommandQueue, event: Event) -> Handled {
        match &event {
            Event::WindowSize(size) => self.size = *size,
            Event::WindowConnected => {
                self.lifecycle(queue, &LifeCycle::Other);
            }
            event @ Event::MouseDown(_)
            | event @ Event::MouseMove(_)
            | event @ Event::MouseUp(_)
            | event @ Event::Wheel(_) => {
                self.handle_pointer_event(event);
            }
            _ => (),
        }

        let Self {
            handle,
            ext_handle,
            phatom_root_id,
            size,
            invalid,
            bump,
            dirty_elements,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue: queue,
            dirty_elements: dirty_elements.clone(),
            bump,
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut event_ctx = EventCtx {
            context_state: &mut context_state,
            child_state: &mut root_state,
            is_active: false,
            is_handled: false,
            parent: None,
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
            bump,
            dirty_elements,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
            dirty_elements: dirty_elements.clone(),
            bump,
        };
        let mut root_state = ElementState::new(*phatom_root_id, Some(size.clone()));

        let mut ctx = LifeCycleCtx {
            child_state: &mut root_state,
            context_state: &mut context_state,
            parent: None,
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
            bump,
            dirty_elements,
            ..
        } = self;

        let mut context_state = ContextState {
            window: handle.clone(),
            ext_handle: ext_handle.clone(),
            text: handle.text(),
            command_queue,
            dirty_elements: dirty_elements.clone(),
            bump,
        };
        let mut inner_root = root.inner.borrow_mut();
        let mut cx = Ui::new(
            &mut inner_root.children,
            &mut context_state,
            Some(Rc::downgrade(&root.inner)),
            true,
        );
        measure_time("app::update", || {
            app(&mut cx);
        });
        cx.cleanup_tree();
        inner_root.merge_child_states_up();
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
