use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use druid_shell::kurbo::{Affine, Insets, Point, Size};
use druid_shell::piet::{Piet, PietText, RenderContext};
use druid_shell::{Region, TimerToken, WindowHandle};

use crate::id::ChildId;
use crate::tree::ChildState;

/// A macro for implementing methods on multiple contexts.
///
/// There are a lot of methods defined on multiple contexts; this lets us only
/// have to write them out once.
macro_rules! impl_context_method {
    ($ty:ty,  { $($method:item)+ } ) => {
        impl $ty { $($method)+ }
    };
    ( $ty:ty, $($more:ty),+, { $($method:item)+ } ) => {
        impl_context_method!($ty, { $($method)+ });
        impl_context_method!($($more),+, { $($method)+ });
    };
}

/// Static state that is shared between most contexts.
pub(crate) struct ContextState<'a> {
    pub(crate) window: &'a WindowHandle,
    pub(crate) text: PietText,
}

pub struct UpdateCtx<'a, 'b> {
    pub(crate) context_state: &'a mut ContextState<'b>,
    pub(crate) child_state: &'a mut ChildState,
}

pub struct EventCtx<'a, 'b> {
    pub(crate) context_state: &'a mut ContextState<'b>,
    pub(crate) child_state: &'a mut ChildState,
    pub(crate) is_handled: bool,
}

pub struct LifeCycleCtx<'a, 'b> {
    pub(crate) context_state: &'a mut ContextState<'b>,
    pub(crate) child_state: &'a mut ChildState,
}

pub struct LayoutCtx<'a, 'b> {
    pub(crate) context_state: &'a mut ContextState<'b>,
    pub(crate) child_state: &'a mut ChildState,
}

pub struct PaintCtx<'a, 'b, 'c> {
    pub(crate) context_state: &'a mut ContextState<'b>,
    pub(crate) child_state: &'a ChildState,
    /// The render context for actually painting.
    pub render_ctx: &'a mut Piet<'c>,
    /// The currently visible region.
    pub(crate) region: Region,
}

/// Z-order paint operations with transformations.
pub(crate) struct ZOrderPaintOp {
    pub z_index: u32,
    pub paint_func: Box<dyn FnOnce(&mut PaintCtx) + 'static>,
    pub transform: Affine,
}

// methods on everyone
impl_context_method!(
    EventCtx<'_, '_>,
    UpdateCtx<'_, '_>,
    LifeCycleCtx<'_, '_>,
    LayoutCtx<'_, '_>,
    PaintCtx<'_, '_, '_>,
    {
        /// get the `ChildId` of the current widget.
        pub fn child_id(&self) -> ChildId {
            self.child_state.id
        }

        /// Returns a reference to the current `WindowHandle`.
        pub fn window(&self) -> &WindowHandle {
            &self.context_state.window
        }

        /// Get an object which can create text layouts.
        pub fn text(&mut self) -> &mut PietText {
            &mut self.context_state.text
        }
    }
);

// methods on everyone but layoutctx
impl_context_method!(
    EventCtx<'_, '_>,
    UpdateCtx<'_, '_>,
    LifeCycleCtx<'_, '_>,
    PaintCtx<'_, '_, '_>,
    {
        /// The layout size.
        ///
        /// This is the layout size as ultimately determined by the parent
        /// container, on the previous layout pass.
        ///
        /// Generally it will be the same as the size returned by the child widget's
        /// [`layout`] method.
        ///
        /// [`layout`]: trait.Widget.html#tymethod.layout
        pub fn size(&self) -> Size {
            self.child_state.size()
        }
    }
);

