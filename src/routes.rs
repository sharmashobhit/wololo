use crate::Assets;
// crate::FEAssets is no longer used directly here.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use axum_embed::ServeEmbed;
use axum_extra::extract::cookie::CookieJar;

use serde_json::json; // For constructing data for Handlebars - THIS REQUIRES serde_json in Cargo.toml
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tokio::process::Command;
use wol::*;

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

// Handler for wake-on-LAN requests
async fn wake_device_handler(
    State(app_state): State<AppState>,
    Path(device_name): Path<String>,
) -> impl IntoResponse {
    // Find the device by name
    let device = app_state.config.devices.iter().find(|d| d.name == device_name);
    
    match device {
        Some(device) => {
            // Parse MAC address
            let mac_addr = match MacAddr::from_str(&device.mac_address) {
                Ok(mac) => mac,
                Err(e) => {
                    eprintln!("Invalid MAC address for device '{}': {}", device_name, e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Html(format!("<p class='text-red-600'>Invalid MAC address: {}</p>", e)),
                    ).into_response();
                }
            };

            // Parse IP address for broadcast
            let ip_addr = match Ipv4Addr::from_str(&device.ip_address) {
                Ok(ip) => ip,
                Err(e) => {
                    eprintln!("Invalid IP address for device '{}': {}", device_name, e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Html(format!("<p class='text-red-600'>Invalid IP address: {}</p>", e)),
                    ).into_response();
                }
            };

            // Send wake-on-LAN packet
            match send_wol(mac_addr, Some(IpAddr::V4(ip_addr)), None) {
                Ok(_) => {
                    println!("Wake-on-LAN packet sent to device: {}", device_name);
                    Html(format!("<p class='text-green-600'>✅ Wake packet sent to {}</p>", device_name)).into_response()
                }
                Err(e) => {
                    eprintln!("Failed to send wake-on-LAN packet to '{}': {}", device_name, e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(format!("<p class='text-red-600'>Failed to wake device: {}</p>", e)),
                    ).into_response()
                }
            }
        }
        None => {
            eprintln!("Device '{}' not found", device_name);
            (
                StatusCode::NOT_FOUND,
                Html(format!("<p class='text-red-600'>Device '{}' not found</p>", device_name)),
            ).into_response()
        }
    }
}

// Handler for ping requests
async fn ping_device_handler(
    State(app_state): State<AppState>,
    Path(device_name): Path<String>,
) -> impl IntoResponse {
    // Find the device by name
    let device = app_state.config.devices.iter().find(|d| d.name == device_name);
    
    match device {
        Some(device) => {
            // Ping the device
            let status = ping_device(&device.ip_address).await;
            
            let (status_class, status_text, icon) = match status {
                DeviceStatus::Online => ("bg-green-100 text-green-800", "Online", "🟢"),
                DeviceStatus::Offline => ("bg-red-100 text-red-800", "Offline", "🔴"),
                DeviceStatus::Unreachable => ("bg-yellow-100 text-yellow-800", "Unreachable", "🟡"),
            };
            
            Html(format!(
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}">
                    <span class="mr-1">{}</span>
                    {}
                </span>"#,
                status_class, icon, status_text
            )).into_response()
        }
        None => {
            (
                StatusCode::NOT_FOUND,
                Html(format!("<span class='text-red-600'>Device '{}' not found</span>", device_name)),
            ).into_response()
        }
    }
}

