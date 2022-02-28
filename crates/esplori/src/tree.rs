use std::any::type_name;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

use std::any::Any;

use druid_shell::kurbo::{Affine, Insets, Point, Rect, Shape, Size, Vec2};
use druid_shell::piet::RenderContext;
use druid_shell::{Region, TimerToken};
use generational_indextree::{Arena, NodeId};

use crate::constraints::Constraints;
use crate::context::{EventCtx, GlobalState, LayoutCtx, LifeCycleCtx, PaintCtx};
use crate::debug_state::DebugState;
use crate::event::Event;
use crate::id::ElementId;
use crate::key::Caller;
use crate::lifecycle::LifeCycle;
use crate::object::{AnyRenderObject, RenderInterface};
use crate::ui::node_to_object_mut;

pub type Children<'a> = Vec<&'a mut RenderObject>;

pub struct StateObject<T> {
    pub(crate) object: T,
}

impl<T: 'static> StateObject<T> {
    pub fn new(object: T) -> Self {
        StateObject { object }
    }
}

pub struct RenderObject {
    pub(crate) name: &'static str,
    pub(crate) key: Caller,
    pub(crate) object: Box<dyn AnyRenderObject + 'static>,
    pub(crate) state: UiState,
}

/// [`RenderObject`] API for `Child` nodes.
impl RenderObject {
    pub(crate) fn new<T: RenderInterface + 'static>(key: Caller, object: T, id: ElementId) -> Self {
        RenderObject {
            name: type_name::<T>(),
            key,
            object: Box::new(object),
            state: UiState::new(id, None),
        }
    }

    // #[doc(hidden)]
    // /// From the current data, get a best-effort description of the state of
    // /// this widget and its children for debugging purposes.
    // pub(crate) fn debug_state(&mut self) -> DebugState {
    //     let children = children(self.state.id).iter().map(|c| c.debug_state()).collect();
    //     let mut map = HashMap::new();
    //     map.insert("key".to_string(), format!("{:?}", self.key));
    //     map.insert("id".to_string(), format!("{:?}", self.state.id));
    //     map.insert("origin".to_string(), format!("{:?}", self.state.origin));
    //     map.insert("size".to_string(), format!("{:?}", self.state.size));
    //     DebugState {
    //         display_name: self.name().to_string(),
    //         children,
    //         other_values: map,
    //         ..Default::default()
    //     }
    // }
}

/// Public API for child nodes.
impl RenderObject {
    pub fn as_any(&mut self) -> &mut dyn Any {
        self.object.as_any()
    }

    /// Set the origin of this widget, in the parent's coordinate space.
    ///
    /// A container widget should call the [`Widget::layout`] method on its children in
    /// its own [`Widget::layout`] implementation, and then call `set_origin` to
    /// position those children.
    ///
    /// The child will receive the [`LifeCycle::Size`] event informing them of the final [`Size`].
    ///
    /// [`Widget::layout`]: trait.Widget.html#tymethod.layout
    /// [`Rect`]: struct.Rect.html
    /// [`Size`]: struct.Size.html
    /// [`LifeCycle::Size`]: enum.LifeCycle.html#variant.Size
    pub fn set_origin(&mut self, _ctx: &mut LayoutCtx, origin: Point) {
        self.state.origin = origin;
    }

    pub fn origin(&self) -> Point {
        self.state.origin
    }

    /// Returns the layout [`Rect`].
    ///
    /// This will be a [`Rect`] with a [`Size`] determined by the child's [`layout`]
    /// method, and the origin that was set by [`set_origin`].
    ///
    /// [`Rect`]: struct.Rect.html
    /// [`Size`]: struct.Size.html
    /// [`layout`]: trait.Widget.html#tymethod.layout
    /// [`set_origin`]: WidgetPod::set_origin
    pub fn layout_rect(&self) -> Rect {
        self.state.layout_rect()
    }

