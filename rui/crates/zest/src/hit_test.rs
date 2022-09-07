use crate::{
    geometry::{Matrix4, Offset},
    render_object::render_object::{RenderObject, WeakRenderObject},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitTestPosition {
    Box(Offset),
    Sliver {
        main_axis_position: f64,
        cross_axis_position: f64,
    },
}

#[derive(Clone)]
pub enum HitTestEntry {
    BoxHitTestEntry(BoxHitTestEntry),
    SliverHitTestEntry(SliverHitTestEntry),
}

#[derive(Clone)]
pub struct SliverHitTestEntry {
    render_object: WeakRenderObject,
    main_axis_position: f64,
    cross_axis_position: f64,
}

impl SliverHitTestEntry {
    pub fn target(&self) -> RenderObject {
        self.render_object.upgrade()
    }
}

impl HitTestEntry {
    pub fn to_box_hit_test_entry(self) -> BoxHitTestEntry {
        match self {
            HitTestEntry::BoxHitTestEntry(entry) => entry,
            HitTestEntry::SliverHitTestEntry(_entry) => todo!(),
        }
    }

    pub(crate) fn new_box_hit_test_entry(render_object: &RenderObject, position: Offset) -> Self {
        HitTestEntry::BoxHitTestEntry(BoxHitTestEntry::new(render_object, position))
    }

    pub(crate) fn new_sliver_hit_test_entry(
        render_object: &RenderObject,
        main_axis_position: f64,
        cross_axis_position: f64,
    ) -> Self {
        HitTestEntry::SliverHitTestEntry(SliverHitTestEntry {
            render_object: render_object.downgrade(),
            main_axis_position,
            cross_axis_position,
        })
    }

    delegate::delegate! {
        to match self {
            HitTestEntry::BoxHitTestEntry(e) => e,
            HitTestEntry::SliverHitTestEntry(e) => e,
        } {
            pub fn target(&self) -> RenderObject;
        }
    }
}

pub struct HitTestResult {
    entries: Vec<HitTestEntry>,
    local_transforms: Vec<Matrix4>,
    transforms: Vec<Matrix4>,
}

impl HitTestResult {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            local_transforms: vec![],
            transforms: vec![Matrix4::identity()],
        }
    }

    pub fn add(&mut self, entry: HitTestEntry) {
        tracing::debug!("add hit test target: {:?}", &entry.target());
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn add_with_paint_offset(
        &mut self,
        offset: Offset,
        position: Offset,
        hit_test: impl FnOnce(&mut HitTestResult, Offset) -> bool,
    ) -> bool {
        let transformed = position - offset;
        if offset != Offset::ZERO {
            self.push_offset(-offset);
        }
        let hit = hit_test(self, transformed);
        if offset != Offset::ZERO {
            self.pop_transform();
        }
        hit
    }

    pub fn entries(&self) -> impl Iterator<Item = &HitTestEntry> {
        self.entries.iter()
    }

    fn push_offset(&mut self, offset: Offset) {
        assert_ne!(offset, Offset::ZERO);
        self.local_transforms
            .push(Matrix4::from_translation(offset.dx, offset.dx));
    }

    fn pop_transform(&mut self) {
        if self.local_transforms.pop().is_none() {
            self.transforms.pop();
        }
    }
}

#[derive(Clone)]
pub struct BoxHitTestEntry {
    render_object: WeakRenderObject,
    position: Offset,
}

impl BoxHitTestEntry {
    pub(crate) fn new(render_object: &RenderObject, position: Offset) -> Self {
        Self {
            render_object: render_object.downgrade(),
            position,
        }
    }

    pub fn target(&self) -> RenderObject {
        self.render_object.upgrade()
    }
}

impl From<BoxHitTestEntry> for HitTestEntry {
    fn from(entry: BoxHitTestEntry) -> Self {
        HitTestEntry::BoxHitTestEntry(entry)
    }
}
