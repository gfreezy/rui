use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::{
    any::Any,
    ops::{Index, IndexMut},
};

use druid_shell::kurbo::{Affine, Insets, Point, Rect, Shape, Size};
use druid_shell::piet::{Color, LineJoin, PaintBrush, RenderContext, StrokeStyle};
use druid_shell::TimerToken;

use crate::box_constraints::BoxConstraints;
use crate::context::{ContextState, EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx};
use crate::event::Event;
use crate::id::ChildId;
use crate::key::Caller;
use crate::lifecycle::LifeCycle;
use crate::object::AnyRenderObject;
use crate::ui::Ui;

#[derive(Default)]
pub struct Children {
    pub(crate) states: Vec<StateNode>,
    pub(crate) renders: Vec<Child>,
    pub(crate) tracked_states: Vec<String>,
}

static COUNTER: AtomicI64 = AtomicI64::new(0);

struct Counter<T> {
    val: T,
}

impl<T> Counter<T> {
    pub fn new(v: T) -> Self {
        println!(
            "new state counter: {}",
            COUNTER.fetch_add(1, Ordering::SeqCst) + 1
        );
        Counter { val: v }
    }
}

impl<T> Drop for Counter<T> {
    fn drop(&mut self) {
        println!(
            "remove state counter: {}",
            COUNTER.fetch_sub(1, Ordering::SeqCst) - 1
        );
    }
}

pub struct State<T> {
    val: Rc<Counter<RefCell<T>>>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State {
            val: self.val.clone(),
        }
    }
}

impl<T: Debug> Debug for State<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = (*self.val).val.borrow();
        (*v).fmt(f)
    }
}

impl<T> State<T> {
    pub fn new(val: T) -> Self {
        State {
            val: Rc::new(Counter::new(RefCell::new(val))),
        }
    }

    pub fn set(&self, val: T) -> T {
        self.val.val.replace(val)
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut refmut = self.val.val.borrow_mut();
        f(&mut *refmut);
    }

    pub fn get(&self) -> Ref<'_, T> {
        (*self.val).val.borrow()
    }
}

pub struct StateNode {
    pub(crate) key: Caller,
    pub(crate) state: Box<dyn Any>,
    pub(crate) dead: bool,
}

pub struct Child {
    pub(crate) key: Caller,
    pub(crate) object: Box<dyn AnyRenderObject>,
    pub(crate) children: Children,
    pub(crate) state: ChildState,
    pub(crate) dead: bool,
}

pub struct ChildState {
    pub(crate) id: ChildId,

    /// The size of the child; this is the value returned by the child's layout
    /// method.
    pub(crate) size: Size,

    /// The origin of the child in the parent's coordinate space; together with
    /// `size` these constitute the child's layout rect.
    pub(crate) origin: Point,

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

    // TODO: consider using bitflags for the booleans.
    // hover state
    pub(crate) is_hot: bool,

    // mouse down with left key
    pub(crate) is_active: bool,

    /// Any descendant is active.
    pub(crate) has_active: bool,

    pub(crate) needs_layout: bool,
}

impl Children {
    pub(crate) fn new() -> Self {
        Children::default()
    }

    pub(crate) fn track_state(&mut self, state: String) {
        self.tracked_states.push(state);
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

    pub fn get(&self, index: usize) -> Option<&Child> {
        self.renders.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Child> {
        self.renders.get_mut(index)
    }

    pub fn iter(&mut self) -> ChildIter {
        self.into_iter()
    }
}

impl Index<usize> for Children {
    type Output = Child;

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
impl Child {}

/// Public API for child nodes.
impl Child {
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
    type Item = &'a mut Child;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.children.renders.get(self.index - 1).map(|node| {
            let node_p = node as *const Child as *mut Child;
            // This is save because each child can only be accessed once.
            unsafe { &mut *node_p }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.children.len(), Some(self.children.len()))
    }
}

impl<'a> IntoIterator for &'a mut Children {
    type Item = &'a mut Child;
    type IntoIter = ChildIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChildIter {
            children: self,
            index: 0,
        }
    }
}

impl ChildState {
    pub(crate) fn new(id: ChildId, size: Option<Size>) -> Self {
        ChildState {
            id,
            origin: Point::ORIGIN,
            size: size.unwrap_or_default(),
            baseline_offset: 0.,
            is_hot: false,
            is_active: false,
            has_active: false,
            needs_layout: true,
            paint_insets: Insets::ZERO,
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
    pub fn merge_up(&mut self, child_state: &mut ChildState) {
        self.needs_layout |= child_state.needs_layout;
        self.has_active |= child_state.has_active;
    }
}

/// [`RenderObject`] API for `Child` nodes.
impl Child {
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
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
                Child::set_hot_state(
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
                Child::set_hot_state(
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
                let hot_changed = Child::set_hot_state(
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
                Child::set_hot_state(
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
        ctx.child_state.merge_up(&mut self.state)
    }

    pub fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        children: &mut Children,
    ) {
        let mut child_ctx = LifeCycleCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        self.object.lifecycle(&mut child_ctx, event, children);
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "layout", ?bc, object_name);
        let _h = span.enter();

        if !self.state.needs_layout {
            return self.state.size;
        }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        let new_size = self.object.layout(&mut child_ctx, bc, &mut self.children);

        self.state.size = new_size;

        new_size
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_save(|ctx| {
            let layout_origin = self.layout_rect().origin().to_vec2();
            ctx.transform(Affine::translate(layout_origin));
            let mut visible = ctx.region().clone();
            visible.intersect_with(self.state.paint_rect());
            visible -= layout_origin;
            ctx.with_child_ctx(visible, |ctx| self.paint_raw(ctx));
        });
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
    pub fn paint_raw(&mut self, ctx: &mut PaintCtx) {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "paint_raw", object_name);
        let _h = span.enter();

        let mut inner_ctx = PaintCtx {
            render_ctx: ctx.render_ctx,
            context_state: ctx.context_state,
            region: ctx.region.clone(),
            child_state: &self.state,
        };
        self.object.paint(&mut inner_ctx, &mut self.children);

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
}

/// Public API for child nodes.
impl Child {
    pub fn size(&self) -> Size {
        self.state.size()
    }

    /// Determines if the provided `mouse_pos` is inside `rect`
    /// and if so updates the hot state and sends `LifeCycle::HotChanged`.
    ///
    /// Returns `true` if the hot state changed.
    ///
    /// The provided `child_state` should be merged up if this returns `true`.
    fn set_hot_state(
        child: &mut dyn AnyRenderObject,
        child_state: &mut ChildState,
        children: &mut Children,
        context_state: &mut ContextState,
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
}