    /// Set the viewport offset.
    ///
    /// This is relevant only for children of a scroll view (or similar). It must
    /// be set by the parent widget whenever it modifies the position of its child
    /// while painting it and propagating events. As a rule of thumb, you need this
    /// if and only if you `Affine::translate` the paint context before painting
    /// your child. For an example, see the implentation of [`Scroll`].
    ///
    /// [`Scroll`]: widget/struct.Scroll.html
    pub fn set_viewport_offset(&mut self, offset: Vec2) {
        if offset != self.state.viewport_offset {
            // We need the parent_window_origin recalculated.
            // It should be possible to just trigger the InternalLifeCycle::ParentWindowOrigin here,
            // instead of full layout. Would need more management in WidgetState.
            self.state.needs_layout = true;
        }
        self.state.viewport_offset = offset;
    }

    /// The viewport offset.
    ///
    /// This will be the same value as set by [`set_viewport_offset`].
    ///
    /// [`set_viewport_offset`]: #method.viewport_offset
    pub fn viewport_offset(&self) -> Vec2 {
        self.state.viewport_offset
    }

    pub fn request_update(&mut self) {
        self.state.request_update = true;
    }

    // #[track_caller]
    pub fn request_layout(&mut self) {
        // let caller = Location::caller();
        // debug!("{} request layout: {caller:?}", self.name());
        self.state.needs_layout = true;
    }

    /// Returns `true` if any descendant is active.
    pub fn has_active(&self) -> bool {
        self.state.has_active
    }

    /// Query the "active" state of the widget.
    pub fn is_active(&self) -> bool {
        self.state.is_active
    }

    /// Query the "hot" state of the widget.
    ///
    /// See [`EventCtx::is_hot`](struct.EventCtx.html#method.is_hot) for
    /// additional information.
    pub fn is_hot(&self) -> bool {
        self.state.is_hot
    }
}

pub struct Element {
    pub(crate) key: Caller,
    pub(crate) element_object: Box<dyn Any>,
}

impl Element {
    pub fn from_widget<T: RenderInterface + 'static>(
        key: Caller,
        widget: T,
        id: ElementId,
    ) -> Self {
        Element {
            key,
            element_object: Box::new(RenderObject::new(key, widget, id)),
        }
    }

    pub fn from_state<T: 'static>(key: Caller, widget: T) -> Self {
        Element {
            key,
            element_object: Box::new(StateObject { object: widget }),
        }
    }
    pub fn key(&self) -> Caller {
        self.key
    }

    pub fn state_object<T: 'static>(&self) -> Option<&StateObject<T>> {
        self.element_object.downcast_ref::<StateObject<T>>()
    }

    pub fn render_object<T: RenderInterface + 'static>(&self) -> Option<&RenderObject> {
        self.element_object.downcast_ref::<RenderObject>()
    }

    pub fn object<T: 'static>(&self) -> Option<&T> {
        self.element_object.downcast_ref::<T>()
    }

    pub fn object_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.element_object.downcast_mut::<T>()
    }
}

pub struct UiState {
    pub(crate) id: ElementId,

    /// The size of the child; this is the value returned by the child's layout
    /// method.
    pub(crate) size: Size,

    /// The origin of the child in the parent's coordinate space; together with
    /// `size` these constitute the child's layout rect.
    pub(crate) origin: Point,

    /// The origin of the parent in the window coordinate space;
    pub(crate) parent_window_origin: Point,

    /// The origin of the parent in the window coordinate space;
    pub(crate) parent_data: Option<Box<dyn Any>>,

    /// The insets applied to the layout rect to generate the paint rect.
    /// In general, these will be zero; the exception is for things like
    /// drop shadows or overflowing text.
    pub(crate) paint_insets: Insets,

    /// The offset of the baseline relative to the bottom of the widget.
    ///
    /// In general, this will be zero; the bottom of the widget will be considered
    /// the baseline. Widgets that contain text or controls that expect to be
    /// laid out alongside text can set this as appropriate.
    pub(crate) baseline_offset: f64,

    // The region that needs to be repainted, relative to the widget's bounds.
    pub(crate) invalid: Region,

    // The part of this widget that is visible on the screen is offset by this
    // much. This will be non-zero for widgets that are children of `Scroll`, or
    // similar, and it is used for propagating invalid regions.
    pub(crate) viewport_offset: Vec2,

    // TODO: consider using bitflags for the booleans.
    // hover state
    pub(crate) is_hot: bool,

    // mouse down with left key
    pub(crate) is_active: bool,

    /// Any descendant is active.
    pub(crate) has_active: bool,

    pub(crate) needs_layout: bool,

