use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use druid_shell::piet::{Piet, PietText};

use super::render_object::{PaintContext, RenderObject, WeakRenderObject};

#[derive(Clone)]
pub(crate) struct PipelineOwner {
    inner: Rc<InnerOwner>,
}

impl Debug for PipelineOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineOwner").finish()
    }
}

impl PartialEq<PipelineOwner> for PipelineOwner {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

#[derive(Clone)]
pub(crate) struct WeakOwner {
    inner: Weak<InnerOwner>,
}

impl WeakOwner {
    pub fn upgrade(&self) -> PipelineOwner {
        self.inner
            .upgrade()
            .map(|inner| PipelineOwner { inner })
            .unwrap()
    }
}

struct InnerOwner {
    nodes_need_layout: RefCell<Vec<WeakRenderObject>>,
    nodes_need_paint: RefCell<Vec<WeakRenderObject>>,
    root: RefCell<Option<WeakRenderObject>>,
    need_visual_update: Cell<bool>,
    text: PietText,
}

impl PipelineOwner {
    pub fn new(text: PietText) -> Self {
        PipelineOwner {
            inner: Rc::new(InnerOwner {
                nodes_need_layout: RefCell::new(Vec::new()),
                nodes_need_paint: RefCell::new(Vec::new()),
                root: RefCell::new(None),
                need_visual_update: Cell::new(false),
                text,
            }),
        }
    }
    pub fn add_node_need_layout(&self, node: RenderObject) {
        self.inner
            .nodes_need_layout
            .borrow_mut()
            .push(node.downgrade());
    }

    pub fn add_node_need_paint(&self, node: RenderObject) {
        self.inner
            .nodes_need_paint
            .borrow_mut()
            .push(node.downgrade());
    }

    pub fn set_render_view(&self, node: &RenderObject) {
        if let Some(o) = &*self.inner.root.borrow() {
            o.upgrade().detach();
        }
        *self.inner.root.borrow_mut() = Some(node.downgrade());
        node.attach(self.clone());
    }

    pub fn downgrade(&self) -> WeakOwner {
        WeakOwner {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn request_visual_update(&self) {}

    pub(crate) fn enable_mutations_to_dirty_subtrees(&self, _callback: impl FnOnce()) {
        todo!()
    }

    pub(crate) fn root_node(&self) -> RenderObject {
        self.inner.root.borrow().as_ref().unwrap().upgrade()
    }

    pub(crate) fn flush_layout(&self) {
        let mut nodes = self.inner.nodes_need_layout.borrow_mut();
        nodes.sort_unstable_by(|a, b| {
            let a = a.upgrade();
            let b = b.upgrade();
            a.depth().cmp(&b.depth())
        });
        for node in &*nodes {
            let node = node.upgrade();
            tracing::debug!("flush_layout node: {:?}", node);
            if dbg!(node.needs_layout()) && dbg!(node.try_owner()) == Some(self.clone()) {
                eprintln!("layout node");
                node.layout_without_resize();
            }
        }
        nodes.clear();
    }

    pub(crate) fn flush_paint(&self, piet: &mut Piet) {
        let mut nodes = self.inner.nodes_need_paint.borrow_mut();

        nodes.sort_unstable_by(|a, b| {
            let a = a.upgrade();
            let b = b.upgrade();
            a.depth().cmp(&b.depth())
        });
        for node in &*nodes {
            let node = node.upgrade();
            tracing::debug!("paint node: {:?}", node);

            if node.needs_paint() && node.try_owner() == Some(self.clone()) {
                // check whether layer is attached
                eprintln!("paint node");

                PaintContext::repaint_composited_child(&node, piet);
            }
        }
        nodes.clear();
    }

    pub(crate) fn text(&self) -> druid_shell::piet::CoreGraphicsText {
        self.inner.text.clone()
    }
}
