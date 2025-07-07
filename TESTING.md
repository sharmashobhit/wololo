# Testing Guide for Wololo

This document explains how to run different types of tests in the wololo project.

## Test Categories

### Unit Tests
Fast tests that don't require network operations or external dependencies.
- Configuration parsing and validation
- Device structure serialization
- Basic HTTP route handlers

### Integration Tests
Medium-speed tests that test component interactions but avoid heavy network operations.
- Application state management
- Configuration validation
- Error handling scenarios

### E2E Tests
Slow tests that involve actual network operations and system commands.
- Wake-on-LAN packet sending
- Network device discovery
- Ping operations
- Large-scale device management

## Running Tests

### Quick Development Testing (Default)
```bash
# Run unit and integration tests only (fast)
make test
# or
cargo test --lib
cargo test --test config_tests
cargo test --test device_status_tests
cargo test --test discovery_tests
```

### Unit Tests Only
```bash
make test-unit
```

### Integration Tests Only
```bash
make test-integration
```

### E2E Tests Only
```bash
make test-e2e
# or
cargo test --features e2e-tests
```

### All Tests (Including E2E)
```bash
make test-all
# or
cargo test --features e2e-tests
```

## Test Commands Reference

| Command | Description | Speed | Network Required |
|---------|-------------|-------|------------------|
| `make test` | Unit + Integration tests | Fast | No |
| `make test-unit` | Unit tests only | Very Fast | No |
| `make test-integration` | Integration tests only | Medium | No |
| `make test-e2e` | E2E tests only | Slow | Yes |
| `make test-all` | All tests | Slow | Yes |

## CI/CD Recommendations

### For Pull Requests
```bash
make test  # Run fast tests for quick feedback
```

### For Main Branch / Releases
```bash
make test-all  # Run all tests including E2E
```

### For Local Development
```bash
make test-unit  # Run only unit tests while developing
```

## Test Structure

```
tests/
├── config_tests.rs        # Unit tests for configuration
├── device_status_tests.rs # Unit tests for device structures
├── discovery_tests.rs     # Unit tests for discovery logic
├── route_tests.rs         # Unit/E2E tests for HTTP routes
└── integration_tests.rs   # Integration/E2E tests
```

## Adding New Tests

### Unit Tests
Add `#[test]` attribute without any additional configuration.

### E2E Tests
Add both `#[test]` and `#[cfg(feature = "e2e-tests")]` attributes:

```rust
#[tokio::test]
#[cfg(feature = "e2e-tests")]
async fn test_network_operation() {
    // Test code that involves network operations
}
```

## Test Coverage

To generate test coverage reports:
```bash
# Install cargo-tarpaulin first
cargo install cargo-tarpaulin

# Generate coverage report
make test-coverage
```

## Troubleshooting

### E2E Tests Fail
- Ensure you have network connectivity
- Check that ping command is available
- Verify arp command is available (for MAC address discovery)

### Tests Timeout
- E2E tests involving network scans may take longer
- Consider increasing test timeout if needed
- Use `make test` instead of `make test-all` for faster feedback

### Permission Issues
- Some network operations may require elevated permissions
- Wake-on-LAN tests might need special network configuration