    /// Any descendant has requested update.
    pub(crate) request_update: bool,
}

impl UiState {
    pub(crate) fn new(id: ElementId, size: Option<Size>) -> Self {
        UiState {
            id,
            origin: Point::ORIGIN,
            size: size.unwrap_or_default(),
            baseline_offset: 0.,
            invalid: Region::EMPTY,
            viewport_offset: Vec2::ZERO,
            parent_data: None,
            is_hot: false,
            is_active: false,
            has_active: false,
            needs_layout: true,
            paint_insets: Insets::ZERO,
            parent_window_origin: Point::ORIGIN,
            request_update: false,
        }
    }

    pub(crate) fn add_timer(&mut self, _timer_token: TimerToken) {}

    #[inline]
    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn layout_rect(&self) -> Rect {
        Rect::from_origin_size(self.origin, self.size)
    }

    /// The paint region for this widget.
    ///
    /// For more information, see [`WidgetPod::paint_rect`].
    ///
    /// [`WidgetPod::paint_rect`]: struct.WidgetPod.html#method.paint_rect
    pub(crate) fn paint_rect(&self) -> Rect {
        self.layout_rect() + self.paint_insets
    }

    /// Update to incorporate state changes from a child.
    ///
    /// This will also clear some requests in the child state.
    ///
    /// This method is idempotent and can be called multiple times.
    pub fn merge_up(&mut self, child_state: &mut UiState) {
        let clip = self
            .layout_rect()
            .with_origin(Point::ORIGIN)
            .inset(self.paint_insets);
        let offset = child_state.layout_rect().origin().to_vec2() - child_state.viewport_offset;
        for &r in child_state.invalid.rects() {
            let r = (r + offset).intersect(clip);
            if r.area() != 0.0 {
                self.invalid.add_rect(r);
            }
        }

        if !child_state.invalid.is_empty() {
            tracing::debug!(
                "merge up: child invalid: {:?}, parent invalid: {:?}, clip: {:?}",
                child_state.invalid,
                self.invalid,
                clip
            );
        }
        // Clearing the invalid rects here is less fragile than doing it while painting. The
        // problem is that widgets (for example, Either) might choose not to paint certain
        // invisible children, and we shouldn't allow these invisible children to accumulate
        // invalid rects.
        child_state.invalid.clear();

        self.needs_layout |= child_state.needs_layout;
        self.has_active |= child_state.has_active;
        self.request_update |= child_state.request_update;
    }

    pub(crate) fn window_origin(&self) -> Point {
        self.parent_window_origin + self.origin.to_vec2() - self.viewport_offset
    }

    pub(crate) fn parent_data<T: 'static>(&self) -> Option<&T> {
        self.parent_data
            .as_ref()
            .map(|v| v.downcast_ref())
            .flatten()
    }

    pub(crate) fn parent_data_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.parent_data
            .as_mut()
            .map(|v| v.downcast_mut())
            .flatten()
    }

    pub(crate) fn set_parent_data(&mut self, parent_data: Option<Box<dyn Any>>) {
        self.parent_data = parent_data;
    }
}

