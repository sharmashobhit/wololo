// Lib file to support testing
pub mod config;
pub mod routes;

pub use config::*;
pub use routes::*;

// Re-export AppState from routes
pub use routes::AppState;

impl AppState {
    pub fn new_for_test(config: config::Config) -> Self {
        use handlebars::Handlebars;
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        
        let mut hb = Handlebars::new();
        // Register minimal templates for testing
        hb.register_template_string("index", "test template").unwrap();
        hb.register_template_string("discovery", "discovery template").unwrap();
        
        Self {
            config,
            handlebars: Arc::new(hb),
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}