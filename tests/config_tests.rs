use std::io::Write;
use tempfile::NamedTempFile;
use wololo::config::*;

#[test]
fn test_default_server_config() {
    let config = ServerConfig::default();
    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert_eq!(config.external_url, "http://127.0.0.1:3000");
}

#[test]
fn test_default_sync_config() {
    let config = SyncConfig::default();
    assert_eq!(config.enabled, true);
    assert_eq!(config.interval_seconds, 60);
}

#[test]
fn test_load_valid_config() {
    let config_content = r#"
server:
  ip: "0.0.0.0"
  port: 8080
  external_url: "http://example.com:8080"

sync:
  enabled: false
  interval_seconds: 120

devices:
  - name: "Test Device"
    mac_address: "AA:BB:CC:DD:EE:FF"
    ip_address: "192.168.1.100"
  - name: "Another Device"
    mac_address: "11:22:33:44:55:66"
    ip_address: "192.168.1.101"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();

    let config: Config = serde_yaml::from_str(config_content).unwrap();

    // Test server config
    assert_eq!(config.server.ip, "0.0.0.0");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.external_url, "http://example.com:8080");

    // Test sync config
    assert_eq!(config.sync.enabled, false);
    assert_eq!(config.sync.interval_seconds, 120);

    // Test devices
    assert_eq!(config.devices.len(), 2);
    assert_eq!(config.devices[0].name, "Test Device");
    assert_eq!(config.devices[0].mac_address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(config.devices[0].ip_address, "192.168.1.100");
}

#[test]
fn test_load_minimal_config() {
    let config_content = r#"
devices:
  - name: "Only Device"
    mac_address: "AA:BB:CC:DD:EE:FF"
    ip_address: "192.168.1.100"
"#;

    let config: Config = serde_yaml::from_str(config_content).unwrap();

    // Should use defaults for missing sections
    assert_eq!(config.server.ip, "127.0.0.1");
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.sync.enabled, true);
    assert_eq!(config.sync.interval_seconds, 60);
    assert_eq!(config.devices.len(), 1);
}

#[test]
fn test_load_nonexistent_config() {
    // Make sure the file doesn't exist
    std::fs::remove_file("config.yaml").ok();

    let result = load_config();
    assert!(result.is_err());
}

#[test]
fn test_device_serialization() {
    let device = Device {
        name: "Test Device".to_string(),
        mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };

    let json = serde_json::to_string(&device).unwrap();
    let deserialized: Device = serde_json::from_str(&json).unwrap();

    assert_eq!(device.name, deserialized.name);
    assert_eq!(device.mac_address, deserialized.mac_address);
    assert_eq!(device.ip_address, deserialized.ip_address);
}

#[test]
fn test_config_with_empty_devices() {
    let config_content = r#"
server:
  ip: "localhost"
  port: 3000
  external_url: "http://localhost:3000"

sync:
  enabled: true
  interval_seconds: 30

devices: []
"#;

    let config: Config = serde_yaml::from_str(config_content).unwrap();

    assert_eq!(config.devices.len(), 0);
    assert_eq!(config.server.ip, "localhost");
    assert_eq!(config.sync.interval_seconds, 30);
}
