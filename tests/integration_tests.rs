use wololo::*;
use axum::{
    body::Body,
    http::{Request, Method},
};
use tower::ServiceExt;

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_full_application_flow() {
    // Create a test config
    let config = Config {
        server: ServerConfig {
            ip: "127.0.0.1".to_string(),
            port: 3000,
            external_url: "http://localhost:3000".to_string(),
        },
        sync: SyncConfig {
            enabled: true,
            interval_seconds: 30,
        },
        devices: vec![
            Device {
                name: "Test PC".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: "192.168.1.100".to_string(),
            },
        ],
    };

    let app_state = AppState::new_for_test(config);
    let app = routes::app_router(app_state);

    // Test 1: Access main page
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Test 2: Access discovery page
    let request = Request::builder()
        .uri("/discovery")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Test 3: Refresh all devices
    let request = Request::builder()
        .uri("/refresh-all")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Test 4: Ping a device
    let request = Request::builder()
        .uri("/ping/Test%20PC")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Test 5: Wake a device
    let request = Request::builder()
        .method(Method::POST)
        .uri("/wake/Test%20PC")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_discovery_workflow() {
    let app_state = AppState::new_for_test(Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        devices: vec![],
    });
    let app = routes::app_router(app_state);

    // Test 1: Start discovery scan
    let request = Request::builder()
        .method(Method::POST)
        .uri("/discovery/scan")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Test 2: Download config (should work even without scan)
    let request = Request::builder()
        .uri("/discovery/download-config")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    
    // Verify response headers
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/x-yaml");
}

#[tokio::test]
async fn test_error_handling() {
    let app_state = AppState::new_for_test(Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        devices: vec![],
    });
    let app = routes::app_router(app_state);

    // Test 1: Non-existent device ping
    let request = Request::builder()
        .uri("/ping/NonExistentDevice")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    // Test 2: Non-existent device wake
    let request = Request::builder()
        .method(Method::POST)
        .uri("/wake/NonExistentDevice")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);

    // Test 3: Invalid route
    let request = Request::builder()
        .uri("/invalid/route")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_app_state_management() {
    let config = Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        devices: vec![
            Device {
                name: "Device 1".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: "192.168.1.1".to_string(),
            },
            Device {
                name: "Device 2".to_string(),
                mac_address: "11:22:33:44:55:66".to_string(),
                ip_address: "192.168.1.2".to_string(),
            },
        ],
    };

    let app_state = AppState::new_for_test(config);

    // Test config access
    assert_eq!(app_state.config.devices.len(), 2);
    assert_eq!(app_state.config.devices[0].name, "Device 1");
    assert_eq!(app_state.config.devices[1].name, "Device 2");

    // Test discovered devices storage
    let discovered_devices = vec![
        routes::DiscoveredDevice {
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("FF:EE:DD:CC:BB:AA".to_string()),
            hostname: Some("discovered-device".to_string()),
            status: "Online".to_string(),
        },
    ];

    {
        let mut storage = app_state.discovered_devices.lock().await;
        storage.insert("test_scan".to_string(), discovered_devices.clone());
    }

    // Verify storage
    {
        let storage = app_state.discovered_devices.lock().await;
        let stored_devices = storage.get("test_scan").unwrap();
        assert_eq!(stored_devices.len(), 1);
        assert_eq!(stored_devices[0].ip_address, "192.168.1.100");
    }
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_concurrent_requests() {
    let app_state = AppState::new_for_test(Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        devices: vec![
            Device {
                name: "Test Device".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: "192.168.1.100".to_string(),
            },
        ],
    });

    // Create multiple apps to simulate concurrent access
    let app1 = routes::app_router(app_state.clone());
    let app2 = routes::app_router(app_state.clone());
    let app3 = routes::app_router(app_state);

    // Make concurrent requests
    let request1 = Request::builder().uri("/").body(Body::empty()).unwrap();
    let request2 = Request::builder().uri("/discovery").body(Body::empty()).unwrap();
    let request3 = Request::builder().uri("/ping/Test%20Device").body(Body::empty()).unwrap();

    let (response1, response2, response3) = tokio::join!(
        app1.oneshot(request1),
        app2.oneshot(request2),
        app3.oneshot(request3)
    );

    // All should succeed
    assert_eq!(response1.unwrap().status(), axum::http::StatusCode::OK);
    assert_eq!(response2.unwrap().status(), axum::http::StatusCode::OK);
    assert_eq!(response3.unwrap().status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_config_validation() {
    // Test with invalid IP format
    let config_with_invalid_ip = Config {
        server: ServerConfig {
            ip: "invalid.ip.format".to_string(),
            port: 3000,
            external_url: "http://localhost:3000".to_string(),
        },
        sync: SyncConfig::default(),
        devices: vec![],
    };

    // App should still be created (validation happens at runtime)
    let app_state = AppState::new_for_test(config_with_invalid_ip);
    assert_eq!(app_state.config.server.ip, "invalid.ip.format");

    // Test with extreme port values
    let config_with_extreme_port = Config {
        server: ServerConfig {
            ip: "127.0.0.1".to_string(),
            port: 0,
            external_url: "http://localhost:0".to_string(),
        },
        sync: SyncConfig::default(),
        devices: vec![],
    };

    let app_state = AppState::new_for_test(config_with_extreme_port);
    assert_eq!(app_state.config.server.port, 0);
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_large_device_list() {
    // Test with a large number of devices
    let mut devices = Vec::new();
    for i in 0..100 {
        devices.push(Device {
            name: format!("Device {}", i),
            mac_address: format!("AA:BB:CC:DD:EE:{:02X}", i),
            ip_address: format!("192.168.1.{}", i + 1),
        });
    }

    let config = Config {
        server: ServerConfig::default(),
        sync: SyncConfig::default(),
        devices,
    };

    let app_state = AppState::new_for_test(config);
    let app = routes::app_router(app_state);

    // Test refresh all with large device list
    let request = Request::builder()
        .uri("/refresh-all")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}