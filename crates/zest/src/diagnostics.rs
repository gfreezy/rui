struct DiagnosticsNode {
    pub name: String,
    pub properties: Vec<DiagnosticsProperty>,
    pub children: Vec<DiagnosticsNode>,
}

impl DiagnosticsNode {
    pub fn new(name: String) -> DiagnosticsNode {
        DiagnosticsNode {
            name,
            properties: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn add_property(&mut self, property: DiagnosticsProperty) {
        self.properties.push(property);
    }

    pub fn add_string_property(&mut self, name: String, value: String) {
        self.properties
            .push(DiagnosticsProperty::new_string(name, value));
    }

    pub fn add_number_property(&mut self, name: String, value: f64) {
        self.properties
            .push(DiagnosticsProperty::new_number(name, value));
    }

    pub fn add_array_property(&mut self, name: String, value: Vec<DiagnosticsPropertyValue>) {
        self.properties
            .push(DiagnosticsProperty::new_array(name, value));
    }

    pub fn add_child(&mut self, child: DiagnosticsNode) {
        self.children.push(child);
    }
}

struct DiagnosticsProperty {
    pub name: String,
    pub value: DiagnosticsPropertyValue,
}

impl DiagnosticsProperty {
    pub fn new(name: String, value: DiagnosticsPropertyValue) -> DiagnosticsProperty {
        DiagnosticsProperty { name, value }
    }

    pub fn new_string(name: String, value: String) -> DiagnosticsProperty {
        DiagnosticsProperty::new(name, DiagnosticsPropertyValue::String(value))
    }

    pub fn new_number(name: String, value: f64) -> DiagnosticsProperty {
        DiagnosticsProperty::new(name, DiagnosticsPropertyValue::Number(value))
    }

    pub fn new_array(name: String, value: Vec<DiagnosticsPropertyValue>) -> DiagnosticsProperty {
        DiagnosticsProperty::new(name, DiagnosticsPropertyValue::Array(value))
    }
}

enum DiagnosticsPropertyValue {
    String(String),
    Number(f64),
    Array(Vec<DiagnosticsPropertyValue>),
}

impl DiagnosticsPropertyValue {
    pub fn new_string(value: String) -> DiagnosticsPropertyValue {
        DiagnosticsPropertyValue::String(value)
    }
    pub fn new_number(value: f64) -> DiagnosticsPropertyValue {
        DiagnosticsPropertyValue::Number(value)
    }
    pub fn new_array(value: Vec<DiagnosticsPropertyValue>) -> DiagnosticsPropertyValue {
        DiagnosticsPropertyValue::Array(value)
    }
}
