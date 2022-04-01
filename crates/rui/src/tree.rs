use std::any::type_name;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

use std::time::Instant;
use std::{
    any::Any,
    ops::{Index, IndexMut},
};

use bumpalo::Bump;
use druid_shell::kurbo::{Affine, Insets, Point, Rect, Shape, Size, Vec2};
use druid_shell::piet::{Color, LineJoin, PaintBrush, RenderContext, StrokeStyle};
use druid_shell::{Region, TimerToken};

use crate::box_constraints::BoxConstraints;

use crate::context::{ContextState, EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx};
use crate::debug_state::DebugState;
use crate::event::Event;
use crate::id::ChildId;
use crate::key::{Key, LocalKey};
use crate::lifecycle::{InternalLifeCycle, LifeCycle};
use crate::object::{AnyParentData, AnyRenderObject};
use crate::sliver_constraints::{SliverConstraints, SliverGeometry};

#[derive(Default)]
pub struct Children {
    pub(crate) states: Vec<StateNode>,
    pub(crate) renders: Vec<Element>,
    pub(crate) bump: Bump,
}

#[derive(Clone, Copy)]
pub struct State<T: 'static> {
    pub(crate) ptr: *mut dyn Any,
    phaton: PhantomData<T>,
}

impl<T> State<T> {
    pub(crate) fn new(raw_obx: *mut dyn Any) -> State<T> {
        State {
            ptr: raw_obx,
            phaton: PhantomData,
        }
    }

    pub fn set(&self, val: T) {
        let v = unsafe { &mut *self.ptr };
        *v.downcast_mut::<T>().unwrap() = val;
    }

    pub fn update(&self, updater: impl FnOnce(&mut T)) {
        let v = unsafe { &mut *self.ptr };
        let old = v.downcast_mut::<T>().unwrap();
        updater(old);
    }
}

impl<T: 'static> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let v = unsafe { &mut *self.ptr };
        v.downcast_mut::<T>().unwrap()
    }
}

pub struct StateNode {
    pub(crate) key: Key,
    pub(crate) state: *mut dyn Any,
    pub(crate) dead: bool,
}

impl Drop for StateNode {
    fn drop(&mut self) {
        let as_mut = unsafe { &mut *self.state };
        let boxed = unsafe { bumpalo::boxed::Box::from_raw(as_mut) };
        drop(boxed);
    }
}

pub struct Element {
    pub(crate) name: &'static str,
    pub(crate) key: Key,
    pub(crate) custom_key: LocalKey,
    pub(crate) object: Box<dyn AnyRenderObject>,
    pub(crate) children: Children,
    pub(crate) state: ElementState,
    pub(crate) dead: bool,
}

pub struct ElementState {
    pub(crate) id: ChildId,

    /// The size of the box child; this is the value returned by the child's `layout_box`
    /// method.
    pub(crate) size: Size,

    /// The geometry of the sliver child; this is the value returned by the child's `layout_sliver`
    /// method.
    pub(crate) geometry: SliverGeometry,

    /// The origin of the child in the parent's coordinate space; together with
    /// `size` these constitute the child's layout rect.
    pub(crate) origin: Point,

    /// The origin of the parent in the window coordinate space;
    pub(crate) parent_window_origin: Point,

    // The part of this widget that is visible on the screen is offset by this
    // much. This will be non-zero for widgets that are children of `Scroll`, or
    // similar, and it is used for propagating invalid regions.
    pub(crate) viewport_offset: Vec2,

    /// The origin of the parent in the window coordinate space;
    pub(crate) parent_data: Option<Box<dyn AnyParentData>>,

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

