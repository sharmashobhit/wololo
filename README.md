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

- Rust 1.70.0 or higher
- Network devices with Wake On LAN support enabled

### Installation

1. Clone the repository:
```bash
git clone https://github.com/sharmashobhit/wololo.git
cd wololo
```

2. Build the project:
```bash
cargo build --release
```

3. Configure your devices in `config.yaml`:
```yaml
devices:
  - name: "Main PC"
    mac_address: "74:56:3c:76:23:1f"
    ip_address: "192.168.29.185"
  - name: "Living Room PC"
    mac_address: "A0:B1:C2:D3:E4:F5"
    ip_address: "192.168.1.101"
```

4. Run the application:
```bash
cargo run
```

5. Open your browser and navigate to `http://localhost:3000`

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

- `server.ip`: IP address to bind the server (default: 0.0.0.0)
- `server.port`: Port to run the server (default: 3000)
- `server.external_url`: External URL for the application
- `sync.enabled`: Enable/disable automatic device status refresh (default: true)
- `sync.interval_seconds`: Interval for automatic refresh in seconds (default: 60)
- `devices`: List of devices to manage
  - `name`: Friendly name for the device
  - `mac_address`: MAC address of the device (required for WoL)
  - `ip_address`: IP address of the device

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

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: HTMX for dynamic interactions
- **Styling**: Tailwind CSS
- **Templating**: Handlebars
- **Configuration**: YAML with serde

## Project Structure

```
wololo/
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration handling
│   └── routes.rs        # HTTP route handlers
├── frontend/
│   └── index.html       # Main HTML template
├── assets/
│   ├── htmx.min.js     # HTMX library
│   └── tailwind.min.js # Tailwind CSS
├── config.yaml         # Device configuration
└── Cargo.toml          # Rust dependencies
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on how to get started.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [wol-rs](https://crates.io/crates/wol-rs) for Wake On LAN functionality
- [Axum](https://github.com/tokio-rs/axum) for the web framework
- [HTMX](https://htmx.org/) for dynamic web interactions
- [Tailwind CSS](https://tailwindcss.com/) for styling

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/sharmashobhit/wololo/issues) on GitHub.