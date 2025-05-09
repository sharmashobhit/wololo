use serde::{Deserialize, Serialize};
use std::fs;

// Struct for individual device configuration
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Device {
    pub name: String,
    pub mac_address: String,
    pub ip_address: String,
}

// Functions to provide default values for ServerConfig
fn default_ip() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_external_url() -> String {
    format!("http://{}:{}", default_ip(), default_port())
}

// Struct for server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_ip")] // Added default for ip
    pub ip: String,
    #[serde(default = "default_port")] // Added default for port
    pub port: u16,
    #[serde(default = "default_external_url")] // Added default for external_url
    pub external_url: String,
    // You can add other server-specific settings here later
}

// Implement Default for ServerConfig so #[serde(default)] on Config.server works
impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            ip: default_ip(),
            port: default_port(),
            external_url: default_external_url(), // Use the default function here
        }
    }
}

// Main configuration struct
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)] // If the whole server section is missing, use ServerConfig::default()
    pub server: ServerConfig,
    pub devices: Vec<Device>,
}

// Function to load and parse config
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    Ok(config)
}
