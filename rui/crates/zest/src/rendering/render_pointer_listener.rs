use std::default;

use crate::render_object::{
    render_box::{BoxHitTestEntry, HitTestResult, RenderBoxWidget},
    render_object::{HitTestEntry, Offset, PointerEvent, RenderObject},
};

pub type PointerEventListener = Box<dyn FnMut(&RenderObject, PointerEvent)>;

/// How to behave during hit tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HitTestBehavior {
    /// Targets that defer to their children receive events within their bounds
    /// only if one of their children is hit by the hit test.
    #[default]
    DeferToChild,

    /// Opaque targets can be hit by hit tests, causing them to both receive
    /// events within their bounds and prevent targets visually behind them from
    /// also receiving events.
    Opaque,

    /// Translucent targets both receive events within their bounds and permit
    /// targets visually behind them to also receive events.
    Translucent,
}

pub struct RenderPointerListener {
    on_pointer_down: PointerEventListener,
    on_pointer_move: PointerEventListener,
    on_pointer_up: PointerEventListener,
    on_pointer_hover: PointerEventListener,
    hit_test_behavior: HitTestBehavior,
}

impl Default for RenderPointerListener {
    fn default() -> Self {
        Self {
            on_pointer_down: Box::new(|_, _| {}),
            on_pointer_move: Box::new(|_, _| {}),
            on_pointer_up: Box::new(|_, _| {}),
            on_pointer_hover: Box::new(|_, _| {}),
            hit_test_behavior: HitTestBehavior::default(),
        }
    }
}

impl RenderPointerListener {
    pub fn new(
        on_pointer_down: Option<PointerEventListener>,
        on_pointer_move: Option<PointerEventListener>,
        on_pointer_up: Option<PointerEventListener>,
        on_pointer_hover: Option<PointerEventListener>,
        hit_test_behavior: Option<HitTestBehavior>,
    ) -> Self {
        Self {
            on_pointer_down: on_pointer_down.unwrap_or_else(|| Box::new(|_, _| {})),
            on_pointer_move: on_pointer_move.unwrap_or_else(|| Box::new(|_, _| {})),
            on_pointer_up: on_pointer_up.unwrap_or_else(|| Box::new(|_, _| {})),
            on_pointer_hover: on_pointer_hover.unwrap_or_else(|| Box::new(|_, _| {})),
            hit_test_behavior: hit_test_behavior.unwrap_or_default(),
        }
    }
}

impl RenderBoxWidget for RenderPointerListener {
    fn handle_event(&mut self, ctx: &RenderObject, event: PointerEvent, _entry: BoxHitTestEntry) {
        match event {
            PointerEvent::MouseUp(_) => (self.on_pointer_up)(ctx, event),
            PointerEvent::MouseDown(_) => (self.on_pointer_down)(ctx, event),
            PointerEvent::MouseMove(_) => (self.on_pointer_move)(ctx, event),
        }
    }

    fn hit_test(
        &mut self,
        ctx: &RenderObject,
        result: &mut HitTestResult,
        position: Offset,
    ) -> bool {
        let mut hit_target = false;
        if ctx.size().contains(position) {
            hit_target =
                self.hit_test_children(ctx, result, position) || self.hit_test_self(ctx, position);
            if hit_target || self.hit_test_behavior == HitTestBehavior::Translucent {
                result.add(HitTestEntry::new_box_hit_test_entry(ctx, position));
            }
        }
        return hit_target;
    }

    fn hit_test_self(
        &mut self,
        _ctx: &RenderObject,
        _position: crate::render_object::render_object::Offset,
    ) -> bool {
        self.hit_test_behavior == HitTestBehavior::Opaque
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
