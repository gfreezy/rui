use std::collections::HashMap;

use rui::{debug_state::DebugState, id::ElementId};

pub struct InspectDebugState<'a> {
    debug_state: &'a DebugState,
}

impl<'a> InspectDebugState<'a> {
    pub fn new(debug_state: &'a DebugState) -> Self {
        InspectDebugState { debug_state }
    }

    pub fn has_children(&self) -> bool {
        !self.debug_state.has_children()
    }

    pub fn flatten<R>(
        &self,
        mut filter: impl (FnMut(&DebugState, usize) -> bool),
        mut line: impl (FnMut(&DebugState, usize) -> R) + Clone,
    ) -> Vec<R> {
        flatten(self, &self.debug_state, 0, &mut filter, &mut line)
    }
}

fn flatten<R>(
    inpspect: &InspectDebugState,
    debug_state: &DebugState,
    level: usize,
    filter: &mut impl (FnMut(&DebugState, usize) -> bool),
    line: &mut impl (FnMut(&DebugState, usize) -> R),
) -> Vec<R> {
    let mut ret = vec![line(debug_state, level)];
    if filter(debug_state, level) {
        for child in debug_state.children() {
            ret.extend(flatten(inpspect, child, level + 1, filter, line));
        }
    }
    ret
}

#[derive(Default)]
pub struct ExpandedState {
    expanded: HashMap<ElementId, bool>,
}

impl ExpandedState {
    const DEFAULT: bool = true;

    pub fn expanded(&self, id: ElementId) -> bool {
        self.expanded.get(&id).cloned().unwrap_or(Self::DEFAULT)
    }

    pub fn toggle(&mut self, id: ElementId) {
        self.expanded
            .entry(id)
            .and_modify(|e| *e = !*e)
            .or_insert(!Self::DEFAULT);
    }
}
