use crate::{
    key::{Key, LocalKey},
    object::{Properties, RenderObject, RenderObjectInterface},
    sliver_constraints::SliverGeometry,
    ui::Ui,
};

use super::sliver_list::{calculate_cache_offset, calculate_paint_offset};

pub struct SliverToBox;

impl SliverToBox {
    #[track_caller]
    pub fn build(self, ui: &mut Ui, local_key: LocalKey, content: impl FnMut(&mut Ui)) {
        ui.render_object((Key::current(), local_key), self, content);
    }
}

impl Properties for SliverToBox {
    type Object = SliverToBoxObject;
}

impl RenderObject<SliverToBox> for SliverToBoxObject {
    type Action = ();

    fn create(props: SliverToBox) -> Self {
        SliverToBoxObject
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: SliverToBox) -> Self::Action {}
}

pub struct SliverToBoxObject;

impl RenderObjectInterface for SliverToBoxObject {
    fn event(
        &mut self,
        ctx: &mut crate::context::EventCtx,
        event: &crate::event::Event,
        children: &mut crate::tree::Children,
    ) {
        for child in children {
            child.event(ctx, event);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut crate::context::LifeCycleCtx,
        event: &crate::lifecycle::LifeCycle,
        children: &mut crate::tree::Children,
    ) {
        for child in children {
            child.lifecycle(ctx, event);
        }
    }

    fn layout_sliver(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        sc: &crate::sliver_constraints::SliverConstraints,
        children: &mut crate::tree::Children,
    ) -> SliverGeometry {
        if children.is_empty() {
            return SliverGeometry::ZERO;
        }
        let child_size =
            children[0].layout_box(ctx, &sc.as_box_constraints(0.0, f64::INFINITY, None));
        let child_extent = match sc.axis() {
            crate::style::axis::Axis::Horizontal => child_size.width,
            crate::style::axis::Axis::Vertical => child_size.height,
        };
        let painted_child_size = calculate_paint_offset(sc, 0.0, child_extent);
        let cache_extent = calculate_cache_offset(sc, 0.0, child_extent);
        assert!(painted_child_size.is_finite());
        assert!(painted_child_size >= 0.0);
        SliverGeometry {
            scroll_extent: child_extent,
            paint_extent: painted_child_size,
            cache_extent,
            max_paint_extent: child_extent,
            hit_test_extent: painted_child_size,
            has_visual_overflow: child_extent > sc.remaining_paint_extent || sc.scroll_offset > 0.0,
            ..Default::default()
        }
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        for child in children {
            child.paint(ctx);
        }
    }
}
