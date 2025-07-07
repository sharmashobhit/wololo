// Assets will be provided by main.rs when used as binary
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
// ServeEmbed will be used in main.rs
use axum_extra::extract::cookie::CookieJar;

use futures::future::join_all;
use ipnet::Ipv4Net;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use regex::Regex;
use serde_json::json; // For constructing data for Handlebars - THIS REQUIRES serde_json in Cargo.toml
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tokio::process::Command;
use wol::*;

use crate::config::Config;
use handlebars::Handlebars;
use serde_urlencoded;
use serde_yaml;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// AppState definition for routes module
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub handlebars: Arc<Handlebars<'static>>,
    pub discovered_devices: Arc<Mutex<HashMap<String, Vec<DiscoveredDevice>>>>,
}

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
        "sync_enabled": &app_state.config.sync.enabled,
        "sync_interval": &app_state.config.sync.interval_seconds,
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
    let device = app_state
        .config
        .devices
        .iter()
        .find(|d| d.name == device_name);

    match device {
        Some(device) => {
            // Parse MAC address
            let mac_addr = match MacAddr::from_str(&device.mac_address) {
                Ok(mac) => mac,
                Err(e) => {
                    eprintln!("Invalid MAC address for device '{}': {}", device_name, e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Html(format!(
                            "<p class='text-red-600'>Invalid MAC address: {}</p>",
                            e
                        )),
                    )
                        .into_response();
                }
            };

            // Parse IP address for broadcast
            let ip_addr = match Ipv4Addr::from_str(&device.ip_address) {
                Ok(ip) => ip,
                Err(e) => {
                    eprintln!("Invalid IP address for device '{}': {}", device_name, e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Html(format!(
                            "<p class='text-red-600'>Invalid IP address: {}</p>",
                            e
                        )),
                    )
                        .into_response();
                }
            };

            // Send wake-on-LAN packet
            match send_wol(mac_addr, Some(IpAddr::V4(ip_addr)), None) {
                Ok(_) => {
                    println!("Wake-on-LAN packet sent to device: {}", device_name);
                    Html(format!(
                        "<p class='text-green-600'>✅ Wake packet sent to {}</p>",
                        device_name
                    ))
                    .into_response()
                }
                Err(e) => {
                    eprintln!(
                        "Failed to send wake-on-LAN packet to '{}': {}",
                        device_name, e
                    );
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(format!(
                            "<p class='text-red-600'>Failed to wake device: {}</p>",
                            e
                        )),
                    )
                        .into_response()
                }
            }
        }
        None => {
            eprintln!("Device '{}' not found", device_name);
            (
                StatusCode::NOT_FOUND,
                Html(format!(
                    "<p class='text-red-600'>Device '{}' not found</p>",
                    device_name
                )),
            )
                .into_response()
        }
    }
}

// Handler for ping requests
async fn ping_device_handler(
    State(app_state): State<AppState>,
    Path(device_name): Path<String>,
) -> impl IntoResponse {
    // Find the device by name
    let device = app_state
        .config
        .devices
        .iter()
        .find(|d| d.name == device_name);

    match device {
        Some(device) => {
            // Ping the device
            let status = ping_device(&device.ip_address).await;

            let (status_class, status_text, icon) = match status {
                DeviceStatus::Online => ("bg-green-500 text-white", "Online", "🟢"),
                DeviceStatus::Offline => ("bg-red-500 text-white", "Offline", "🔴"),
                DeviceStatus::Unreachable => ("bg-yellow-500 text-white", "Unreachable", "🟡"),
            };

            Html(format!(
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}">
                    <span class="mr-1">{}</span>
                    {}
                </span>"#,
                status_class, icon, status_text
            )).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Html(format!(
                "<span class='text-red-600'>Device '{}' not found</span>",
                device_name
            )),
        )
            .into_response(),
    }
}

