mod sliver_list_item;
mod sliver_list_parent_data;

use self::sliver_list_item::SliverListItem;
use self::sliver_list_parent_data::SliverListParentData;
use crate::{
    box_constraints::BoxConstraints,
    context::LayoutCtx,
    key::{Key, LocalKey},
    object::{AnyParentData, Properties, RenderObject, RenderObjectInterface},
    physics::tolerance::{near_equal, Tolerance},
    sliver_constraints::{
        apply_growth_direction_to_axis_direction, AxisDirection, SliverConstraints, SliverGeometry,
    },
    tree::{Children, Element},
    ui::Ui,
};
use druid_shell::kurbo::{Point, Vec2};
use std::{collections::HashMap, panic::Location};

pub trait SliverChildDelegate {
    fn key(&self, index: usize) -> String;
    fn build(&self, ui: &mut Ui, index: usize);
    fn estimated_count(&self) -> Option<usize>;
    fn find_index_by_key(&self, key: &LocalKey) -> Option<usize> {
        None
    }

    fn estimate_max_scroll_offset(
        &self,
        sc: &SliverConstraints,
        first_index: usize,
        last_index: usize,
        leading_scroll_offset: f64,
        trailing_scroll_offset: f64,
    ) -> Option<f64>;
    fn did_finish_layout(&self, first_index: usize, last_index: usize) {}
    fn should_rebuild(&self, old_delegate: &dyn SliverChildDelegate) -> bool;
}

pub struct SliverList {
    delegate: Box<dyn SliverChildDelegate>,
}

impl SliverList {
    pub fn new(delegate: Box<dyn SliverChildDelegate>) -> Self {
        SliverList { delegate }
    }

    #[track_caller]
    pub fn build(self, ui: &mut Ui) {
        ui.render_object(crate::key::Key::current(), self, |_| {})
    }
}

impl RenderObject<SliverList> for RenderSliverList {
    type Action = ();

    fn create(props: SliverList) -> Self {
        tracing::debug!("create sliver list");
        RenderSliverList {
            delegate: props.delegate,
            keep_alive_bucket: HashMap::new(),
            items: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &mut crate::context::UpdateCtx, props: SliverList) -> Self::Action {
        tracing::debug!("update sliver list");
        if props.delegate.should_rebuild(&*self.delegate) {
            tracing::debug!("rebuild sliver list");
            self.delegate = props.delegate;
            self.keep_alive_bucket = HashMap::new();
            ctx.request_layout();
        }
    }
}

impl Properties for SliverList {
    type Object = RenderSliverList;
}

pub struct RenderSliverList {
    delegate: Box<dyn SliverChildDelegate>,
    keep_alive_bucket: HashMap<usize, Element>,
    items: Vec<SliverListItem>,
}

impl RenderSliverList {
    fn insert(&mut self, children: &mut Children, child: Element, after: Option<usize>) {
        assert!(self
            .keep_alive_bucket
            .values()
            .find(|v| &v.key == &child.key)
            .is_none());
        children.insert(after.unwrap_or(0), child);
    }

    fn rebuild_items(&mut self, ui: &mut Ui) {
        for item in self.items.iter() {
            let index = self.delegate.find_index_by_key(&item.local_key);
            let caller = Key::current();
            ui.render_object((caller, item.local_key.clone()), item.clone(), |ui| {
                self.build_item(ui, item.index);
            });
        }
    }

    fn build_item(&self, ui: &mut Ui, index: usize) {
        if let Some(count) = self.delegate.estimated_count() {
            if index < count {
                self.delegate.build(ui, index);
            }
        }
    }

    fn remove(
        &mut self,
        children: &mut Children,
        child: usize,
        parent_data: &SliverListParentData,
    ) -> Option<Element> {
        if !parent_data.kept_alive {
            return children.remove_element(child);
        }
        self.keep_alive_bucket.remove(&parent_data.index)
    }