    pub(crate) visible: bool,

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

impl Children {
    pub(crate) fn new() -> Self {
        Children::default()
    }
}

/// Public API for accessing children.
impl Children {
    pub fn len(&self) -> usize {
        self.renders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&Element> {
        self.renders.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Element> {
        self.renders.get_mut(index)
    }

    pub fn first(&self) -> Option<&Element> {
        self.renders.first()
    }

    pub(crate) fn first_mut(&mut self) -> Option<&mut Element> {
        self.renders.first_mut()
    }

    pub fn last(&self) -> Option<&Element> {
        self.renders.last()
    }

    pub fn iter(&mut self) -> ChildIter {
        self.into_iter()
    }

    /// Should only be used when `Element` need to change their child lists.
    /// Used by `sliver_list`.
    /// Calling this in other cases will lead to an inconsistent tree and probably cause crashes.
    pub fn remove_element(&mut self, index: usize) -> Option<Element> {
        if index >= self.renders.len() {
            return None;
        }
        let mut el = self.renders.remove(index);
        el.set_parent_data(None);
        Some(el)
    }

    pub fn insert(&mut self, index: usize, child: Element) {
        self.renders.insert(index, child);
    }
}

impl Index<usize> for Children {
    type Output = Element;

    fn index(&self, index: usize) -> &Self::Output {
        &self.renders[index]
    }
}

impl IndexMut<usize> for Children {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.renders[index]
    }
}

/// [`RenderObject`] API for `Child` nodes.
impl Element {
    pub(crate) fn new<T>(key: Key, object: T) -> Self
    where
        T: AnyRenderObject,
    {
        Element {
            name: type_name::<T>(),
            key,
            custom_key: "".to_string(),
            object: Box::new(object),
            children: Children::new(),
            state: ElementState::new(ChildId::next(), None),
            dead: false,
        }
    }

    #[doc(hidden)]
    /// From the current data, get a best-effort description of the state of
    /// this widget and its children for debugging purposes.
    pub(crate) fn debug_state(&mut self) -> DebugState {
        let children = self.children.iter().map(|c| c.debug_state()).collect();
        let mut map = HashMap::new();
        map.insert("id".to_string(), format!("{:?}", self.state.id));
        map.insert("origin".to_string(), format!("{:?}", self.state.origin));
        map.insert(
            "paint_rect".to_string(),
            format!("{:?}", self.state.paint_rect()),
        );
        map.insert(
            "window_origin".to_string(),
            format!("{:?}", self.state.window_origin()),
        );
        map.insert("size".to_string(), format!("{:?}", self.state.size));
        if !self.custom_key.is_empty() {
            map.insert("key".to_string(), self.custom_key.clone());
        }
        let custom_map = self.object.debug_state();
        map.extend(custom_map.into_iter());

        DebugState {
            display_name: self.name().to_string(),
            children,
            other_values: map,
            ..Default::default()
        }
    }
}

/// Public API for child nodes.
impl Element {
    pub fn as_any(&mut self) -> &mut dyn Any {
        self.object.as_any()
    }

    pub fn id(&self) -> ChildId {
        self.state.id
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
    pub fn set_origin(&mut self, ctx: &mut LayoutCtx, origin: Point) {
        self.state.origin = origin;
    }

    pub fn set_viewport_offset(&mut self, offset: Vec2) {
        self.state.set_viewport_offset(offset);
    }

    pub fn origin(&self) -> Point {
        self.state.origin
    }

    pub fn set_paint_insets(&mut self, insets: Insets) {
        self.state.paint_insets = insets;
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

    /// The paint region for this widget.
    ///
    /// For more information, see [`WidgetPod::paint_rect`].
    ///
    /// [`WidgetPod::paint_rect`]: struct.WidgetPod.html#method.paint_rect
    pub fn paint_rect(&self) -> Rect {
        self.state.paint_rect()
    }

    pub fn request_update(&mut self) {
        self.state.request_update = true;
    }

    #[track_caller]
    pub fn request_layout(&mut self) {
        tracing::debug!(
            "request_layout, caller: {:?}",
            std::panic::Location::caller()
        );
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

/// Allows iterating over a set of [`Children`].
pub struct ChildIter<'a> {
    children: &'a mut Children,
    index: usize,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = &'a mut Element;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.children.renders.get(self.index - 1).map(|node| {
            let node_p = node as *const Element as *mut Element;
            // This is save because each child can only be accessed once.
            unsafe { &mut *node_p }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.children.len(), Some(self.children.len()))
    }
}

impl<'a> DoubleEndedIterator for ChildIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.index -= 1;
        self.children.renders.get(self.index - 1).map(|node| {
            let node_p = node as *const Element as *mut Element;
            // This is save because each child can only be accessed once.
            unsafe { &mut *node_p }
        })
    }
}

impl<'a> IntoIterator for &'a mut Children {
    type Item = &'a mut Element;
    type IntoIter = ChildIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChildIter {
            children: self,
            index: 0,
        }
    }
}

