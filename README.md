# Wololo - Wake On LAN Management Tool

<div align="center">
  <img src="assets/logo/logo_640x640.png" alt="Wololo Logo" width="150" height="150">
</div>

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A simple and efficient web-based Wake On LAN (WoL) management tool built with Rust, designed for homelab environments. Wololo provides an intuitive interface to manage and wake up devices on your local network.

## Features

- 🌐 **Web-based Interface**: Clean, responsive UI built with HTMX and Tailwind CSS
- 🔧 **YAML Configuration**: Simple device management through configuration files
- 🔍 **Network Discovery**: Automatic device discovery with config generation
- 📊 **Real-time Status**: Live device status monitoring with ping functionality
- ⏱️ **Configurable Sync**: Automatic periodic status updates
- 🚀 **Fast & Lightweight**: Built with Rust for optimal performance
- 🏠 **Homelab Ready**: Perfect for self-hosted environments
- 📱 **Mobile Friendly**: Responsive design works on all devices

## Quick Start

### Prerequisites

- Docker (recommended) or Rust 1.70.0+ for building from source
- Network devices with Wake On LAN support enabled

### Docker Installation (Recommended)

1. Pull the container image:
```bash
docker pull ghcr.io/sharmashobhit/wololo:latest
```

2. Create a `config.yaml` file (see Configuration section below)

3. Run the container:
```bash
docker run -d \
  --name wololo \
  -p 3000:3000 \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/sharmashobhit/wololo:latest
```

4. Open your browser and navigate to `http://localhost:3000`

For more deployment options, see [Deployment Guide](docs/DEPLOYMENT.md) and [Container Guide](docs/CONTAINER.md).

### Building from Source

If you prefer to build from source:

1. Clone and build:
```bash
git clone https://github.com/sharmashobhit/wololo.git
cd wololo
cargo build --release
```

2. Configure your devices in `config.yaml` (see Configuration section)

3. Run:
```bash
cargo run --release
```

For development setup and technical details, see [Developer Guide](docs/GUIDE.md).

## Configuration

The application uses a YAML configuration file (`config.yaml`) to manage devices and server settings:

```yaml
server:
  ip: "0.0.0.0"
  port: 3000
  external_url: "http://localhost:3000"

sync:
  enabled: true
  interval_seconds: 30  # Auto-refresh device status every 30 seconds

devices:
  - name: "Device Name"
    mac_address: "XX:XX:XX:XX:XX:XX"
    ip_address: "192.168.1.XXX"
```

### Configuration Options

- `server.ip`: IP address to bind the server (default: `127.0.0.1`)
- `server.port`: Port to run the server (default: `3000`)
- `server.external_url`: External URL for the application (default: `http://127.0.0.1:3000`)
- `sync.enabled`: Enable/disable automatic device status refresh (default: `true`)
- `sync.interval_seconds`: Interval for automatic refresh in seconds (default: `60`)
- `devices`: List of devices to manage
  - `name`: Friendly name for the device
  - `mac_address`: MAC address of the device (required for WoL, format: `XX:XX:XX:XX:XX:XX`)
  - `ip_address`: IP address of the device

See `config-examples.yaml` for additional configuration examples.

## Device Discovery

Wololo includes a powerful network discovery feature to automatically find devices on your network:

1. **Navigate to Discovery**: Click the "Discovery" tab in the web interface
2. **Start Network Scan**: Click "Start Network Scan" to discover devices
3. **Review Results**: Found devices will be displayed with their status, IP, MAC, and hostname
4. **Generate Config**: Select desired devices and click "Generate Config"
5. **Download**: Download the updated `config.yaml` file with discovered devices

### Discovery Features

- **Automatic Network Detection**: Scans your local network subnets
- **Device Information**: Retrieves IP addresses, MAC addresses, and hostnames
- **Status Checking**: Shows which devices are currently online/offline
- **Selective Addition**: Choose which discovered devices to include
- **Config Integration**: Merges with existing configuration seamlessly

## Usage

### Dashboard

The main dashboard displays all configured devices with their current status. You can:
- View device online/offline status
- Wake devices with a single click
- Manually refresh device status
- See auto-refresh status if enabled

### Waking Devices

Click the "Wake" button next to any device to send a Wake On LAN packet. The device should power on if WoL is properly configured in its BIOS/UEFI settings.

### Device Status

Device status is checked via ping. Green indicates online, red indicates offline. Status refreshes automatically if sync is enabled, or manually via the "Refresh All" button.

## Deployment

Wololo can be deployed in various ways:

- **Docker**: Containerized deployment (recommended)
- **Systemd**: Linux service deployment
- **Reverse Proxy**: Behind Nginx, Traefik, or similar

For detailed deployment instructions, see [Deployment Guide](docs/DEPLOYMENT.md) and [Container Guide](docs/CONTAINER.md).

## Documentation

- **[Developer Guide](docs/GUIDE.md)** - Technical documentation, API reference, and development setup
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Deployment options and configurations
- **[Container Guide](docs/CONTAINER.md)** - Container-specific documentation
- **[Testing Guide](TESTING.md)** - Testing documentation
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute to the project

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [wol-rs](https://crates.io/crates/wol-rs) for Wake On LAN functionality
- [Axum](https://github.com/tokio-rs/axum) for the web framework
- [HTMX](https://htmx.org/) for dynamic web interactions
- [Tailwind CSS](https://tailwindcss.com/) for styling

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/sharmashobhit/wololo/issues) on GitHub.