impl_context_method!(EventCtx<'_, '_>, UpdateCtx<'_, '_>, LifeCycleCtx<'_, '_>, {
    /// Request a layout pass.
    ///
    /// A Widget's [`layout`] method is always called when the widget tree
    /// changes, or the window is resized.
    ///
    /// If your widget would like to have layout called at any other time,
    /// (such as if it would like to change the layout of children in
    /// response to some event) it must call this method.
    ///
    /// [`layout`]: trait.Widget.html#tymethod.layout
    pub fn request_layout(&mut self) {
        self.child_state.needs_layout = true;
    }
});

impl<'a> ContextState<'a> {
    fn request_timer(&self, child_state: &mut ChildState, deadline: Duration) -> TimerToken {
        let timer_token = self.window.request_timer(deadline);
        child_state.add_timer(timer_token);
        timer_token
    }
}

impl LayoutCtx<'_, '_> {
    /// Set explicit paint [`Insets`] for this widget.
    ///
    /// You are not required to set explicit paint bounds unless you need
    /// to paint outside of your layout bounds. In this case, the argument
    /// should be an [`Insets`] struct that indicates where your widget
    /// needs to overpaint, relative to its bounds.
    ///
    /// For more information, see [`WidgetPod::paint_insets`].
    ///
    /// [`Insets`]: struct.Insets.html
    /// [`WidgetPod::paint_insets`]: struct.WidgetPod.html#method.paint_insets
    pub fn set_paint_insets(&mut self, insets: impl Into<Insets>) {
        self.child_state.paint_insets = insets.into().nonnegative();
    }

    /// Set an explicit baseline position for this widget.
    ///
    /// The baseline position is used to align widgets that contain text,
    /// such as buttons, labels, and other controls. It may also be used
    /// by other widgets that are opinionated about how they are aligned
    /// relative to neighbouring text, such as switches or checkboxes.
    ///
    /// The provided value should be the distance from the *bottom* of the
    /// widget to the baseline.
    pub fn set_baseline_offset(&mut self, baseline: f64) {
        self.child_state.baseline_offset = baseline
    }
}

impl<'c> Deref for PaintCtx<'_, '_, 'c> {
    type Target = Piet<'c>;

    fn deref(&self) -> &Self::Target {
        self.render_ctx
    }
}

impl<'c> DerefMut for PaintCtx<'_, '_, 'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.render_ctx
    }
}

impl PaintCtx<'_, '_, '_> {
    /// Returns the region that needs to be repainted.
    #[inline]
    pub fn region(&self) -> &Region {
        &self.region
    }

    /// Saves the current context, executes the closures, and restores the context.
    ///
    /// This is useful if you would like to transform or clip or otherwise
    /// modify the drawing context but do not want that modification to
    /// effect other widgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use druid::{Env, PaintCtx, RenderContext, theme};
    /// # struct T;
    /// # impl T {
    /// fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
    ///     let clip_rect = ctx.size().to_rect().inset(5.0);
    ///     ctx.with_save(|ctx| {
    ///         ctx.clip(clip_rect);
    ///         ctx.stroke(clip_rect, &env.get(theme::PRIMARY_DARK), 5.0);
    ///     });
    /// }
    /// # }
    /// ```
    pub fn with_save(&mut self, f: impl FnOnce(&mut PaintCtx)) {
        if let Err(e) = self.render_ctx.save() {
            tracing::error!("Failed to save RenderContext: '{}'", e);
            return;
        }

        f(self);

        if let Err(e) = self.render_ctx.restore() {
            tracing::error!("Failed to restore RenderContext: '{}'", e);
        }
    }

    /// Creates a temporary `PaintCtx` with a new visible region, and calls
    /// the provided function with that `PaintCtx`.
    ///
    /// This is used by containers to ensure that their children have the correct
    /// visible region given their layout.
    pub fn with_child_ctx(&mut self, region: impl Into<Region>, f: impl FnOnce(&mut PaintCtx)) {
        let mut child_ctx = PaintCtx {
            child_state: self.child_state,
            context_state: self.context_state,
            render_ctx: self.render_ctx,
            region: region.into(),
        };
        f(&mut child_ctx);
    }
}
