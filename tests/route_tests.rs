use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
};
use tower::ServiceExt;
use wololo::*; // for `oneshot`

// Helper function to create test app state
fn create_test_app_state() -> AppState {
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
        sound: SoundConfig::default(),
        devices: vec![
            Device {
                name: "Test Device 1".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: "192.168.1.100".to_string(),
            },
            Device {
                name: "Test Device 2".to_string(),
                mac_address: "11:22:33:44:55:66".to_string(),
                ip_address: "192.168.1.101".to_string(),
            },
        ],
    };
    AppState::new_for_test(config)
}

#[tokio::test]
async fn test_root_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_hello_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/hello")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("Hello from Axum"));
}

#[tokio::test]
async fn test_discovery_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/discovery")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_refresh_all_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/refresh-all")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    // Should contain device information
    assert!(body_str.contains("Test Device 1") || body_str.contains("Loading"));
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_wake_device_route_existing_device() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/wake/Test%20Device%201")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return OK (even if WoL fails in test environment)
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    // Should contain success or error message
    assert!(body_str.contains("Wake packet sent") || body_str.contains("Failed to wake"));
}

#[tokio::test]
async fn test_wake_device_route_nonexistent_device() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/wake/Nonexistent%20Device")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    assert!(body_str.contains("not found"));
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_ping_device_route_existing_device() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/ping/Test%20Device%201")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    // Should contain a status indicator
    assert!(
        body_str.contains("Online")
            || body_str.contains("Offline")
            || body_str.contains("Unreachable")
    );
}

#[tokio::test]
async fn test_ping_device_route_nonexistent_device() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/ping/Nonexistent%20Device")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    assert!(body_str.contains("not found"));
}

#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_discovery_scan_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/discovery/scan")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();

    // Should contain either discovered devices or no devices message
    assert!(body_str.contains("Discovered Devices") || body_str.contains("No devices discovered"));
}

#[tokio::test]
async fn test_assets_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/assets/htmx.min.js")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should return the asset file or 404 if not found
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_route() {
    let app_state = create_test_app_state();
    let app = routes::app_router(app_state);

    let request = Request::builder()
        .uri("/invalid/route")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
