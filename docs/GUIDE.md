# Developer Guide

This guide provides technical documentation for developers working on or contributing to Wololo.

## Table of Contents

- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Project Structure](#project-structure)
- [API Endpoints](#api-endpoints)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Code Organization](#code-organization)
- [Build System](#build-system)

## Architecture

Wololo follows a modular architecture with clear separation of concerns:

### Components

- **Configuration Layer** (`src/config.rs`): Handles YAML configuration parsing and validation
- **Route Handlers** (`src/routes.rs`): HTTP endpoint handlers and business logic
- **Frontend Templates** (`frontend/`): Handlebars templates for UI rendering
- **Static Assets** (`assets/`): Embedded JavaScript and CSS libraries

### Request Flow

1. HTTP request received by Axum router
2. Route handler extracts state and parameters
3. Business logic processes the request
4. Response generated (HTML template or JSON)
5. Response sent to client

### State Management

The application uses `AppState` to share:

- Configuration (`Config`)
- Template engine (`Handlebars`)
- Discovered devices cache (`HashMap<String, Vec<DiscoveredDevice>>`)

## Technology Stack

### Backend

- **Rust**: Systems programming language for performance and safety
- **Axum 0.7**: Modern, ergonomic web framework
- **Tokio**: Async runtime for concurrent operations
- **Handlebars 6.3.2**: Server-side templating engine
- **Serde**: Serialization/deserialization framework
- **serde_yaml**: YAML configuration parsing

### Frontend

- **HTMX**: Dynamic HTML interactions without JavaScript
- **Tailwind CSS**: Utility-first CSS framework
- **Handlebars**: Client-side template rendering (via server)

### Network & WoL

- **wol-rs 1.1.0**: Wake On LAN packet generation
- **network-interface 1.1.4**: Network interface detection
- **ipnet 2.10.1**: IP network calculations
- **eui48 1.1.0**: MAC address parsing

### Utilities

- **rust-embed 8**: Embed static files in binary
- **axum-embed 0.1.0**: Serve embedded files
- **regex 1**: Pattern matching for network discovery
- **futures 0.3**: Async utilities

## Project Structure

```
wololo/
├── src/
│   ├── main.rs          # Application entry point, server initialization
│   ├── lib.rs           # Library exports for testing
│   ├── config.rs        # Configuration structs and loading
│   └── routes.rs        # HTTP route handlers and business logic
├── frontend/
│   ├── index.html       # Main dashboard template (Handlebars)
│   └── discovery.html   # Network discovery template (Handlebars)
├── assets/
│   ├── logo/            # Application logos (various sizes)
│   ├── htmx.min.js      # HTMX library (embedded)
│   └── tailwind.min.js   # Tailwind CSS (embedded)
├── tests/               # Test suites
│   ├── config_tests.rs           # Configuration parsing tests
│   ├── device_status_tests.rs    # Device status logic tests
│   ├── discovery_tests.rs        # Network discovery tests
│   ├── integration_tests.rs      # Integration tests
│   └── route_tests.rs            # HTTP route tests
├── docs/                # Documentation
│   ├── GUIDE.md         # This file
│   ├── CONTAINER.md     # Container deployment guide
│   └── DEPLOYMENT.md    # Deployment options
├── scripts/             # Build and release scripts
│   └── release.sh       # Release automation
├── config.yaml          # Default configuration file
├── config-examples.yaml # Configuration examples
├── Dockerfile           # Multi-stage container build
├── Makefile             # Test management commands
├── Cargo.toml           # Rust dependencies and metadata
├── Cargo.lock           # Dependency lock file
└── README.md            # End-user documentation
```

## API Endpoints

### Web Pages

- `GET /` - Main dashboard page

  - Returns: Rendered Handlebars template with device list
  - Template: `index.html`

- `GET /discovery` - Network discovery page
  - Returns: Rendered Handlebars template for discovery UI
  - Template: `discovery.html`

### Device Management

- `POST /wake/:device_name` - Wake a device by name

  - Parameters: `device_name` (URL path parameter)
  - Returns: HTML fragment with success/error message
  - Behavior: Sends Wake On LAN magic packet to device's broadcast address

- `GET /ping/:device_name` - Check device status

  - Parameters: `device_name` (URL path parameter)
  - Returns: HTML fragment with device status (online/offline)
  - Behavior: Pings device IP address to check connectivity

- `GET /refresh-all` - Refresh status of all devices
  - Returns: HTML fragment with updated device list
  - Behavior: Pings all configured devices and returns status

### Network Discovery

- `POST /discovery/scan` - Start network scan

  - Returns: HTML fragment with discovered devices
  - Behavior: Scans local network subnets, discovers devices via ARP

- `POST /discovery/generate-config` - Generate config from discovered devices

  - Body: Form data with selected device names
  - Returns: HTML fragment with generated config preview
  - Behavior: Merges selected devices with existing config

- `GET /discovery/download-config` - Download generated config file
  - Returns: YAML file download
  - Behavior: Returns merged configuration as YAML file

### Health & Utilities

- `GET /hello` - Health check endpoint
  - Returns: Simple HTML response
  - Use: Container health checks, monitoring

### Static Assets

- `GET /assets/*` - Serve embedded static files
  - Serves: JavaScript, CSS, images from embedded assets

## Development Setup

### Prerequisites

- Rust 1.70.0 or higher
- Cargo (comes with Rust)
- Git
- Network access for testing WoL functionality

### Initial Setup

1. Clone the repository:

```bash
git clone https://github.com/sharmashobhit/wololo.git
cd wololo
```

2. Install dependencies (automatically via Cargo):

```bash
cargo build
```

3. Create a development config:

```bash
cp config-examples.yaml config.yaml
# Edit config.yaml with your devices
```

4. Run the development server:

```bash
cargo run
```

### Development Workflow

1. Make changes to source code
2. Run tests: `make test` or `cargo test`
3. Format code: `cargo fmt`
4. Check linting: `cargo clippy`
5. Test manually: `cargo run` and visit `http://localhost:3000`

### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt --check
```

### Linting

```bash
# Run clippy with warnings as errors
cargo clippy -- -D warnings

# Run clippy with all lints
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

### Test Categories

Wololo uses a tiered testing approach:

1. **Unit Tests** (`make test-unit`): Fast, isolated tests

   - Configuration parsing
   - Data structure validation
   - Helper function logic

2. **Integration Tests** (`make test-integration`): Component interaction

   - Route handler behavior
   - State management
   - Error handling

3. **E2E Tests** (`make test-e2e`): Full system tests
   - Network operations (ping, WoL)
   - Discovery workflows
   - Real HTTP requests

### Running Tests

```bash
# Fast tests (unit + integration, no network)
make test

# Unit tests only
make test-unit

# Integration tests only
make test-integration

# E2E tests (requires network)
make test-e2e

# All tests
make test-all

# With verbose output
cargo test --verbose

# Specific test file
cargo test --test config_tests
```

### Test Structure

Tests are organized in `tests/` directory:

- Each file focuses on a specific module/feature
- E2E tests are gated behind `#[cfg(feature = "e2e-tests")]`
- Tests use `tokio-test` for async testing

### Writing Tests

Example unit test:

```rust
#[test]
fn test_config_loading() {
    let config = load_config_from_path("test-config.yaml").unwrap();
    assert_eq!(config.devices.len(), 2);
}
```

Example E2E test:

```rust
#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_wake_device() {
    // Test network operation
}
```

### Test Coverage

Generate coverage report:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
make test-coverage
```

## Code Organization

### Module Structure

- **`main.rs`**: Application bootstrap, server initialization
- **`lib.rs`**: Library exports for testing
- **`config.rs`**: Configuration data structures and loading
- **`routes.rs`**: HTTP handlers and business logic

### Key Data Structures

#### `Config`

```rust
pub struct Config {
    pub server: ServerConfig,
    pub sync: SyncConfig,
    pub devices: Vec<Device>,
}
```

#### `Device`

```rust
pub struct Device {
    pub name: String,
    pub mac_address: String,
    pub ip_address: String,
}
```

#### `AppState`

```rust
pub struct AppState {
    pub config: Config,
    pub handlebars: Arc<Handlebars<'static>>,
    pub discovered_devices: Arc<Mutex<HashMap<String, Vec<DiscoveredDevice>>>>,
}
```

### Error Handling

- Configuration errors: Return `Result<Config, Box<dyn Error>>`
- Route handlers: Return `impl IntoResponse` with appropriate status codes
- Network operations: Handle timeouts and connection errors gracefully

### Async Patterns

- All route handlers are `async fn`
- Use `tokio::process::Command` for external commands (ping, arp)
- Use `futures::join_all` for concurrent operations

## Build System

### Cargo Features

- `default`: Standard build
- `e2e-tests`: Enable end-to-end tests that require network access

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run in release mode
cargo run --release

# Build with specific features
cargo build --features e2e-tests
```

### Release Process

1. Update version in `Cargo.toml`
2. Create git tag: `git tag v1.0.0`
3. Run release script: `./scripts/release.sh`
4. Push tag: `git push --tags`

### Container Build

The Dockerfile uses multi-stage builds:

1. **chef**: Dependency planning
2. **planner**: Recipe generation
3. **builder**: Compilation
4. **runtime**: Final minimal image

Build container:

```bash
docker build -t wololo:local .
```

## Additional Resources

- [Contributing Guide](../CONTRIBUTING.md) - Contribution guidelines
- [Testing Guide](../TESTING.md) - Detailed testing documentation
- [Deployment Guide](DEPLOYMENT.md) - Deployment options
- [Container Guide](CONTAINER.md) - Container-specific documentation
