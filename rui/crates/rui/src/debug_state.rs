//! A data structure for representing widget trees.

use std::collections::{BTreeMap, HashMap};

use crate::id::ElementId;

/// A description widget and its children, clonable and comparable, meant
/// for testing and debugging. This is extremely not optimized.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct DebugState {
    /// The widget's unique id.
    pub id: ElementId,
    /// The widget's type as a human-readable string.
    pub display_name: String,
    /// If a widget has a "central" value (for instance, a textbox's contents),
    /// it is stored here.
    pub main_value: String,
    /// Untyped values that reveal useful information about the widget.
    pub other_values: HashMap<String, String>,
    /// Debug info of child widgets.
    pub children: Vec<DebugState>,
}

impl std::fmt::Debug for DebugState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.other_values.is_empty() && self.children.is_empty() && self.main_value.is_empty() {
            f.write_str(&self.display_name)
        } else if self.other_values.is_empty() && self.children.is_empty() {
            f.debug_tuple(&self.display_name)
                .field(&self.main_value)
                .finish()
        } else if self.other_values.is_empty() && self.main_value.is_empty() {
            let mut f_tuple = f.debug_tuple(&self.display_name);
            for child in &self.children {
                f_tuple.field(child);
            }
            f_tuple.finish()
        } else {
            let mut f_struct = f.debug_struct(&self.display_name);
            if !self.main_value.is_empty() {
                f_struct.field("_main_value_", &self.main_value);
            }
            f_struct.field("child_len", &self.children.len());
            let mut other_values: Vec<_> = self.other_values.iter().collect();
            other_values.sort();
            for (key, value) in other_values {
                f_struct.field(key, &value);
            }
            if !self.children.is_empty() {
                f_struct.field("children", &self.children);
            }
            f_struct.finish()
        }
    }
}

impl DebugState {
    pub fn visit<T: FnMut(&DebugState, usize)>(&self, visitor: &mut T, level: usize) {
        visitor(self, level);
        for child in &self.children {
            child.visit(visitor, level + 1);
        }
    }

    pub fn debug_state_for_id(&self, id: ElementId) -> Option<&DebugState> {
        if self.id == id {
            Some(self)
        } else {
            for child in &self.children {
                if let Some(child) = child.debug_state_for_id(id) {
                    return Some(child);
                }
            }
            None
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn children(&self) -> impl Iterator<Item = &DebugState> {
        self.children.iter()
    }
}

impl ToString for DebugState {
    fn to_string(&self) -> String {
        let mut map: BTreeMap<_, _> = self
            .other_values
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        map.insert("display_name".to_string(), self.display_name.clone());
        map.insert("main_value".to_string(), self.main_value.clone());
        map.insert("id".to_string(), self.id.to_string());
        format!("{:#?}", map)
    }
}
