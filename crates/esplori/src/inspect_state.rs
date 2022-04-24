use std::collections::HashMap;

use rui::{debug_state::DebugState, id::ElementId};

pub struct InspectDebugState {
    /// The widget's unique id.
    pub id: ElementId,
    /// The widget's type as a human-readable string.
    pub display_name: String,
    /// If a widget has a "central" value (for instance, a textbox's contents),
    /// it is stored here.
    pub main_value: String,
    /// Untyped values that reveal useful information about the widget.
    pub other_values: HashMap<String, String>,
    pub expanded: bool,
    pub level: usize,
    pub children: Vec<InspectDebugState>,
}

impl InspectDebugState {
    pub fn new(debug_state: &DebugState) -> Self {
        Self::new_with_level(debug_state, 0)
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    fn new_with_level(debug_state: &DebugState, level: usize) -> Self {
        InspectDebugState {
            id: debug_state.id,
            display_name: debug_state.display_name.clone(),
            main_value: debug_state.main_value.clone(),
            other_values: debug_state.other_values.clone(),
            expanded: true,
            level,
            children: debug_state
                .children
                .iter()
                .map(|child| InspectDebugState::new_with_level(child, level + 1))
                .collect(),
        }
    }

    pub fn toggle(&mut self, id: ElementId) {
        if self.id == id {
            self.expanded = !self.expanded;
        } else {
            for child in &mut self.children {
                child.toggle(id);
            }
        }
    }

    pub fn flatten<R>(&self, mut line: impl (FnMut(&Self) -> R) + Clone) -> Vec<R> {
        let mut result = vec![(line.clone())(self)];
        for child in &self.children {
            if self.expanded {
                result.append(&mut child.flatten(line.clone()));
            }
        }
        result
    }
}