// Handler for refreshing all devices
async fn refresh_all_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    // Create the devices HTML with updated status
    let mut devices_html = String::new();
    
    if app_state.config.devices.is_empty() {
        devices_html = r#"
        <div class="text-center py-8">
            <svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path>
            </svg>
            <p class="text-gray-500 text-lg">No devices configured</p>
            <p class="text-gray-400 text-sm mt-1">Add devices to your config.yaml file</p>
        </div>
        "#.to_string();
    } else {
        devices_html.push_str("<div class=\"grid gap-4\">");
        
        for (index, device) in app_state.config.devices.iter().enumerate() {
            let status = ping_device(&device.ip_address).await;
            let (status_class, status_text, icon) = match status {
                DeviceStatus::Online => ("bg-green-100 text-green-800", "Online", "🟢"),
                DeviceStatus::Offline => ("bg-red-100 text-red-800", "Offline", "🔴"),
                DeviceStatus::Unreachable => ("bg-yellow-100 text-yellow-800", "Unreachable", "🟡"),
            };
            
            let device_html = format!(
                "<div class=\"bg-gray-50 rounded-lg border border-gray-200 p-4 hover:shadow-md transition duration-200 ease-in-out\">\
                    <div class=\"flex items-center justify-between\">\
                        <div class=\"flex-1\">\
                            <div class=\"flex items-center gap-3 mb-2\">\
                                <h3 class=\"font-semibold text-gray-800 text-lg\">{}</h3>\
                                <div id=\"status-{}\" class=\"flex items-center\">\
                                    <span class=\"inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}\">\
                                        <span class=\"mr-1\">{}</span>\
                                        {}\
                                    </span>\
                                </div>\
                            </div>\
                            <div class=\"text-sm text-gray-600 space-y-1\">\
                                <p><span class=\"font-medium\">IP:</span> {}</p>\
                                <p><span class=\"font-medium\">MAC:</span> {}</p>\
                            </div>\
                        </div>\
                        <div class=\"flex flex-col gap-2 items-end\">\
                            <button hx-get=\"/ping/{}\" hx-target=\"#status-{}\" hx-swap=\"innerHTML\" class=\"bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-3 rounded-md shadow-sm transition duration-150 ease-in-out flex items-center\">\
                                <svg class=\"w-4 h-4 mr-1\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                    <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z\"></path>\
                                </svg>\
                                Check Status\
                            </button>\
                            <button hx-post=\"/wake/{}\" hx-target=\"#wake-response-{}\" hx-swap=\"innerHTML\" class=\"bg-green-500 hover:bg-green-600 text-white font-medium py-2 px-3 rounded-md shadow-sm transition duration-150 ease-in-out flex items-center\">\
                                <svg class=\"w-4 h-4 mr-1\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                    <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M13 10V3L4 14h7v7l9-11h-7z\"></path>\
                                </svg>\
                                Wake Up\
                            </button>\
                            <div id=\"wake-response-{}\" class=\"text-sm\"></div>\
                        </div>\
                    </div>\
                </div>",
                device.name, index, status_class, icon, status_text,
                device.ip_address, device.mac_address,
                device.name, index, device.name, index, index
            );
            
            devices_html.push_str(&device_html);
        }
        
        devices_html.push_str("</div>");
    }
    
    Html(devices_html).into_response()
}

// Enum for device status
#[derive(Debug, Clone)]
enum DeviceStatus {
    Online,
    Offline,
    Unreachable,
}

// Function to ping a device and determine its status
async fn ping_device(ip: &str) -> DeviceStatus {
    // Use ping command to check device status
    let output = Command::new("ping")
        .args(&["-c", "1", "-W", "2", ip]) // 1 packet, 2 second timeout
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                DeviceStatus::Online
            } else {
                DeviceStatus::Offline
            }
        }
        Err(_) => DeviceStatus::Unreachable,
    }
}

// Function to create and configure the Axum router
pub fn app_router(app_state: AppState) -> Router {
    // Accept the single AppState
    Router::new()
        .route("/", get(root_handler))
        .route("/hello", get(hello_handler))
        .route("/wake/:device_name", post(wake_device_handler))
        .route("/ping/:device_name", get(ping_device_handler))
        .route("/refresh-all", get(refresh_all_handler))
        .nest_service("/assets", ServeEmbed::<Assets>::new())
        .with_state(app_state) // Use with_state to make AppState available to handlers
}
