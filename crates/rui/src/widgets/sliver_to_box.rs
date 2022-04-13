use druid_shell::{
    kurbo::{Affine, Vec2},
    piet::RenderContext,
};

use crate::{
    key::{Key, LocalKey},
    object::{Properties, RenderObject, RenderObjectInterface},
    sliver_constraints::{apply_growth_direction_to_axis_direction, AxisDirection, SliverGeometry},
    tree::Children,
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
        SliverToBoxObject {
            paint_offset: Vec2::ZERO,
        }
    }

    fn update(
        &mut self,
        ctx: &mut crate::context::UpdateCtx,
        props: SliverToBox,
        children: &mut Children,
    ) -> Self::Action {
    }
}

pub struct SliverToBoxObject {
    paint_offset: Vec2,
}

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
            children[0].layout_box(ctx, &sc.as_box_constraints(0.0, f64::INFINITY, None), true);
        let child_extent = match sc.axis() {
            crate::style::axis::Axis::Horizontal => child_size.width,
            crate::style::axis::Axis::Vertical => child_size.height,
        };

        let painted_child_size = calculate_paint_offset(sc, 0.0, child_extent);
        let cache_extent = calculate_cache_offset(sc, 0.0, child_extent);
        assert!(painted_child_size.is_finite());
        assert!(painted_child_size >= 0.0);
        let geometry = SliverGeometry::new(
            child_extent,
            None,
            painted_child_size,
            None,
            child_extent,
            None,
            None,
            None,
            child_extent > sc.remaining_paint_extent || sc.scroll_offset > 0.0,
            None,
            cache_extent,
        );

        self.paint_offset = match apply_growth_direction_to_axis_direction(
            sc.axis_direction,
            sc.growth_direction,
        ) {
            AxisDirection::Down => Vec2::new(0.0, -sc.scroll_offset),
            AxisDirection::Left => Vec2::new(
                -(geometry.scroll_extent - (geometry.paint_extent + sc.scroll_offset)),
                0.0,
            ),
            AxisDirection::Right => Vec2::new(-sc.scroll_offset, 0.0),
            AxisDirection::Up => Vec2::new(
                0.0,
                -(geometry.scroll_extent - (geometry.paint_extent + sc.scroll_offset)),
            ),
        };

        geometry
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        ctx.transform(Affine::translate(self.paint_offset));

        for child in children {
            child.paint(ctx);
        }
    }
}
