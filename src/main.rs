use handlebars::Handlebars;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc; // Added for Arc
use tokio::net::TcpListener;
use tokio::sync::Mutex; // Required for IpAddr::from_str // Added for Handlebars

mod config;
use config::load_config; // Ensure Config is imported

extern crate serde_json;

mod routes;
use routes::{app_router, AppState};

#[derive(RustEmbed, Clone)]
#[folder = "frontend/"]
pub struct FEAssets;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
pub struct Assets;

#[tokio::main]
async fn main() {
    let config = match load_config() {
        Ok(cfg) => {
            println!("Loaded configuration: {:#?}", cfg);
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}. Exiting.", e);
            return;
        }
    };

    // Initialize Handlebars
    let mut hb = Handlebars::<'static>::new();
    hb.set_strict_mode(true); // Optional: enable strict mode

    // Register the index.html template
    match FEAssets::get("index.html") {
        Some(file) => {
            let template_content =
                std::str::from_utf8(&file.data).expect("index.html is not valid UTF-8");
            if let Err(e) = hb.register_template_string("index", template_content) {
                eprintln!("Failed to register index.html template: {}. Exiting.", e);
                return;
            }
        }
        None => {
            eprintln!("frontend/index.html not found in embedded assets. Exiting.");
            return;
        }
    }

    // Register the discovery.html template
    match FEAssets::get("discovery.html") {
        Some(file) => {
            let template_content =
                std::str::from_utf8(&file.data).expect("discovery.html is not valid UTF-8");
            if let Err(e) = hb.register_template_string("discovery", template_content) {
                eprintln!(
                    "Failed to register discovery.html template: {}. Exiting.",
                    e
                );
                return;
            }
        }
        None => {
            eprintln!("frontend/discovery.html not found in embedded assets. Exiting.");
            return;
        }
    }
    let hb_arc = Arc::new(hb);

    // Create the application state
    let app_state = AppState {
        config: config.clone(), // config needs to be Clone
        handlebars: hb_arc,
        discovered_devices: Arc::new(Mutex::new(HashMap::new())),
    };

    // Use server config for IP and Port
    let configured_ip = match IpAddr::from_str(&config.server.ip) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!(
                "Invalid IP address in config: {}. Defaulting to 127.0.0.1. Error: {}",
                config.server.ip, e
            );
            IpAddr::from([127, 0, 0, 1]) // Default to 127.0.0.1 if parsing fails
        }
    };
    let addr = SocketAddr::new(configured_ip, config.server.port);

    println!("External URL (from config): {}", config.server.external_url);
    println!("Listening on http://{}", addr);

    let mut app = app_router(app_state); // Pass the combined app_state

    // Add assets service to the router
    app = app.nest_service("/assets", axum_embed::ServeEmbed::<Assets>::new());

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}. Exiting.", addr, e);
            return;
        }
    };

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