    fn add_initial_child(&mut self, ctx: &mut LayoutCtx, children: &mut Children) -> bool {
        assert!(children.is_empty());
        self.create_or_obtain_child(ctx, children, 0, None);
        if let Some(first) = children.first_mut() {
            let parent_data = first
                .parent_data_mut::<SliverListParentData>()
                .expect("no parent data");
            parent_data.layout_offset = 0.0;
            return true;
        }
        false
    }

    fn create_or_obtain_child(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        index: usize,
        after: Option<usize>,
    ) {
        invoke_layout_callback(ctx, |ctx| {
            if let Some(mut child) = self.keep_alive_bucket.remove(&index) {
                if let Some(parent_data) = child.parent_data_mut::<SliverListParentData>() {
                    assert!(parent_data.kept_alive);
                    parent_data.kept_alive = false;
                    self.insert(children, child, after);
                } else {
                    panic!("no parent data found.");
                }
            } else {
                self.create_child(ctx, children, index, after);
            }
        });
    }

    fn destroy_or_cache_child(&mut self, children: &mut Children, child_index: usize) {
        let mut parent_data = children[child_index]
            .take_parent_data::<SliverListParentData>()
            .expect("no valid parent data");
        if parent_data.keep_alive {
            assert!(!parent_data.kept_alive);
            let mut el = self.remove(children, child_index, &parent_data).unwrap();
            parent_data.kept_alive = true;
            let index = parent_data.index;
            el.set_parent_data(Some(parent_data));
            self.keep_alive_bucket.insert(index, el);
        } else {
            self.remove(children, child_index, &parent_data);
        }
    }

    fn collect_garbage(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        leading_garbage: usize,
        trailing_garbage: usize,
    ) {
        assert!(children.len() >= leading_garbage + trailing_garbage);
        invoke_layout_callback(ctx, |ctx| {
            (0..leading_garbage)
                .into_iter()
                .for_each(|_| self.destroy_or_cache_child(children, 0));
            (0..trailing_garbage).into_iter().for_each(|_| {
                let last_index = children.len() - 1;
                self.destroy_or_cache_child(children, last_index);
            });
            self.keep_alive_bucket.retain(|_, el| {
                let parent_data = el
                    .parent_data::<SliverListParentData>()
                    .expect("no valid parent data");
                parent_data.keep_alive
            });
        })
    }

    fn create_child(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        index: usize,
        after: Option<usize>,
    ) {
        let mut ui = Ui::new_in_the_middle(
            children,
            ctx.context_state,
            after.map(|v| v + 1).unwrap_or(0),
        );
        ui.set_parent_data(Some(Box::new(SliverListParentData {
            keep_alive: true,
            kept_alive: false,
            layout_offset: 0.0,
            index,
        })));

        self.build_item(&mut ui, index);
    }

    /// Called during layout to create, add, and layout the child before
    /// [firstChild].
    ///
    /// Calls [RenderSliverBoxChildManager.createChild] to actually create and add
    /// the child if necessary. The child may instead be obtained from a cache;
    /// see [SliverMultiBoxAdaptorParentData.keepAlive].
    ///
    /// Returns the new child or null if no child was obtained.
    ///
    /// The child that was previously the first child, as well as any subsequent
    /// children, may be removed by this call if they have not yet been laid out
    /// during this layout pass. No child should be added during that call except
    /// for the one that is created and returned by `createChild`.
    fn insert_and_layout_leading_child(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        child_constraints: &BoxConstraints,
    ) -> bool {
        let index = child_index(&children[0]) - 1;
        self.create_or_obtain_child(ctx, children, index, None);
        if child_index(&children[0]) == index {
            let _ = children[0].layout_box(ctx, child_constraints);
            // inserted
            return true;
        }
        // not inserted
        false
    }