/// [`RenderObject`] API for `Child` nodes.
impl RenderObject {
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "event", object_name);
        let _h = span.enter();

        if ctx.is_handled {
            // This function is called by containers to propagate an event from
            // containers to children. Non-recurse events will be invoked directly
            // from other points in the library.
            return;
        }
        let had_active = self.state.has_active;
        let rect = self.layout_rect();

        // If we need to replace either the event or its data.
        let mut modified_event = None;

        let recurse = match event {
            Event::WindowConnected => true,
            Event::WindowSize(_) => {
                self.state.needs_layout = true;
                true
            }
            Event::MouseDown(mouse_event) => {
                RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseDown(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::MouseUp(mouse_event) => {
                RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseUp(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::MouseMove(mouse_event) => {
                let hot_changed = RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() || hot_changed {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseMove(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::Wheel(mouse_event) => {
                RenderObject::set_hot_state(
                    &mut *self.object,
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );

                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::Wheel(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::AnimFrame(_) => false,
            Event::KeyDown(_) => true,
            Event::KeyUp(_) => true,
            Event::Paste(_) => true,
            Event::Zoom(_) => true,
            Event::Timer(_) => false, // This event was targeted only to our parent
            _ => true,
        };

        if recurse {
            let mut inner_ctx = EventCtx {
                global_state: ctx.global_state,
                ui_state: &mut self.state,
                arena: &mut ctx.arena,
                is_active: false,
                is_handled: false,
            };
            let inner_event = modified_event.as_ref().unwrap_or(event);
            inner_ctx.ui_state.has_active = false;

            self.object.event(&mut inner_ctx, inner_event);

            inner_ctx.ui_state.has_active |= inner_ctx.ui_state.is_active;
            ctx.is_handled |= inner_ctx.is_handled;
        }
        ctx.ui_state.merge_up(&mut self.state);
    }

    pub fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        let mut child_ctx = LifeCycleCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        self.object.lifecycle(&mut child_ctx, event);
    }

    pub fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "dry_layout", ?c, object_name);
        let _h = span.enter();

        let mut child_ctx = LayoutCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        self.object.dry_layout(&mut child_ctx, c)
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "layout", ?c, object_name);
        let _h = span.enter();

        // if !self.state.needs_layout {
        //     return self.state.size;
        // }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        let new_size = self.object.layout(&mut child_ctx, c);

        self.state.size = new_size;

        new_size
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "paint", object_name);
        let _h = span.enter();

        ctx.with_save(|ctx| {
            let layout_origin = self.layout_rect().origin().to_vec2();
            ctx.transform(Affine::translate(layout_origin));
            let mut visible = ctx.region().clone();
            visible.intersect_with(self.state.paint_rect());
            visible -= layout_origin;
            ctx.with_child_ctx(visible, |ctx| self.paint_raw(ctx));
        });
    }
}

/// Public API for child nodes.
impl RenderObject {
    pub fn size(&self) -> Size {
        self.state.size()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
    /// Determines if the provided `mouse_pos` is inside `rect`
    /// and if so updates the hot state and sends `LifeCycle::HotChanged`.
    ///
    /// Returns `true` if the hot state changed.
    ///
    /// The provided `child_state` should be merged up if this returns `true`.
    fn set_hot_state(
        child: &mut (dyn AnyRenderObject + 'static),
        child_state: &mut UiState,
        ctx: &mut EventCtx,
        rect: Rect,
        mouse_pos: Option<Point>,
    ) -> bool {
        let had_hot = child_state.is_hot;
        child_state.is_hot = match mouse_pos {
            Some(pos) => rect.winding(pos) != 0,
            None => false,
        };
        if had_hot != child_state.is_hot {
            let hot_changed_event = LifeCycle::HotChanged(child_state.is_hot);
            let mut child_ctx = LifeCycleCtx {
                global_state: ctx.global_state,
                arena: ctx.arena,
                ui_state: child_state,
            };
            child.lifecycle(&mut child_ctx, &hot_changed_event);
            // if hot changes and we're showing widget ids, always repaint
            // if env.get(Env::DEBUG_WIDGET_ID) {
            //     child_ctx.request_paint();
            // }
            return true;
        }
        false
    }

    /// Paint a child widget.
    ///
    /// Generally called by container widgets as part of their [`Widget::paint`]
    /// method.
    ///
    /// Note that this method does not apply the offset of the layout rect.
    /// If that is desired, use [`paint`] instead.
    ///
    /// [`layout`]: trait.Widget.html#tymethod.layout
    /// [`Widget::paint`]: trait.Widget.html#tymethod.paint
    /// [`paint`]: #method.paint
    pub(crate) fn paint_raw(&mut self, ctx: &mut PaintCtx) {
        let mut inner_ctx = PaintCtx {
            render_ctx: ctx.render_ctx,
            arena: ctx.arena,
            global_state: ctx.global_state,
            region: ctx.region.clone(),
            ui_state: &mut self.state,
        };
        self.object.paint(&mut inner_ctx);

        // debug!("layout rect: {:?}", self.layout_rect());
        // let _rect = inner_ctx.size().to_rect();

        // const STYLE: StrokeStyle = StrokeStyle::new()
        //     .dash_pattern(&[4.0, 2.0])
        //     .dash_offset(8.0)
        //     .line_join(LineJoin::Round);
        // inner_ctx.render_ctx.stroke_styled(
        //     rect,
        //     &PaintBrush::Color(Color::rgb8(0, 0, 0)),
        //     1.,
        //     &STYLE,
        // );
    }

    pub(crate) fn needs_update(&self) -> bool {
        self.state.request_update
    }

    pub(crate) fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    pub(crate) fn set_parent_data(&mut self, parent_data: Option<Box<dyn Any>>) {
        self.state.set_parent_data(parent_data)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]
pub struct ElementId(pub(crate) NodeId);

impl ElementId {
    pub(crate) fn render_object<'a>(
        &self,
        arena: &'a mut Arena<Element>,
    ) -> Option<&'a mut RenderObject> {
        node_to_object_mut::<RenderObject>(arena, self.0)
    }

    pub(crate) fn needs_layout(&self, arena: &mut Arena<Element>) -> bool {
        self.render_object(arena).unwrap().state.needs_layout
    }
}

/// [`RenderObject`] API for `ElementId` nodes.
impl ElementId {
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "event", object_name);
        let _h = span.enter();

        if ctx.is_handled {
            // This function is called by containers to propagate an event from
            // containers to children. Non-recurse events will be invoked directly
            // from other points in the library.
            return;
        }
        let had_active = self.state.has_active;
        let rect = self.layout_rect();

        // If we need to replace either the event or its data.
        let mut modified_event = None;

        let recurse = match event {
            Event::WindowConnected => true,
            Event::WindowSize(_) => {
                self.state.needs_layout = true;
                true
            }
            Event::MouseDown(mouse_event) => {
                RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseDown(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::MouseUp(mouse_event) => {
                RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseUp(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::MouseMove(mouse_event) => {
                let hot_changed = RenderObject::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );
                if had_active || self.is_hot() || hot_changed {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::MouseMove(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::Wheel(mouse_event) => {
                RenderObject::set_hot_state(
                    &mut *self.object,
                    &mut self.state,
                    ctx,
                    rect,
                    Some(mouse_event.pos),
                );

                if had_active || self.is_hot() {
                    let mut mouse_event = mouse_event.clone();
                    mouse_event.pos -= rect.origin().to_vec2();
                    modified_event = Some(Event::Wheel(mouse_event));
                    true
                } else {
                    false
                }
            }
            Event::AnimFrame(_) => false,
            Event::KeyDown(_) => true,
            Event::KeyUp(_) => true,
            Event::Paste(_) => true,
            Event::Zoom(_) => true,
            Event::Timer(_) => false, // This event was targeted only to our parent
            _ => true,
        };

        if recurse {
            let mut inner_ctx = EventCtx {
                global_state: ctx.global_state,
                ui_state: &mut self.state,
                arena: &mut ctx.arena,
                is_active: false,
                is_handled: false,
            };
            let inner_event = modified_event.as_ref().unwrap_or(event);
            inner_ctx.ui_state.has_active = false;

            self.object.event(&mut inner_ctx, inner_event);

            inner_ctx.ui_state.has_active |= inner_ctx.ui_state.is_active;
            ctx.is_handled |= inner_ctx.is_handled;
        }
        ctx.ui_state.merge_up(&mut self.state);
    }

    pub fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        let mut child_ctx = LifeCycleCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        self.object.lifecycle(&mut child_ctx, event);
    }

    pub fn dry_layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "dry_layout", ?c, object_name);
        let _h = span.enter();

        let mut child_ctx = LayoutCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        self.object.dry_layout(&mut child_ctx, c)
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, c: &Constraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "layout", ?c, object_name);
        let _h = span.enter();

        // if !self.state.needs_layout {
        //     return self.state.size;
        // }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            global_state: ctx.global_state,
            arena: ctx.arena,
            ui_state: &mut self.state,
        };

        let new_size = self.object.layout(&mut child_ctx, c);

        self.state.size = new_size;

        new_size
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "paint", object_name);
        let _h = span.enter();

        ctx.with_save(|ctx| {
            let layout_origin = self.layout_rect().origin().to_vec2();
            ctx.transform(Affine::translate(layout_origin));
            let mut visible = ctx.region().clone();
            visible.intersect_with(self.state.paint_rect());
            visible -= layout_origin;
            ctx.with_child_ctx(visible, |ctx| self.paint_raw(ctx));
        });
    }
}