impl ElementState {
    pub(crate) fn new(id: ChildId, size: Option<Size>) -> Self {
        ElementState {
            id,
            origin: Point::ORIGIN,
            viewport_offset: Vec2::ZERO,
            size: size.unwrap_or_default(),
            geometry: SliverGeometry::ZERO,
            baseline_offset: 0.,
            invalid: Region::EMPTY,
            parent_data: None,
            is_hot: false,
            is_active: false,
            has_active: false,
            needs_layout: true,
            paint_insets: Insets::ZERO,
            parent_window_origin: Point::ORIGIN,
            visible: true,
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
    pub fn merge_up(&mut self, child_state: &mut ElementState) {
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
            // tracing::debug!(
            //     "merge up: child invalid: {:?}, parent invalid: {:?}, clip: {:?}",
            //     child_state.invalid,
            //     self.invalid,
            //     clip
            // );
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
        if offset != self.viewport_offset {
            // We need the parent_window_origin recalculated.
            // It should be possible to just trigger the InternalLifeCycle::ParentWindowOrigin here,
            // instead of full layout. Would need more management in WidgetState.
            self.needs_layout = true;
        }
        self.viewport_offset = offset;
    }

    pub(crate) fn take_parent_data<T: 'static>(&mut self) -> Option<Box<T>> {
        self.parent_data
            .take()
            .map(|v| v.to_any_box().downcast().ok())
            .flatten()
    }

    pub(crate) fn parent_data<T: 'static>(&self) -> Option<&T> {
        self.parent_data
            .as_ref()
            .map(|v| v.as_any().downcast_ref())
            .flatten()
    }

    pub(crate) fn parent_data_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.parent_data
            .as_mut()
            .map(|v| v.as_any_mut().downcast_mut())
            .flatten()
    }

    pub(crate) fn set_parent_data(&mut self, parent_data: Option<Box<dyn AnyParentData>>) -> bool {
        let changed = match (&self.parent_data, &parent_data) {
            (None, None) => false,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (Some(l), Some(r)) => !l.eql(r.deref()),
        };
        if changed {
            tracing::debug!(
                "set parent data, old: {:?}, new: {:?}",
                self.parent_data,
                parent_data
            );
        }
        self.parent_data = parent_data;
        changed
    }
}

/// [`RenderObject`] API for `Element` nodes.
impl Element {
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        let object_name = self.object.name();
        let instant = Instant::now();
        // let span = tracing::span!(tracing::Level::DEBUG, "event", object_name);
        // let _h = span.enter();

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
                Element::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    &mut self.children,
                    ctx.context_state,
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
                Element::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    &mut self.children,
                    ctx.context_state,
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
                let hot_changed = Element::set_hot_state(
                    self.object.as_mut(),
                    &mut self.state,
                    &mut self.children,
                    ctx.context_state,
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
                Element::set_hot_state(
                    &mut *self.object,
                    &mut self.state,
                    &mut self.children,
                    ctx.context_state,
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
                context_state: ctx.context_state,
                child_state: &mut self.state,
                is_active: false,
                is_handled: false,
            };
            let inner_event = modified_event.as_ref().unwrap_or(event);
            inner_ctx.child_state.has_active = false;

            self.object
                .event(&mut inner_ctx, inner_event, &mut self.children);

            inner_ctx.child_state.has_active |= inner_ctx.child_state.is_active;
            ctx.is_handled |= inner_ctx.is_handled;
        }
        ctx.child_state.merge_up(&mut self.state);

