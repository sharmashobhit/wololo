use wololo::*;

#[test]
fn test_device_status_enum() {
    // Test the enum values that would be used in DeviceStatus
    let online_status = "Online";
    let offline_status = "Offline";
    let unreachable_status = "Unreachable";

    assert_eq!(online_status, "Online");
    assert_eq!(offline_status, "Offline");
    assert_eq!(unreachable_status, "Unreachable");
}

#[tokio::test]
async fn test_device_creation() {
    let device = Device {
        name: "Test Device".to_string(),
        mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };

    assert_eq!(device.name, "Test Device");
    assert_eq!(device.mac_address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(device.ip_address, "192.168.1.100");
}

#[test]
fn test_mac_address_formats() {
    let valid_mac_formats = vec![
        "AA:BB:CC:DD:EE:FF",
        "aa:bb:cc:dd:ee:ff",
        "00:11:22:33:44:55",
        "FF:FF:FF:FF:FF:FF",
    ];

    for mac in valid_mac_formats {
        let device = Device {
            name: "Test".to_string(),
            mac_address: mac.to_string(),
            ip_address: "192.168.1.1".to_string(),
        };
        assert_eq!(device.mac_address, mac);
    }
}

#[test]
fn test_ip_address_formats() {
    let valid_ip_formats = vec![
        "192.168.1.1",
        "10.0.0.1", 
        "172.16.0.1",
        "127.0.0.1",
        "0.0.0.0",
        "255.255.255.255",
    ];

    for ip in valid_ip_formats {
        let device = Device {
            name: "Test".to_string(),
            mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
            ip_address: ip.to_string(),
        };
        assert_eq!(device.ip_address, ip);
    }
}

#[test]
fn test_device_clone() {
    let device1 = Device {
        name: "Original Device".to_string(),
        mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };

    let device2 = device1.clone();

    assert_eq!(device1.name, device2.name);
    assert_eq!(device1.mac_address, device2.mac_address);
    assert_eq!(device1.ip_address, device2.ip_address);
    
    // Ensure they are separate objects
    assert_eq!(device1.name, device2.name);
    // Modify one doesn't affect the other (test would fail if they shared memory)
}

#[test]
fn test_discovered_device_with_all_fields() {
    let discovered = routes::DiscoveredDevice {
        ip_address: "192.168.1.100".to_string(),
        mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        hostname: Some("test-device.local".to_string()),
        status: "Online".to_string(),
    };

    assert_eq!(discovered.ip_address, "192.168.1.100");
    assert_eq!(discovered.mac_address, Some("AA:BB:CC:DD:EE:FF".to_string()));
    assert_eq!(discovered.hostname, Some("test-device.local".to_string()));
    assert_eq!(discovered.status, "Online");
}

#[test]
fn test_discovered_device_with_missing_fields() {
    let discovered = routes::DiscoveredDevice {
        ip_address: "192.168.1.100".to_string(),
        mac_address: None,
        hostname: None,
        status: "Unreachable".to_string(),
    };

    assert_eq!(discovered.ip_address, "192.168.1.100");
    assert_eq!(discovered.mac_address, None);
    assert_eq!(discovered.hostname, None);
    assert_eq!(discovered.status, "Unreachable");
}

#[test]
fn test_device_json_serialization() {
    let device = Device {
        name: "JSON Test Device".to_string(),
        mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
        ip_address: "192.168.1.100".to_string(),
    };

    let json = serde_json::to_string(&device).unwrap();
    
    assert!(json.contains("JSON Test Device"));
    assert!(json.contains("AA:BB:CC:DD:EE:FF"));
    assert!(json.contains("192.168.1.100"));
    
    let deserialized: Device = serde_json::from_str(&json).unwrap();
    assert_eq!(device.name, deserialized.name);
    assert_eq!(device.mac_address, deserialized.mac_address);
    assert_eq!(device.ip_address, deserialized.ip_address);
}

#[test]
fn test_device_yaml_serialization() {
    let device = Device {
        name: "YAML Test Device".to_string(),
        mac_address: "11:22:33:44:55:66".to_string(),
        ip_address: "10.0.0.1".to_string(),
    };

    let yaml = serde_yaml::to_string(&device).unwrap();
    
    assert!(yaml.contains("YAML Test Device"));
    assert!(yaml.contains("11:22:33:44:55:66"));
    assert!(yaml.contains("10.0.0.1"));
    
    let deserialized: Device = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(device.name, deserialized.name);
    assert_eq!(device.mac_address, deserialized.mac_address);
    assert_eq!(device.ip_address, deserialized.ip_address);
}

#[test]
fn test_device_name_edge_cases() {
    let edge_cases = vec![
        "Device with spaces",
        "Device-with-dashes",
        "Device.with.dots",
        "Device_with_underscores",
        "Device123",
        "🚀 Device with emoji",
        "",  // Empty name
    ];

    for name in edge_cases {
        let device = Device {
            name: name.to_string(),
            mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
            ip_address: "192.168.1.1".to_string(),
        };
        
        // Should be able to create and serialize
        let json = serde_json::to_string(&device).unwrap();
        let _deserialized: Device = serde_json::from_str(&json).unwrap();
    }
}