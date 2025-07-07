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

use serde_json::json; // For constructing data for Handlebars - THIS REQUIRES serde_json in Cargo.toml
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tokio::process::Command;
use wol::*;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use ipnet::Ipv4Net;
use futures::future::join_all;
use regex::Regex;
use axum::extract::Form;

use crate::config::Config;
use handlebars::Handlebars;
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

// Discovery page handler
async fn discovery_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let data = json!({
        "devices": &app_state.config.devices,
        "device_count": app_state.config.devices.len(),
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
    
    let mut devices_html = String::new();
    
    if discovered_devices.is_empty() {
        devices_html = r#"
        <div class="text-center py-8 text-gray-500">
            <svg class="w-12 h-12 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
            </svg>
            <h3 class="text-lg font-medium mb-2">No devices discovered</h3>
            <p>Try running the scan again or check your network connection</p>
        </div>
        "#.to_string();
    } else {
        devices_html.push_str(&format!(
            "<form id=\"discovery-form\">\
                <div class=\"mb-6\">\
                    <div class=\"flex justify-between items-center mb-4\">\
                        <h3 class=\"text-lg font-semibold text-gray-800\">Discovered Devices ({})</h3>\
                        <button type=\"submit\" hx-post=\"/discovery/generate-config\" hx-include=\"#discovery-form\" hx-target=\"#config-download\" class=\"bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-md shadow-sm transition duration-150 ease-in-out flex items-center\">\
                            <svg class=\"w-4 h-4 mr-2\" fill=\"none\" stroke=\"currentColor\" viewBox=\"0 0 24 24\">\
                                <path stroke-linecap=\"round\" stroke-linejoin=\"round\" stroke-width=\"2\" d=\"M12 10v6m0 0l-3-3m3 3l3-3M3 17V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2z\"></path>\
                            </svg>\
                            Generate Config\
                        </button>\
                    </div>",
            discovered_devices.len()
        ));
        
        devices_html.push_str("<div class=\"grid gap-3\">");
        
        for (index, device) in discovered_devices.iter().enumerate() {
            let status_class = match device.status.as_str() {
                "Online" => "bg-green-100 text-green-800",
                "Offline" => "bg-red-100 text-red-800",
                _ => "bg-yellow-100 text-yellow-800",
            };
            
            let device_html = format!(
                "<div class=\"bg-gray-50 rounded-lg border border-gray-200 p-4\">\
                    <div class=\"flex items-center justify-between\">\
                        <div class=\"flex-1\">\
                            <div class=\"flex items-center gap-3 mb-2\">\
                                <h4 class=\"font-medium text-gray-800\">{}</h4>\
                                <span class=\"inline-flex items-center px-2 py-1 rounded-full text-xs font-medium {}\">\
                                    {}\
                                </span>\
                            </div>\
                            <div class=\"text-sm text-gray-600 space-y-1\">\
                                <p><span class=\"font-medium\">IP:</span> {}</p>\
                                <p><span class=\"font-medium\">MAC:</span> {}</p>\
                                <p><span class=\"font-medium\">Hostname:</span> {}</p>\
                            </div>\
                        </div>\
                        <div class=\"flex flex-col gap-2\">\
                            <input type=\"checkbox\" id=\"device-{}\" name=\"selected_devices\" value=\"{}\" class=\"rounded border-gray-300 text-blue-600 focus:ring-blue-500\" checked>\
                            <label for=\"device-{}\" class=\"text-xs text-gray-500\">Include</label>\
                        </div>\
                    </div>\
                </div>",
                device.hostname.as_deref().unwrap_or("Unknown Device"),
                status_class,
                device.status,
                device.ip_address,
                device.mac_address.as_deref().unwrap_or("Unknown"),
                device.hostname.as_deref().unwrap_or("Unknown"),
                index,
                device.ip_address,
                index
            );
            devices_html.push_str(&device_html);
        }
        
        devices_html.push_str("</div>");
        devices_html.push_str("<div id=\"config-download\" class=\"mt-6\"></div>");
        devices_html.push_str("</div>");
        devices_html.push_str("</form>");
    }
    
    Html(devices_html).into_response()
}

// Form data structure for device selection
#[derive(Debug, serde::Deserialize)]
pub struct DeviceSelectionForm {
    pub selected_devices: Option<Vec<String>>,
}

// Config generation handler
async fn generate_config_handler(
    State(app_state): State<AppState>,
    Form(form_data): Form<DeviceSelectionForm>,
) -> impl IntoResponse {
    println!("Generating config with selected devices...");
    
    // Get the discovered devices from storage
    let discovered_devices = {
        let storage = app_state.discovered_devices.lock().await;
        storage.get("latest_scan").cloned().unwrap_or_default()
    };
    
    // Parse selected device IPs and find corresponding devices
    let selected_ips = form_data.selected_devices.unwrap_or_default();
    let selected_devices: Vec<DiscoveredDevice> = discovered_devices
        .into_iter()
        .filter(|device| selected_ips.contains(&device.ip_address))
        .collect();
    
    println!("Selected {} devices for config generation", selected_devices.len());
    
    let config_content = generate_config_yaml(&app_state.config, &selected_devices).await;
    
    // Store the generated config for download
    {
        let mut stored_config = GENERATED_CONFIG.lock().await;
        *stored_config = Some(config_content.clone());
    }
    
    let download_html = format!(
        r#"<div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div class="flex items-center justify-between">
                <div>
                    <h4 class="font-medium text-blue-800">Configuration Generated</h4>
                    <p class="text-sm text-blue-600 mt-1">Updated config.yaml ready for download</p>
                </div>
                <a 
                    href="/discovery/download-config" 
                    download="config.yaml"
                    class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-md shadow-sm transition duration-150 ease-in-out flex items-center">
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m-8 5h16l-1-1V9l-1-1H4l-1 1v10l1 1z"></path>
                    </svg>
                    Download Config
                </a>
            </div>
            <details class="mt-4">
                <summary class="cursor-pointer text-sm text-blue-600 hover:text-blue-800">Preview Configuration</summary>
                <pre class="mt-2 p-3 bg-gray-100 rounded text-xs overflow-x-auto"><code>{}</code></pre>
            </details>
        </div>"#,
        html_escape::encode_text(&config_content)
    );
    
    Html(download_html).into_response()
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
        ("Content-Disposition", "attachment; filename=\"config.yaml\""),
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
        if let Some(addr) = interface.addr.iter().find(|addr| addr.ip().is_ipv4() && !addr.ip().is_loopback()) {
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
    if let Ok(output) = Command::new("nslookup")
        .arg(ip)
        .output()
        .await
    {
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
    if let Ok(output) = Command::new("arp")
        .args(&["-n", ip])
        .output()
        .await
    {
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
pub async fn generate_config_yaml(current_config: &crate::config::Config, selected_devices: &[DiscoveredDevice]) -> String {
    let mut config_content = format!(
        "server:\n  ip: \"{}\"\n  port: {}\n  external_url: \"{}\"\n\n",
        current_config.server.ip,
        current_config.server.port,
        current_config.server.external_url
    );
    
    config_content.push_str(&format!(
        "sync:\n  enabled: {}\n  interval_seconds: {}  # Auto-refresh device status every {} seconds\n\n",
        current_config.sync.enabled,
        current_config.sync.interval_seconds,
        current_config.sync.interval_seconds
    ));
    
    config_content.push_str("devices:\n");
    
    // Add existing devices first
    for device in &current_config.devices {
        config_content.push_str(&format!(
            "  - name: \"{}\"\n    mac_address: \"{}\"\n    ip_address: \"{}\"\n",
            device.name, device.mac_address, device.ip_address
        ));
    }
    
    // Add selected discovered devices
    for device in selected_devices {
        let default_name = format!("Device-{}", device.ip_address);
        let device_name = device.hostname.as_deref().unwrap_or(&default_name);
        let mac_address = device.mac_address.as_deref().unwrap_or("XX:XX:XX:XX:XX:XX");
        
        // Only add if we have a MAC address (required for WoL)
        if device.mac_address.is_some() {
            config_content.push_str(&format!(
                "  - name: \"{}\"\n    mac_address: \"{}\"\n    ip_address: \"{}\"\n",
                device_name, mac_address, device.ip_address
            ));
        } else {
            // Add as a comment if no MAC address available
            config_content.push_str(&format!(
                "  # - name: \"{}\"  # No MAC address found\n  #   mac_address: \"XX:XX:XX:XX:XX:XX\"\n  #   ip_address: \"{}\"\n",
                device_name, device.ip_address
            ));
        }
    }
    
    config_content
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