        // tracing::debug!(
        //     "{} event took {}",
        //     object_name,
        //     instant.elapsed().as_millis()
        // );
    }

    pub fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        let object_name = self.object.name();
        let instant = Instant::now();
        match event {
            LifeCycle::Internal(internal) => match internal {
                InternalLifeCycle::ParentWindowOrigin => {
                    self.state.parent_window_origin = ctx.child_state.window_origin();
                }
            },
            _ => {}
        }

        let mut child_ctx = LifeCycleCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        self.object
            .lifecycle(&mut child_ctx, event, &mut self.children);

        // tracing::debug!(
        //     "{} lifecycle took {}",
        //     object_name,
        //     instant.elapsed().as_millis()
        // );
    }

    pub fn dry_layout_box(&mut self, ctx: &mut LayoutCtx, c: &BoxConstraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "dry_layout", ?c, object_name);
        let _h = span.enter();

        let mut child_ctx = LayoutCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        self.object
            .dry_layout(&mut child_ctx, c, &mut self.children)
    }

    pub fn layout_box(&mut self, ctx: &mut LayoutCtx, c: &BoxConstraints) -> Size {
        let object_name = self.object.name();
        let instant = Instant::now();
        // let span = tracing::span!(tracing::Level::DEBUG, "layout_box", ?c, object_name);
        // let _h = span.enter();

        // if !self.state.needs_layout {
        //     return self.state.size;
        // }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        let new_size = self
            .object
            .layout_box(&mut child_ctx, c, &mut self.children);

        // tracing::debug!(
        //     "{} layout_box took {}",
        //     object_name,
        //     instant.elapsed().as_millis()
        // );
        self.state.size = new_size;

        new_size
    }

    pub fn layout_sliver(&mut self, ctx: &mut LayoutCtx, sc: &SliverConstraints) -> SliverGeometry {
        let object_name = self.object.name();
        let instant = Instant::now();
        // let span = tracing::span!(tracing::Level::DEBUG, "layout_sliver", ?c, object_name);
        // let _h = span.enter();

        if !self.state.needs_layout {
            return self.state.geometry.clone();
        }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        let geometry = self
            .object
            .layout_sliver(&mut child_ctx, sc, &mut self.children);

        self.state.size = match sc.axis() {
            crate::style::axis::Axis::Horizontal => {
                Size::new(geometry.scroll_extent, sc.cross_axis_extent)
            }
            crate::style::axis::Axis::Vertical => {
                Size::new(sc.cross_axis_extent, geometry.scroll_extent)
            }
        };
        self.state.geometry = geometry.clone();

        geometry
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        let object_name = self.object.name();
        let instant = Instant::now();
        // let span = tracing::span!(tracing::Level::DEBUG, "paint", object_name);
        // let _h = span.enter();

        ctx.with_save(|ctx| {
            let origin = self.paint_rect().origin().to_vec2();
            ctx.transform(Affine::translate(origin));
            let mut visible = ctx.region().clone();
            visible.intersect_with(self.state.paint_rect());
            visible -= origin;
            ctx.with_child_ctx(visible, |ctx| self.paint_raw(ctx));
        });
        // tracing::debug!(
        //     "{} paint took {} us",
        //     object_name,
        //     instant.elapsed().as_micros()
        // );
    }
}

/// Public API for child nodes.
impl Element {
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
        child: &mut dyn AnyRenderObject,
        child_state: &mut ElementState,
        children: &mut Children,
        context_state: &ContextState,
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
                context_state,
                child_state,
            };
            child.lifecycle(&mut child_ctx, &hot_changed_event, children);
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
            context_state: ctx.context_state,
            region: ctx.region.clone(),
            child_state: &mut self.state,
        };
        self.object.paint(&mut inner_ctx, &mut self.children);

        // debug!("layout rect: {:?}", self.layout_rect());
        let rect = inner_ctx.size().to_rect();

        const STYLE: StrokeStyle = StrokeStyle::new()
            .dash_pattern(&[4.0, 2.0])
            .dash_offset(8.0)
            .line_join(LineJoin::Round);
        inner_ctx.render_ctx.stroke_styled(
            rect,
            &PaintBrush::Color(Color::rgb8(0, 0, 0)),
            1.,
            &STYLE,
        );
    }

    pub(crate) fn needs_update(&self) -> bool {
        self.state.request_update
    }

    pub(crate) fn needs_layout(&self) -> bool {
        self.state.needs_layout
    }

    pub(crate) fn take_parent_data<T: 'static>(&mut self) -> Option<Box<T>> {
        self.state.take_parent_data()
    }

    pub(crate) fn parent_data<T: 'static>(&self) -> Option<&T> {
        self.state.parent_data()
    }

    pub(crate) fn parent_data_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.parent_data_mut()
    }

    /// return whether parent is changed
    pub(crate) fn set_parent_data(&mut self, parent_data: Option<Box<dyn AnyParentData>>) -> bool {
        self.state.set_parent_data(parent_data)
    }

    pub(crate) fn merge_child_states_up(&mut self) {
        for child in &mut self.children {
            self.state.merge_up(&mut child.state);
        }
    }
}
