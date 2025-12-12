use wololo::*;

#[tokio::test]
async fn test_discovery_device_structure() {
    let device = routes::DiscoveredDevice {
        ip_address: "192.168.1.100".to_string(),
        mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        hostname: Some("test-device".to_string()),
        status: "Online".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&device).unwrap();
    assert!(json.contains("192.168.1.100"));
    assert!(json.contains("AA:BB:CC:DD:EE:FF"));
    assert!(json.contains("test-device"));
    assert!(json.contains("Online"));

    // Test deserialization
    let deserialized: routes::DiscoveredDevice = serde_json::from_str(&json).unwrap();
    assert_eq!(device.ip_address, deserialized.ip_address);
    assert_eq!(device.mac_address, deserialized.mac_address);
    assert_eq!(device.hostname, deserialized.hostname);
    assert_eq!(device.status, deserialized.status);
}

#[tokio::test]
async fn test_config_generation_with_discovered_devices() {
    let current_config = Config {
        server: ServerConfig {
            ip: "0.0.0.0".to_string(),
            port: 3000,
            external_url: "http://localhost:3000".to_string(),
        },
        sync: SyncConfig {
            enabled: true,
            interval_seconds: 30,
        },
        sound: SoundConfig::default(),
        devices: vec![Device {
            name: "Existing Device".to_string(),
            mac_address: "11:22:33:44:55:66".to_string(),
            ip_address: "192.168.1.50".to_string(),
        }],
    };

    let discovered_devices = vec![
        routes::DiscoveredDevice {
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: Some("router".to_string()),
            status: "Online".to_string(),
        },
        routes::DiscoveredDevice {
            ip_address: "192.168.1.101".to_string(),
            mac_address: None, // No MAC address
            hostname: Some("printer".to_string()),
            status: "Online".to_string(),
        },
    ];

    let config_yaml = routes::generate_config_yaml(&current_config, &discovered_devices).await;

    // Check that server config is preserved
    assert!(config_yaml.contains("ip: 0.0.0.0"));
    assert!(config_yaml.contains("port: 3000"));
    assert!(config_yaml.contains("external_url: http://localhost:3000"));

    // Check that sync config is preserved
    assert!(config_yaml.contains("enabled: true"));
    assert!(config_yaml.contains("interval_seconds: 30"));

    // Check that existing device is preserved
    assert!(config_yaml.contains("name: Existing Device"));
    assert!(config_yaml.contains("mac_address: 11:22:33:44:55:66"));
    assert!(config_yaml.contains("ip_address: 192.168.1.50"));

    // Check that discovered device with MAC is added
    assert!(config_yaml.contains("name: router"));
    assert!(config_yaml.contains("mac_address: AA:BB:CC:DD:EE:FF"));
    assert!(config_yaml.contains("ip_address: 192.168.1.100"));

    // Check that discovered device without MAC is commented
    assert!(config_yaml.contains("# Device 'printer' (192.168.1.101) could not be added because it is missing a MAC address."));
}

#[tokio::test]
async fn test_config_generation_no_discovered_devices() {
    let current_config = Config {
        server: ServerConfig {
            ip: "127.0.0.1".to_string(),
            port: 8080,
            external_url: "http://example.com:8080".to_string(),
        },
        sync: SyncConfig {
            enabled: false,
            interval_seconds: 60,
        },
        sound: SoundConfig::default(),
        devices: vec![Device {
            name: "Only Device".to_string(),
            mac_address: "FF:EE:DD:CC:BB:AA".to_string(),
            ip_address: "10.0.0.1".to_string(),
        }],
    };

    let discovered_devices = vec![];
    let config_yaml = routes::generate_config_yaml(&current_config, &discovered_devices).await;

    // Should only contain existing configuration
    assert!(config_yaml.contains("ip: 127.0.0.1"));
    assert!(config_yaml.contains("port: 8080"));
    assert!(config_yaml.contains("enabled: false"));
    assert!(config_yaml.contains("interval_seconds: 60"));
    assert!(config_yaml.contains("name: Only Device"));
    assert!(config_yaml.contains("mac_address: FF:EE:DD:CC:BB:AA"));

    // Should not contain any commented devices
    assert!(!config_yaml.contains("# Device"));
}

#[tokio::test]
async fn test_discovery_with_special_characters() {
    let discovered_devices = vec![
        routes::DiscoveredDevice {
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: Some("device-with-dash".to_string()),
            status: "Online".to_string(),
        },
        routes::DiscoveredDevice {
            ip_address: "192.168.1.101".to_string(),
            mac_address: Some("11:22:33:44:55:66".to_string()),
            hostname: Some("device.with.dots".to_string()),
            status: "Online".to_string(),
        },
    ];

    let current_config = Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        sound: SoundConfig::default(),
        devices: vec![],
    };

    let config_yaml = routes::generate_config_yaml(&current_config, &discovered_devices).await;

    // Check that special characters in hostnames are properly handled
    assert!(config_yaml.contains("name: device-with-dash"));
    assert!(config_yaml.contains("name: device.with.dots"));
}