// Handler for refreshing all devices
async fn refresh_all_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    // Create the devices HTML with updated status
    let mut devices_html = String::new();

    if app_state.config.devices.is_empty() {
        devices_html = r#"
        <div class="text-center py-16">
            <div class="bg-gray-800 rounded-full p-6 w-24 h-24 mx-auto mb-6">
                <svg class="w-12 h-12 text-gray-500 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path>
                </svg>
            </div>
            <h3 class="text-xl font-semibold text-white mb-2">No devices configured</h3>
            <p class="text-white/70 max-w-sm mx-auto">Start by adding devices to your config.yaml file or discover devices on your network</p>
            <a href="/discovery" class="inline-flex items-center space-x-2 mt-6 bg-gray-700 hover:bg-gray-600 text-white font-medium py-3 px-6 rounded-xl transition-all duration-200">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
                </svg>
                <span>Discover Devices</span>
            </a>
        </div>
        "#.to_string();
    } else {
        devices_html.push_str("<div class=\"grid gap-6\">");

        for (index, device) in app_state.config.devices.iter().enumerate() {
            let status = ping_device(&device.ip_address).await;
            let (status_class, status_text, _icon, status_bg_color) = match status {
                DeviceStatus::Online => ("text-green-400", "Online", "●", "bg-green-500"),
                DeviceStatus::Offline => ("text-red-400", "Offline", "●", "bg-red-500"),
                DeviceStatus::Unreachable => {
                    ("text-yellow-400", "Unreachable", "●", "bg-yellow-500")
                }
            };

            let device_html = format!(
                "<div class=\"bg-gray-800 rounded-2xl p-6 border border-gray-700 transition-all duration-300\">\
                    <div class=\"flex flex-col lg:flex-row lg:items-center lg:justify-between space-y-4 lg:space-y-0\">\
                        <div class=\"flex-1\">\
                            <div class=\"flex items-center gap-4 mb-4\">\
                                <div class=\"bg-gray-700 p-3 rounded-xl\">\
                                    <svg class=\"w-6 h-6 text-white\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                        <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z\"></path>\
                                    </svg>\
                                </div>\
                                <div>\
                                    <h3 class=\"text-xl font-bold text-white mb-1\">{}</h3>\
                                    <div id=\"status-{}\" class=\"flex items-center space-x-2\">\
                                        <div class=\"flex items-center space-x-2 px-3 py-1 rounded-full bg-gray-700\">\
                                            <span class=\"w-2 h-2 rounded-full {}\"></span>\
                                            <span class=\"text-sm font-medium {}\">{}</span>\
                                        </div>\
                                    </div>\
                                </div>\
                            </div>\
                            <div class=\"grid grid-cols-1 md:grid-cols-2 gap-3 text-sm\">\
                                <div class=\"bg-gray-900 rounded-lg p-3\">\
                                    <span class=\"text-gray-400 font-medium\">IP Address</span>\
                                    <p class=\"text-white font-mono\">{}</p>\
                                </div>\
                                <div class=\"bg-gray-900 rounded-lg p-3\">\
                                    <span class=\"text-gray-400 font-medium\">MAC Address</span>\
                                    <p class=\"text-white font-mono text-sm\">{}</p>\
                                </div>\
                            </div>\
                        </div>\
                        <div class=\"flex flex-row lg:flex-col gap-3 lg:items-end\">\
                            <button hx-get=\"/ping/{}\" hx-target=\"#status-{}\" hx-swap=\"innerHTML\" class=\"group flex-1 lg:flex-none bg-gray-700 hover:bg-gray-600 text-white font-medium py-3 px-4 rounded-xl transition-all duration-200 flex items-center justify-center space-x-2\">\
                                <svg class=\"w-4 h-4\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                    <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z\"></path>\
                                </svg>\
                                <span>Check</span>\
                            </button>\
                            <button hx-post=\"/wake/{}\" hx-target=\"#wake-response-{}\" hx-swap=\"innerHTML\" class=\"group flex-1 lg:flex-none bg-emerald-700 hover:bg-emerald-600 text-white font-medium py-3 px-4 rounded-xl transition-all duration-200 flex items-center justify-center space-x-2\">\
                                <svg class=\"w-4 h-4\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                    <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M13 10V3L4 14h7v7l9-11h-7z\"></path>\
                                </svg>\
                                <span>Wake</span>\
                            </button>\
                        </div>\
                    </div>\
                    <div id=\"wake-response-{}\" class=\"mt-4 text-sm\"></div>\
                </div>",
                device.name, index, status_bg_color, status_class, status_text,
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

// Discovery page handler
async fn discovery_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let data = json!({
        "devices": &app_state.config.devices,
        "device_count": app_state.config.devices.len(),
        "config_data": &app_state.config, // Add config_data to the context
    });

    match app_state.handlebars.render("discovery", &data) {
        Ok(rendered_html) => Html(rendered_html).into_response(),
        Err(e) => {
            eprintln!("Error rendering discovery template: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Error</h1><p>Failed to render discovery page.</p>"),
            )
                .into_response()
        }
    }
}

// Network scan handler
async fn discovery_scan_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Starting network discovery scan...");

    // Discover devices on the network
    let discovered_devices = discover_network_devices().await;

    // Store discovered devices in app state for later use
    {
        let mut storage = app_state.discovered_devices.lock().await;
        storage.insert("latest_scan".to_string(), discovered_devices.clone());
    }

    let mut discovered_devices_html = String::new();

    if discovered_devices.is_empty() {
        discovered_devices_html = r#"
        <div class="text-center py-16">
            <div class="bg-gray-800 rounded-full p-6 w-24 h-24 mx-auto mb-6">
                <svg class="w-12 h-12 text-gray-500 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
            </div>
            <h3 class="text-xl font-semibold text-white mb-2">No devices found</h3>
            <p class="text-white/70">Ensure your devices are on the same network and WOL is enabled.</p>
        </div>
        "#.to_string();
    } else {
        discovered_devices_html.push_str("<form id=\"discovery-form\" hx-post=\"/discovery/generate-config\" hx-target=\"#config-preview\" hx-swap=\"outerHTML\">");
        discovered_devices_html.push_str("<div class=\"space-y-4\">");
        
        // Add select all/unselect all controls
        discovered_devices_html.push_str(r#"
        <div class="flex items-center justify-between p-4 bg-white/5 rounded-lg backdrop-blur-sm border border-white/10">
            <div class="flex items-center space-x-3">
                <input type="checkbox" id="select-all" class="form-checkbox h-5 w-5 bg-gray-900 border-gray-600 text-emerald-600 focus:ring-emerald-500 rounded">
                <label for="select-all" class="text-white font-medium">Select All Devices</label>
            </div>
            <span class="text-white/60 text-sm" id="selection-count">0 devices selected</span>
        </div>
        "#);

        for device in discovered_devices {
            let device_html = format!(
                r#"<div class="bg-gray-800 p-4 rounded-lg border border-gray-700 flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        <input type="checkbox" name="selected_devices" value='{{"ip_address":"{}","mac_address":"{}","hostname":"{}"}}' class="device-checkbox form-checkbox h-5 w-5 bg-gray-900 border-gray-600 text-emerald-600 focus:ring-emerald-500 rounded">
                        <div>
                            <p class="font-semibold text-white">{}</p>
                            <p class="text-sm text-gray-400">{}</p>
                        </div>
                    </div>
                    <span class="text-xs font-mono text-gray-500 bg-gray-900 px-2 py-1 rounded-md">{}</span>
                </div>"#,
                device.ip_address,
                device.mac_address.as_deref().unwrap_or("N/A"),
                device.hostname.as_deref().unwrap_or("N/A"),
                device.hostname.as_deref().unwrap_or(&device.ip_address),
                device.ip_address,
                device.mac_address.as_deref().unwrap_or("N/A")
            );
            discovered_devices_html.push_str(&device_html);
        }

        discovered_devices_html.push_str("</div>");
        discovered_devices_html.push_str(r#"
        <div class="mt-8 text-right">
            <button type="submit" class="bg-emerald-600 hover:bg-emerald-500 text-white font-bold py-3 px-6 rounded-lg transition-colors">
                Generate Configuration
            </button>
        </div>
        "#);
        discovered_devices_html.push_str("</form>");
        
        // Add JavaScript for select all functionality
        discovered_devices_html.push_str(r#"
        <script>
            document.getElementById('select-all').addEventListener('change', function() {
                const checkboxes = document.querySelectorAll('.device-checkbox');
                checkboxes.forEach(checkbox => {
                    checkbox.checked = this.checked;
                });
                updateSelectionCount();
            });
            
            // Update selection count when individual checkboxes change
            document.querySelectorAll('.device-checkbox').forEach(checkbox => {
                checkbox.addEventListener('change', function() {
                    updateSelectionCount();
                    updateSelectAllState();
                });
            });
            
            function updateSelectionCount() {
                const selectedCount = document.querySelectorAll('.device-checkbox:checked').length;
                const totalCount = document.querySelectorAll('.device-checkbox').length;
                document.getElementById('selection-count').textContent = selectedCount + ' of ' + totalCount + ' devices selected';
            }
            
            function updateSelectAllState() {
                const checkboxes = document.querySelectorAll('.device-checkbox');
                const checkedBoxes = document.querySelectorAll('.device-checkbox:checked');
                const selectAllCheckbox = document.getElementById('select-all');
                
                if (checkedBoxes.length === 0) {
                    selectAllCheckbox.indeterminate = false;
                    selectAllCheckbox.checked = false;
                } else if (checkedBoxes.length === checkboxes.length) {
                    selectAllCheckbox.indeterminate = false;
                    selectAllCheckbox.checked = true;
                } else {
                    selectAllCheckbox.indeterminate = true;
                }
            }
            
            // Initialize counts
            updateSelectionCount();
        </script>
        "#);
    }

    Html(discovered_devices_html)
}

// Handler for generating YAML config from discovered devices
async fn generate_config_handler(
    State(app_state): State<AppState>,
    body: String,
) -> impl IntoResponse {
    let selected_json_strings: Vec<String> = serde_urlencoded::from_str::<Vec<(String, String)>>(&body)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, value)| {
            if key == "selected_devices" {
                Some(value)
            } else {
                None
            }
        })
        .collect();

    // Parse the JSON strings to extract IP addresses
    let selected_ips: Vec<String> = selected_json_strings
        .iter()
        .filter_map(|json_str| {
            // Parse the JSON string to extract the IP address
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(ip) = json_value.get("ip_address").and_then(|v| v.as_str()) {
                    Some(ip.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Get the discovered devices from storage
    let discovered_devices = {
        let storage = app_state.discovered_devices.lock().await;
        storage.get("latest_scan").cloned().unwrap_or_default()
    };

    let selected_devices: Vec<DiscoveredDevice> = discovered_devices
        .into_iter()
        .filter(|device| selected_ips.contains(&device.ip_address))
        .collect();

    let config_yaml = generate_config_yaml(&app_state.config, &selected_devices).await;

    // Store the generated config for download
    {
        let mut stored_config = GENERATED_CONFIG.lock().await;
        *stored_config = Some(config_yaml.clone());
    }

    let response_html = format!(
        r#"<div id="config-preview" class="glass rounded-2xl p-6 sm:p-8 mb-8 card-hover">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
                <h3 class="text-xl sm:text-2xl font-bold text-white">Generated Configuration</h3>
                <div class="flex items-center space-x-2">
                    <div class="w-2 h-2 bg-green-400 rounded-full"></div>
                    <span class="text-white/80 text-sm">Ready to download</span>
                </div>
            </div>
            <div class="bg-white/10 backdrop-blur-sm rounded-xl p-4 border border-white/20 mb-6 overflow-x-auto">
                <pre class="text-sm text-white/90 font-mono"><code>{}</code></pre>
            </div>
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <p class="text-sm text-white/70">Review the configuration and save it to your <code class="bg-white/20 text-white px-2 py-1 rounded-md font-mono text-xs">config.yaml</code> file.</p>
                <div class="flex gap-3">
                    <button id="copy-button" class="bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white font-semibold py-2 px-4 rounded-lg transition-all duration-200 flex items-center space-x-2">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>
                        </svg>
                        <span>Copy</span>
                    </button>
                    <a href="/discovery/download-config" download="config.yaml" class="bg-emerald-600 hover:bg-emerald-500 text-white font-semibold py-2 px-4 rounded-lg transition-all duration-200 flex items-center space-x-2 no-underline">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                        </svg>
                        <span>Download</span>
                    </a>
                </div>
            </div>
            <script>
                document.getElementById('copy-button').addEventListener('click', () => {{
                    const textToCopy = `{}`;
                    navigator.clipboard.writeText(textToCopy).then(() => {{
                        // Show success feedback
                        const button = document.getElementById('copy-button');
                        const originalText = button.innerHTML;
                        button.innerHTML = '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg><span>Copied!</span>';
                        setTimeout(() => {{
                            button.innerHTML = originalText;
                        }}, 2000);
                    }}, (err) => {{
                        alert('Failed to copy configuration.');
                        console.error('Could not copy text: ', err);
                    }});
                }});
            </script>
        </div>"#,
        config_yaml,
        config_yaml.replace("`", "\\`")
    );

    Html(response_html)
}

// Store the generated config temporarily
static GENERATED_CONFIG: tokio::sync::Mutex<Option<String>> = tokio::sync::Mutex::const_new(None);

// Config download handler
async fn download_config_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    // Check if we have a generated config, otherwise use current config
    let config_content = {
        let stored_config = GENERATED_CONFIG.lock().await;
        if let Some(config) = stored_config.as_ref() {
            config.clone()
        } else {
            generate_config_yaml(&app_state.config, &[]).await
        }
    };

    let headers = [
        ("Content-Type", "application/x-yaml"),
        (
            "Content-Disposition",
            "attachment; filename=\"config.yaml\"",
        ),
    ];

    (headers, config_content).into_response()
}

// Discovered device structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredDevice {
    pub ip_address: String,
    pub mac_address: Option<String>,
    pub hostname: Option<String>,
    pub status: String,
}

// Network discovery function
async fn discover_network_devices() -> Vec<DiscoveredDevice> {
    let mut discovered_devices = Vec::new();

    // Get network interfaces
    let interfaces = NetworkInterface::show().unwrap_or_default();

    for interface in interfaces {
        if let Some(addr) = interface
            .addr
            .iter()
            .find(|addr| addr.ip().is_ipv4() && !addr.ip().is_loopback())
        {
            // Use a common subnet assumption for simplicity
            let network_str = format!("{}/24", addr.ip());
            if let Ok(network) = network_str.parse::<Ipv4Net>() {
                println!("Scanning network: {}", network);

                // Scan the network range
                let scan_results = scan_network_range(network).await;
                discovered_devices.extend(scan_results);
            }
        }
    }

    discovered_devices
}

// Scan network range function
async fn scan_network_range(network: Ipv4Net) -> Vec<DiscoveredDevice> {
    let mut devices = Vec::new();
    let mut scan_futures = Vec::new();

    // Limit scan to reasonable range (e.g., first 254 hosts)
    let host_count = std::cmp::min(network.hosts().count(), 254);

    for host_ip in network.hosts().take(host_count) {
        let ip_str = host_ip.to_string();
        scan_futures.push(scan_single_host(ip_str));
    }

    let results = join_all(scan_futures).await;

    for result in results {
        if let Some(device) = result {
            devices.push(device);
        }
    }

    devices
}

// Scan single host function
async fn scan_single_host(ip: String) -> Option<DiscoveredDevice> {
    // Ping the host
    let ping_result = Command::new("ping")
        .args(&["-c", "1", "-W", "1", &ip])
        .output()
        .await;

    if let Ok(output) = ping_result {
        if output.status.success() {
            // Host is reachable, try to get hostname and MAC
            let hostname = get_hostname(&ip).await;
            let mac_address = get_mac_address(&ip).await;

            return Some(DiscoveredDevice {
                ip_address: ip,
                mac_address,
                hostname,
                status: "Online".to_string(),
            });
        }
    }

    None
}

// Get hostname function
async fn get_hostname(ip: &str) -> Option<String> {
    if let Ok(output) = Command::new("nslookup").arg(ip).output().await {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse nslookup output for hostname
            if let Some(line) = output_str.lines().find(|line| line.contains("name =")) {
                if let Some(hostname) = line.split("name = ").nth(1) {
                    return Some(hostname.trim_end_matches('.').to_string());
                }
            }
        }
    }
    None
}

// Get MAC address function
async fn get_mac_address(ip: &str) -> Option<String> {
    // Try to get MAC from ARP table
    if let Ok(output) = Command::new("arp").args(&["-n", ip]).output().await {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mac_regex = Regex::new(r"([0-9a-fA-F]{2}[:-]){5}([0-9a-fA-F]{2})").unwrap();

            if let Some(mac_match) = mac_regex.find(&output_str) {
                return Some(mac_match.as_str().to_string());
            }
        }
    }
    None
}

// Generate config YAML function
pub async fn generate_config_yaml(
    current_config: &crate::config::Config,
    selected_devices: &[DiscoveredDevice],
) -> String {
    let mut updated_config = current_config.clone();

    let existing_macs: std::collections::HashSet<String> = updated_config
        .devices
        .iter()
        .map(|d| d.mac_address.clone())
        .collect();

    for device in selected_devices {
        if let Some(mac_address) = &device.mac_address {
            if !existing_macs.contains(mac_address) {
                let new_device = crate::config::Device {
                    name: device
                        .hostname
                        .clone()
                        .unwrap_or_else(|| format!("New-Device-{}", mac_address.replace(":", ""))),
                    mac_address: mac_address.clone(),
                    ip_address: device.ip_address.clone(),
                };
                updated_config.devices.push(new_device);
            }
        }
    }

    // Add comments for devices without MAC addresses
    let mut yaml_string = serde_yaml::to_string(&updated_config).unwrap_or_default();
    for device in selected_devices {
        if device.mac_address.is_none() {
            let comment = format!(
                "\n# Device '{}' ({}) could not be added because it is missing a MAC address.",
                device.hostname.as_deref().unwrap_or("Unknown"),
                device.ip_address
            );
            yaml_string.push_str(&comment);
        }
    }

    yaml_string
}

// Function to create and configure the Axum router
pub fn app_router(app_state: AppState) -> Router {
    // Accept the single AppState
    Router::new()
        .route("/", get(root_handler))
        .route("/hello", get(hello_handler))
        .route("/discovery", get(discovery_handler))
        .route("/discovery/scan", post(discovery_scan_handler))
        .route("/discovery/generate-config", post(generate_config_handler))
        .route("/discovery/download-config", get(download_config_handler))
        .route("/wake/:device_name", post(wake_device_handler))
        .route("/ping/:device_name", get(ping_device_handler))
        .route("/refresh-all", get(refresh_all_handler))
        // Assets service will be added by main.rs
        .with_state(app_state) // Use with_state to make AppState available to handlers
}
