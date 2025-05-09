use crate::Assets;
// crate::FEAssets is no longer used directly here.
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_embed::ServeEmbed;
use axum_extra::extract::cookie::CookieJar;

use serde_json::json; // For constructing data for Handlebars - THIS REQUIRES serde_json in Cargo.toml

// Import AppState from main.rs (or wherever it's defined)
use crate::AppState;
// Config is now part of AppState, direct import might not be needed here unless used elsewhere
// use crate::config::Config;

// Handler for the /hello route
async fn hello_handler(_jar: CookieJar, _headers: HeaderMap) -> Html<&'static str> {
    Html("<h1>Hello from Axum! Cookies and headers printed to server console.</h1>")
}

// Handler for the / route, serving frontend/index.html via Handlebars using AppState
async fn root_handler(
    State(app_state): State<AppState>, // Extract the whole AppState
) -> impl IntoResponse {
    // Access config and handlebars from app_state
    // Adjust .devices and .server.external_url according to your actual Config struct fields
    let data = json!({
        "devices": &app_state.config.devices, // Ensure config.devices exists and is Serialize
        "external_url": &app_state.config.server.external_url,
    });

    match app_state.handlebars.render("index", &data) {
        Ok(rendered_html) => Html(rendered_html).into_response(),
        Err(e) => {
            eprintln!("Error rendering template: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1><p>Failed to render page.</p>"),
            )
                .into_response()
        }
    }
}

// Function to create and configure the Axum router
pub fn app_router(app_state: AppState) -> Router {
    // Accept the single AppState
    Router::new()
        .route("/", get(root_handler))
        .route("/hello", get(hello_handler))
        .nest_service("/assets", ServeEmbed::<Assets>::new())
        .with_state(app_state) // Use with_state to make AppState available to handlers
}
