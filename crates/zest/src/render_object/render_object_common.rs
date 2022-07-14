use super::{
    abstract_node::AbstractNode,
    pipeline_owner::PipelineOwner,
    render_box::RenderBox,
    render_object::{Constraints, Offset, PaintContext, ParentData, RenderObject},
    render_sliver::RenderSliver,
    render_view::RenderView,
};

macro_rules! impl_method {
    ($ty:ty,  { $($method:item)+ } ) => {
        impl $ty { $($method)+ }
    };
    ( $ty:ty, $($more:ty),+, { $($method:item)+ } ) => {
        impl_method!($ty, { $($method)+ });
        impl_method!($($more),+, { $($method)+ });
    };
}

impl_method! {
    RenderBox, RenderSliver, RenderView, {
        pub(crate) fn owner(&self) -> PipelineOwner {
            self.state(|s| s.owner())
        }

        pub(crate) fn try_owner(&self) -> Option<PipelineOwner> {
            self.state(|s| s.try_owner())
        }

        pub(crate) fn parent_data(&self) -> ParentData {
            self.state(|s| s.parent_data())
        }

        pub(crate) fn try_parent_data(&self) -> Option<ParentData> {
            self.state(|s| s.try_parent_data())
        }

        pub(crate) fn mark_needs_paint(&self) {
            self.state(|s| s.mark_needs_paint())
        }

        pub(crate) fn clean_relayout_boundary(&self) {
            self.state(|s| s.clean_relayout_boundary())
        }

        pub(crate) fn set_relayout_boundary(&self, bondary: Option<RenderObject>) {
            self.state(|s| s.set_relayout_boundary(bondary))
        }

        pub(crate) fn propagate_relayout_bondary(&self) {
            self.state(|s| s.propagate_relayout_bondary())
        }

        pub(crate) fn relayout_boundary(&self) -> RenderObject {
            self.state(|s| s.relayout_boundary())
        }

        pub(crate) fn invoke_layout_callback(&self, callback: impl FnOnce(&Constraints)) {
            self.state(|s| s.invoke_layout_callback(callback))
        }

        pub(crate) fn needs_layout(&self) -> bool {
            self.state(|s| s.needs_layout())
        }

        pub(crate) fn needs_paint(&self) -> bool {
            self.state(|s| s.needs_paint())
        }

        pub(crate) fn paint_with_context(&self, context: &mut PaintContext, offset: Offset) {
            self.state(|s| s.clear_needs_paint());
            self.paint(context, offset);
            assert!(!self.needs_layout());
            assert!(!self.needs_paint());
        }

        pub(crate) fn try_relayout_boundary(&self) -> Option<RenderObject> {
            self.state(|s| s.try_relayout_boundary())
        }

        pub(crate) fn doing_this_layout_with_callback(&self) -> bool {
            self.state(|s| s.doing_this_layout_with_callback())
        }

        pub(crate) fn layer(&self) -> Option<super::layer::Layer> {
            self.state(|s| s.try_layer())
        }

        pub(crate) fn set_layer(&self, child_layer: super::layer::Layer) {
            self.state(|s| s.set_layer(Some(child_layer)));
        }

        pub(crate) fn try_constraints(&self) -> Option<Constraints> {
            self.state(|s| s.try_constraints())
        }

        pub(crate) fn constraints(&self) -> Constraints {
            self.state(|s| s.constraints())
        }

    }
}
