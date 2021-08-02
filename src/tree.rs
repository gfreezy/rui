use std::{
    any::Any,
    ops::{Index, IndexMut},
};

use druid_shell::kurbo::{Affine, Insets, Point, Rect, Size};
use druid_shell::piet::{Brush, Color, LineJoin, PaintBrush, RenderContext, StrokeStyle};
use druid_shell::TimerToken;

use crate::box_constraints::BoxConstraints;
use crate::context::{EventCtx, LayoutCtx, LifeCycleCtx, PaintCtx};
use crate::event::Event;
use crate::id::ChildId;
use crate::key::Caller;
use crate::lifecycle::LifeCycle;
use crate::object::AnyRenderObject;

#[derive(Default)]
pub struct Children {
    pub(crate) states: Vec<State>,
    pub(crate) renders: Vec<Child>,
}

pub struct State {
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

    pub(crate) needs_layout: bool,
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
    pub fn set_origin(&mut self, ctx: &mut LayoutCtx, origin: Point) {
        self.state.origin = origin;
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
            needs_layout: true,
            paint_insets: Insets::ZERO,
        }
    }

    pub(crate) fn add_timer(&mut self, timer_token: TimerToken) {}

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
    fn merge_up(&mut self, child_state: &mut ChildState) {
        self.needs_layout |= child_state.needs_layout;
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
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                modified_event = Some(Event::MouseDown(mouse_event));
                true
            }
            Event::MouseUp(mouse_event) => {
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                modified_event = Some(Event::MouseUp(mouse_event));
                true
            }
            Event::MouseMove(mouse_event) => {
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                modified_event = Some(Event::MouseMove(mouse_event));
                true
            }
            Event::Wheel(mouse_event) => {
                let mut mouse_event = mouse_event.clone();
                mouse_event.pos -= rect.origin().to_vec2();
                modified_event = Some(Event::Wheel(mouse_event));
                true
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
                is_handled: false,
            };
            let inner_event = modified_event.as_ref().unwrap_or(event);

            self.object
                .event(&mut inner_ctx, inner_event, &mut self.children);
            ctx.is_handled |= inner_ctx.is_handled;
        }
        ctx.child_state.merge_up(&mut self.state)
    }

    pub fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle) {
        let mut child_ctx = LifeCycleCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        self.object.lifecycle(&mut child_ctx, event);
    }

    pub fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints) -> Size {
        let object_name = self.object.name();
        let span = tracing::span!(tracing::Level::DEBUG, "layout", ?bc, object_name);
        let _h = span.enter();

        if !self.state.needs_layout {
            tracing::debug!("skip child layout");
            return self.state.size;
        }

        self.state.needs_layout = false;

        let mut child_ctx = LayoutCtx {
            context_state: ctx.context_state,
            child_state: &mut self.state,
        };

        let new_size = self.object.layout(&mut child_ctx, bc, &mut self.children);

        self.state.size = new_size;

        tracing::debug!(?new_size, ?self.state.origin);
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
