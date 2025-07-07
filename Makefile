# Makefile for wololo project test management

# Run unit tests only (fast tests)
.PHONY: test-unit
test-unit:
	cargo test --lib
	cargo test --test config_tests
	cargo test --test device_status_tests
	cargo test --test discovery_tests
	cargo test --test route_tests -- --skip test_wake_device_route_existing_device --skip test_ping_device_route_existing_device --skip test_discovery_scan_route

# Run integration tests only (without E2E features)
.PHONY: test-integration
test-integration:
	cargo test --test integration_tests -- --skip test_full_application_flow --skip test_discovery_workflow --skip test_concurrent_requests --skip test_large_device_list

# Run E2E tests only (slow tests that involve network operations)
.PHONY: test-e2e
test-e2e:
	cargo test --features e2e-tests

# Run all tests except E2E
.PHONY: test
test: test-unit test-integration

# Run all tests including E2E
.PHONY: test-all
test-all:
	cargo test --features e2e-tests

# Run tests with verbose output
.PHONY: test-verbose
test-verbose:
	cargo test --verbose

# Run tests with coverage (requires cargo-tarpaulin)
.PHONY: test-coverage
test-coverage:
	cargo tarpaulin --out Html --output-dir coverage

# Clean test artifacts
.PHONY: clean-test
clean-test:
	cargo clean
	rm -rf coverage/

# Help command
.PHONY: help
help:
	@echo "Available test commands:"
	@echo "  test-unit        - Run unit tests (fast)"
	@echo "  test-integration - Run integration tests (medium speed)"
	@echo "  test-e2e         - Run E2E tests (slow, involves network)"
	@echo "  test             - Run unit + integration tests"
	@echo "  test-all         - Run all tests including E2E"
	@echo "  test-verbose     - Run tests with verbose output"
	@echo "  test-coverage    - Run tests with coverage report"
	@echo "  clean-test       - Clean test artifacts"
	@echo "  help             - Show this help message"