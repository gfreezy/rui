use flipper_client::{json, FlipperClient, FlipperPlugin, Result, Value};

use crate::render_object::render_object::RenderObject;

pub struct LayoutPlugin {
    root: RenderObject,
}

impl LayoutPlugin {
    pub fn new(root: RenderObject) -> Self {
        LayoutPlugin { root }
    }
}

impl FlipperPlugin for LayoutPlugin {
    fn get_id(&self) -> String {
        "Inspector".to_string()
    }

    fn on_connect(&mut self) {
        tracing::info!("connected to inspector");
    }

    fn on_disconnect(&mut self) {
        tracing::info!("disconnected from inspector");
    }

    fn run_in_background(&self) -> bool {
        tracing::info!("run in background");
        false
    }

    fn call(&mut self, method: &str, params: &Value) -> Result<Value> {
        tracing::info!(?method, ?params);

        Ok(Value::Null)
    }

    fn is_method_supported(&self, method: &str) -> bool {
        false
    }
}