    /// Called during layout to create, add, and layout the child after
    /// the given child.
    ///
    /// Calls [RenderSliverBoxChildManager.createChild] to actually create and add
    /// the child if necessary. The child may instead be obtained from a cache;
    /// see [SliverMultiBoxAdaptorParentData.keepAlive].
    ///
    /// Returns the new child. It is the responsibility of the caller to configure
    /// the child's scroll offset.
    ///
    /// Children after the `after` child may be removed in the process. Only the
    /// new child may be added.
    fn insert_and_layout_child(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Children,
        child_constraints: &BoxConstraints,
        after: usize,
    ) -> bool {
        let index = child_index(&children[after]) + 1;
        self.create_or_obtain_child(ctx, children, index, Some(after));
        if let Some(child) = children.get_mut(after + 1) {
            if child_index(child) == index {
                child.layout_box(ctx, child_constraints);
                return true;
            }
        }
        false
    }

    fn advance(
        &mut self,
        sc: &SliverConstraints,
        in_layout_range: &mut bool,
        index: &mut usize,
        end_scroll_offset: &mut f64,
        children: &mut Children,
        ctx: &mut LayoutCtx,
        child_constraints: &BoxConstraints,
        child: &mut usize,
        trailing_child_with_layout: &mut usize,
    ) -> bool {
        assert!(!children.is_empty());
        if child == trailing_child_with_layout {
            *in_layout_range = false;
        }
        *child += 1;
        if *child >= children.len() {
            *in_layout_range = false;
        }
        *index += 1;
        if !*in_layout_range {
            if children.get(*child).is_none() || child_index(&children[*child]) != *index {
                if !self.insert_and_layout_child(
                    ctx,
                    children,
                    child_constraints,
                    *trailing_child_with_layout,
                ) {
                    return false;
                }
            } else {
                children[*child].layout_box(ctx, child_constraints);
            }
            *trailing_child_with_layout = *child;
        }

        let parent_data = children[*child]
            .parent_data_mut::<SliverListParentData>()
            .expect("no valid parent data");
        parent_data.layout_offset = *end_scroll_offset;
        assert_eq!(parent_data.index, *index);
        *end_scroll_offset =
            child_scroll_offset(&children[*child]) + paint_extent_of_child(sc, &children[*child]);

        true
    }

    fn estimate_max_scroll_offset(
        &self,
        sc: &SliverConstraints,
        first_index: usize,
        last_index: usize,
        leading_scroll_offset: f64,
        trailing_scroll_offset: f64,
    ) -> f64 {
        let estimated_child_count = self.delegate.estimated_count();
        let child_count = match estimated_child_count {
            None => return f64::INFINITY,
            Some(child_count) => child_count,
        };
        if let Some(estimated_max_scroll_offset) = self.delegate.estimate_max_scroll_offset(
            sc,
            first_index,
            last_index,
            leading_scroll_offset,
            trailing_scroll_offset,
        ) {
            return estimated_max_scroll_offset;
        }

        if last_index == child_count - 1 {
            return trailing_scroll_offset;
        }
        let reified_count = (last_index - first_index + 1) as f64;
        let averate_extent = (trailing_scroll_offset - leading_scroll_offset) / reified_count;
        let remaining_count = (child_count - last_index - 1) as f64;
        trailing_scroll_offset + averate_extent * remaining_count
    }
}

impl RenderObjectInterface for RenderSliverList {
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

    fn dry_layout_box(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        bc: &crate::box_constraints::BoxConstraints,
        children: &mut crate::tree::Children,
    ) -> druid_shell::kurbo::Size {
        todo!()
    }

    fn paint(&mut self, ctx: &mut crate::context::PaintCtx, children: &mut crate::tree::Children) {
        for child in children {
            child.paint(ctx);
        }
    }

