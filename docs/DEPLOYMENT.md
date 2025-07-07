# Deployment Guide

This guide covers different deployment options for Wololo.

## Docker Deployment

### Using Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:
```yaml
version: '3.8'

services:
  wololo:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config.yaml:/app/config.yaml:ro
    restart: unless-stopped
    network_mode: host  # Required for Wake On LAN
```

2. Create a `Dockerfile`:
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY frontend ./frontend
COPY assets ./assets

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/wololo .
COPY config.yaml .

EXPOSE 8080

CMD ["./wololo"]
```

3. Deploy:
```bash
docker-compose up -d
```

## Systemd Service

For running as a system service on Linux:

1. Create service file `/etc/systemd/system/wololo.service`:
```ini
[Unit]
Description=Wololo Wake On LAN Service
After=network.target

[Service]
Type=simple
User=wololo
Group=wololo
WorkingDirectory=/opt/wololo
ExecStart=/opt/wololo/wololo
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

2. Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable wololo
sudo systemctl start wololo
```

## Reverse Proxy Setup

### Nginx

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Traefik

```yaml
# docker-compose.yml
version: '3.8'

services:
  wololo:
    build: .
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.wololo.rule=Host(`wololo.your-domain.com`)"
      - "traefik.http.routers.wololo.entrypoints=websecure"
      - "traefik.http.routers.wololo.tls.certresolver=letsencrypt"
    volumes:
      - ./config.yaml:/app/config.yaml:ro
    restart: unless-stopped
    network_mode: host
```

## Security Considerations

- **Network Access**: Wololo needs to be on the same network as target devices
- **Firewall**: Ensure port 8080 is accessible
- **Authentication**: Consider adding authentication for production use
- **HTTPS**: Use a reverse proxy with SSL/TLS certificates

## Monitoring

### Health Check

Add a health check endpoint by modifying `src/routes.rs`:

```rust
// Add to your routes
.route("/health", get(health_check))

async fn health_check() -> &'static str {
    "OK"
}
```

### Logs

View logs:
```bash
# Docker
docker-compose logs -f wololo

# Systemd
journalctl -u wololo -f
```

## Performance Tuning

### Configuration

Adjust `config.yaml` for your environment:
```yaml
server:
  ip: "0.0.0.0"
  port: 8080
  external_url: "https://your-domain.com"
  
# Add more devices as needed
devices:
  - name: "Device 1"
    mac_address: "XX:XX:XX:XX:XX:XX"
    ip_address: "192.168.1.100"
```

### Resource Limits

For Docker:
```yaml
services:
  wololo:
    # ... other config
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
        reservations:
          cpus: '0.25'
          memory: 128M
```

## Backup

Important files to backup:
- `config.yaml` - Device configurations
- Application logs
- Any custom modifications

## Updates

### Manual Updates

1. Stop the service
2. Backup configuration
3. Update the binary
4. Restart the service

### Docker Updates

```bash
docker-compose pull
docker-compose up -d
```

## Troubleshooting

### Common Issues

1. **Wake On LAN not working**:
   - Ensure devices have WoL enabled in BIOS/UEFI
   - Check network configuration
   - Verify MAC addresses are correct

2. **Connection refused**:
   - Check if port 8080 is available
   - Verify firewall settings
   - Check service status

3. **Configuration errors**:
   - Validate YAML syntax
   - Check file permissions
   - Review logs for specific errors

### Debug Mode

Run with verbose logging:
```bash
RUST_LOG=debug ./wololo
```