    fn layout_sliver(
        &mut self,
        ctx: &mut crate::context::LayoutCtx,
        sc: &SliverConstraints,
        children: &mut crate::tree::Children,
    ) -> SliverGeometry {
        let scroll_offset = sc.scroll_offset + sc.cache_origin;
        assert!(scroll_offset >= 0.0);
        let remaining_extent = sc.remaining_cache_extent;
        assert!(remaining_extent >= 0.0);
        let target_end_scroll_offset = scroll_offset + remaining_extent;
        let child_constraints = sc.as_box_constraints(0.0, f64::INFINITY, None);
        let mut leading_garbage = 0;
        let mut reached_end = false;

        // This algorithm in principle is straight-forward: find the first child
        // that overlaps the given scrollOffset, creating more children at the top
        // of the list if necessary, then walk down the list updating and laying out
        // each child and adding more at the end if necessary until we have enough
        // children to cover the entire viewport.
        //
        // It is complicated by one minor issue, which is that any time you update
        // or create a child, it's possible that some of the children that
        // haven't yet been laid out will be removed, leaving the list in an
        // inconsistent state, and requiring that missing nodes be recreated.
        //
        // To keep this mess tractable, this algorithm starts from what is currently
        // the first child, if any, and then walks up and/or down from there, so
        // that the nodes that might get removed are always at the edges of what has
        // already been laid out.

        if children.is_empty() {
            if !self.add_initial_child(ctx, children) {
                return SliverGeometry::ZERO;
            }
        }

        // We have at least one child.
        tracing::debug!("add initial child, children: {:?}", children);

        // These variables track the range of children that we have laid out. Within
        // this range, the children have consecutive indices. Outside this range,
        // it's possible for a child to get removed without notice.
        let mut leading_child_index_with_layout: Option<usize> = None;
        let mut trailing_child_index_with_layout: Option<usize> = None;

        // A firstChild with null layout offset is likely a result of children
        // reordering.
        //
        // We rely on firstChild to have accurate layout offset. In the case of null
        // layout offset, we have to find the first child that has valid layout
        // offset.
        if get_child_scroll_offset(&children[0]).is_none() {
            let leading_children_without_layout_offset = children
                .iter()
                .take_while(|c| get_child_scroll_offset(c).is_none())
                .count();
            self.collect_garbage(ctx, children, leading_children_without_layout_offset, 0);
            // If can not find a valid layout offset, start from the initial child.
            if children.is_empty() {
                if !self.add_initial_child(ctx, children) {
                    return SliverGeometry::ZERO;
                }
            }
        }

        // Find the last child that is at or before the scrollOffset.
        loop {
            let earlist_scroll_offset = child_scroll_offset(&children[0]);
            if earlist_scroll_offset <= scroll_offset {
                break;
            }
            // We have to add children before the earliestUsefulChild.
            if !self.insert_and_layout_leading_child(ctx, children, &child_constraints) {
                let parent_data = children[0]
                    .parent_data_mut::<SliverListParentData>()
                    .expect("no valid parent data");
                parent_data.layout_offset = 0.0;
                let index = parent_data.index;
                if scroll_offset == 0.0 {
                    // insertAndLayoutLeadingChild only lays out the children before
                    // firstChild. In this case, nothing has been laid out. We have
                    // to lay out firstChild manually.
                    let _ = children[0].layout_box(ctx, &child_constraints);

                    leading_child_index_with_layout = Some(index);
                    if trailing_child_index_with_layout.is_none() {
                        trailing_child_index_with_layout = Some(index);
                    }
                    break;
                } else {
                    return SliverGeometry::new(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        -scroll_offset,
                        None,
                    );
                }
            }

            // `earlist_scroll_offset` is the original first child's scroll offset.
            let first_child_scroll_offset =
                earlist_scroll_offset - paint_extent_of_child(sc, &children[0]);

            // firstChildScrollOffset may contain double precision error
            if first_child_scroll_offset < -PRECISION_ERROR_TOLERANCE {
                // Let's assume there is no child before the first child. We will
                // correct it on the next layout if it is not.
                let parent_data = children[0]
                    .parent_data_mut::<SliverListParentData>()
                    .expect("no valid parent data");
                parent_data.layout_offset = 0.0;

                return SliverGeometry::new(
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    -first_child_scroll_offset,
                    None,
                );
            }

            let parent_data = children[0]
                .parent_data_mut::<SliverListParentData>()
                .expect("no valid parent data");
            parent_data.layout_offset = first_child_scroll_offset;
            leading_child_index_with_layout = Some(parent_data.index);
            if trailing_child_index_with_layout.is_none() {
                trailing_child_index_with_layout = Some(parent_data.index);
            }
        }

        assert!(child_scroll_offset(&children[0]) > -PRECISION_ERROR_TOLERANCE);

        // If the scroll offset is at zero, we should make sure we are
        // actually at the beginning of the list.
        if scroll_offset < PRECISION_ERROR_TOLERANCE {
            loop {
                let first_child_index = child_index(&children[0]);
                if first_child_index == 0 {
                    break;
                }
                let earlist_scroll_offset = child_scroll_offset(&children[0]);
                let inserted =
                    self.insert_and_layout_leading_child(ctx, children, &child_constraints);
                assert!(inserted);
                let first_child_scroll_offset =
                    earlist_scroll_offset - paint_extent_of_child(sc, &children[0]);

                let parent_data = children[0]
                    .parent_data_mut::<SliverListParentData>()
                    .expect("no valid parent data");
                parent_data.layout_offset = 0.0;

                if first_child_scroll_offset < -PRECISION_ERROR_TOLERANCE {
                    return SliverGeometry::new(
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        -first_child_scroll_offset,
                        None,
                    );
                }
            }
        }

        // At this point, earliestUsefulChild is the first child, and is a child
        // whose scrollOffset is at or before the scrollOffset, and
        // leadingChildWithLayout and trailingChildWithLayout are either null or
        // cover a range of render boxes that we have laid out with the first being
        // the same as earliestUsefulChild and the last being either at or after the
        // scroll offset.
        assert!(child_scroll_offset(&children[0]) <= scroll_offset);
        if leading_child_index_with_layout.is_none() {
            let size = children[0].layout_box(ctx, &child_constraints);
            tracing::debug!("first_child layout size: {:?}", size);
        }
        let mut trailing_child_with_layout =
            find_child_with_index(children, trailing_child_index_with_layout).unwrap_or(0);

        // Here, earliestUsefulChild is still the first child, it's got a
        // scrollOffset that is at or before our actual scrollOffset, and it has
        // been laid out, and is in fact our leadingChildWithLayout. It's possible
        // that some children beyond that one have also been laid out.

        let mut in_layout_range = true;
        let mut child = 0;
        let mut index = child_index(&children[0]);
        let first_child = &children[0];
        let mut end_scroll_offset =
            child_scroll_offset(&first_child) + paint_extent_of_child(sc, &first_child);

        // Find the first child that ends after the scroll offset.
        while end_scroll_offset < scroll_offset {
            leading_garbage += 1;
            if !self.advance(
                sc,
                &mut in_layout_range,
                &mut index,
                &mut end_scroll_offset,
                children,
                ctx,
                &child_constraints,
                &mut child,
                &mut trailing_child_with_layout,
            ) {
                assert_eq!(leading_garbage, children.len());
                self.collect_garbage(ctx, children, leading_garbage - 1, 0);
                assert_eq!(children.len(), 1);
                let extent =
                    child_scroll_offset(&children[0]) + paint_extent_of_child(sc, &children[0]);
                return SliverGeometry::new(
                    extent,
                    None,
                    None,
                    None,
                    extent,
                    None,
                    None,
                    None,
                    None,
                    -scroll_offset,
                    None,
                );
            }
        }

        // Now find the first child that ends after our end.
        while end_scroll_offset < target_end_scroll_offset {
            if !self.advance(
                sc,
                &mut in_layout_range,
                &mut index,
                &mut end_scroll_offset,
                children,
                ctx,
                &child_constraints,
                &mut child,
                &mut trailing_child_with_layout,
            ) {
                reached_end = true;
                break;
            }
        }

        // Finally count up all the remaining children and label them as garbage. keep one
        let trailing_garbage = children.len() - child;

        // At this point everything should be good to go, we just have to clean up
        // the garbage and report the geometry.
        self.collect_garbage(ctx, children, 0, trailing_garbage);

        let estimated_max_scroll_offset = if reached_end {
            end_scroll_offset
        } else {
            let first_index = child_index(&children[0]);
            let last_index = child_index(children.last().unwrap());
            let leading_scroll_offset = child_scroll_offset(&children[0]);
            let estimated_max_scroll_offset = self.estimate_max_scroll_offset(
                sc,
                first_index,
                last_index,
                leading_scroll_offset,
                end_scroll_offset,
            );
            assert!(
                estimated_max_scroll_offset
                    >= end_scroll_offset - child_scroll_offset(&children[0])
            );
            estimated_max_scroll_offset
        };

        let paint_extent =
            calculate_paint_offset(sc, child_scroll_offset(&children[0]), end_scroll_offset);
        let cache_extent =
            calculate_cache_offset(sc, child_scroll_offset(&children[0]), end_scroll_offset);
        let target_end_scroll_offset_for_paint = sc.scroll_offset + sc.remaining_paint_extent;

        let geometry = SliverGeometry::new(
            estimated_max_scroll_offset,
            None,
            paint_extent,
            None,
            estimated_max_scroll_offset,
            None,
            None,
            None,
            end_scroll_offset > target_end_scroll_offset_for_paint || sc.scroll_offset > 0.0,
            -scroll_offset,
            cache_extent,
        );
        let offset = ctx.child_state.origin;
        let paint_extent = geometry.paint_extent;
        let (main_axis_unint, cross_axis_unit, origin_offset, add_extent) =
            match apply_growth_direction_to_axis_direction(sc.axis_direction, sc.growth_direction) {
                AxisDirection::Up => (
                    Vec2::new(0., -1.),
                    Vec2::new(1., 0.),
                    offset + Vec2::new(0., paint_extent),
                    true,
                ),
                AxisDirection::Right => (Vec2::new(1., 0.), Vec2::new(0., 1.), offset, false),
                AxisDirection::Down => (Vec2::new(0., 1.), Vec2::new(1., 0.), offset, false),
                AxisDirection::Left => (
                    Vec2::new(-1., 0.),
                    Vec2::new(0., 1.),
                    offset + Vec2::new(0., paint_extent),
                    true,
                ),
            };
        for child in children {
            let main_axis_delta = child_main_axis_position(sc, child);
            let cross_axis_delta = child_cross_axis_position(sc, child);
            let mut child_origin = Point::new(
                origin_offset.x
                    + main_axis_unint.x * main_axis_delta
                    + cross_axis_unit.x * cross_axis_delta,
                origin_offset.y
                    + main_axis_unint.y * main_axis_delta
                    + cross_axis_unit.y * cross_axis_delta,
            );
            if add_extent {
                child_origin += main_axis_unint * paint_extent_of_child(sc, child);
            }

            if main_axis_delta < sc.remaining_paint_extent
                && main_axis_delta + paint_extent_of_child(sc, child) > 0.
            {
                child.set_origin(ctx, child_origin);
            }
        }

        geometry
    }
}

const PRECISION_ERROR_TOLERANCE: f64 = 1e-10;

fn get_child_scroll_offset(child: &Element) -> Option<f64> {
    let parent_data = child.parent_data::<SliverListParentData>();
    parent_data.map(|d| d.layout_offset)
}

fn child_scroll_offset(child: &Element) -> f64 {
    get_child_scroll_offset(child).unwrap()
}

fn child_main_axis_position(sc: &SliverConstraints, child: &Element) -> f64 {
    child_scroll_offset(child) - sc.scroll_offset
}

fn child_cross_axis_position(sc: &SliverConstraints, child: &Element) -> f64 {
    0.0
}

fn child_index(child: &Element) -> usize {
    let parent_data = child.parent_data::<SliverListParentData>();
    parent_data.map(|d| d.index).unwrap()
}

fn find_child_with_index(children: &mut Children, index: Option<usize>) -> Option<usize> {
    let index = if let Some(index) = index {
        index
    } else {
        return None;
    };
    children.iter_mut().position(|e| child_index(e) == index)
}

fn paint_extent_of_child(sc: &SliverConstraints, child: &Element) -> f64 {
    assert!(!child.size().is_empty());
    match sc.axis() {
        crate::style::axis::Axis::Horizontal => child.size().width,
        crate::style::axis::Axis::Vertical => child.size().height,
    }
}
/// Allows mutations to be made to this object's child list (and any
/// descendants) as well as to any other dirty nodes in the render tree owned
/// by the same [PipelineOwner] as this object. The `callback` argument is
/// invoked synchronously, and the mutations are allowed only during that
/// callback's execution.
///
/// This exists to allow child lists to be built on-demand during layout (e.g.
/// based on the object's size), and to enable nodes to be moved around the
/// tree as this happens (e.g. to handle [GlobalKey] reparenting), while still
/// ensuring that any particular node is only laid out once per frame.
///
/// Calling this function disables a number of assertions that are intended to
/// catch likely bugs. As such, using this function is generally discouraged.
///
/// This function can only be called during layout.
fn invoke_layout_callback<R, T: FnOnce(&mut LayoutCtx) -> R>(
    ctx: &mut LayoutCtx,
    callback: T,
) -> R {
    ctx.child_state.doing_this_layout_with_callback = true;
    let ret = callback(ctx);
    ctx.child_state.doing_this_layout_with_callback = false;
    ret
}

/// Computes the portion of the region from `from` to `to` that is visible,
/// assuming that only the region from the [SliverConstraints.scrollOffset]
/// that is [SliverConstraints.remainingPaintExtent] high is visible, and that
/// the relationship between scroll offsets and paint offsets is linear.
///
/// For example, if the constraints have a scroll offset of 100 and a
/// remaining paint extent of 100, and the arguments to this method describe
/// the region 50..150, then the returned value would be 50 (from scroll
/// offset 100 to scroll offset 150).
///
/// This method is not useful if there is not a 1:1 relationship between
/// consumed scroll offset and consumed paint extent. For example, if the
/// sliver always paints the same amount but consumes a scroll offset extent
/// that is proportional to the [SliverConstraints.scrollOffset], then this
/// function's results will not be consistent.
// This could be a static method but isn't, because it would be less convenient
// to call it from subclasses if it was.
pub fn calculate_paint_offset(sc: &SliverConstraints, from: f64, to: f64) -> f64 {
    assert!(from <= to);
    let a = sc.scroll_offset;
    let b = sc.scroll_offset + sc.remaining_paint_extent;
    (to.clamp(a, b) - from.clamp(a, b)).clamp(0.0, sc.remaining_paint_extent)
}

/// Computes the portion of the region from `from` to `to` that is within
/// the cache extent of the viewport, assuming that only the region from the
/// [SliverConstraints.cacheOrigin] that is
/// [SliverConstraints.remainingCacheExtent] high is visible, and that
/// the relationship between scroll offsets and paint offsets is linear.
///
/// This method is not useful if there is not a 1:1 relationship between
/// consumed scroll offset and consumed cache extent.
pub fn calculate_cache_offset(sc: &SliverConstraints, from: f64, to: f64) -> f64 {
    assert!(from <= to);
    let a = sc.scroll_offset + sc.cache_origin;
    let b = sc.scroll_offset + sc.remaining_cache_extent;
    (to.clamp(a, b) - from.clamp(a, b)).clamp(0.0, sc.remaining_cache_extent)
}
