# Container Deployment Guide

This document describes how to deploy Wololo using containers.

## Available Container Images

Container images are automatically built and published to GitHub Container Registry when new tags are created.

- **Registry**: `ghcr.io/sharmashobhit/wololo`
- **Tags**: Available tags follow semantic versioning (e.g., `v1.0.0`, `v1.2.3`)

## Quick Start

### Using Docker

```bash
# Pull the latest image
docker pull ghcr.io/sharmashobhit/wololo:latest

# Run the container
docker run -d \
  --name wololo \
  -p 3000:3000 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/sharmashobhit/wololo:latest
```

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  wololo:
    image: ghcr.io/sharmashobhit/wololo:latest
    container_name: wololo
    ports:
      - "3000:3000"
    volumes:
      - ./config.yaml:/app/config.yaml:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/hello"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s
```

Then run:

```bash
docker-compose up -d
```

## Configuration

### Volume Mounts

- **Configuration**: Mount your `config.yaml` file to `/app/config.yaml`
- **Logs**: Application logs are written to stdout/stderr (viewable with `docker logs`)

### Environment Variables

The container uses the following default configuration:
- **Port**: 3000
- **Config File**: `/app/config.yaml`

### Network Requirements

The container needs network access to:
- Send Wake-on-LAN packets to target devices
- Ping devices for status checking
- Perform network discovery

For network discovery and WoL functionality, you may need to run the container with:

```bash
docker run -d \
  --name wololo \
  --network host \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  ghcr.io/sharmashobhit/wololo:latest
```

## Security Considerations

- The container runs as a non-root user (`wololo`)
- Only the necessary system packages are installed
- Network capabilities are limited to what's required for the application

## Building Locally

To build the container locally:

```bash
# Build the image
docker build -t wololo:local .

# Run the local image
docker run -d \
  --name wololo-local \
  -p 3000:3000 \
  -v $(pwd)/config.yaml:/app/config.yaml:ro \
  wololo:local
```

## Troubleshooting

### Container Health Check

The container includes a health check that pings the `/hello` endpoint. Check the health status with:

```bash
docker inspect wololo --format='{{.State.Health.Status}}'
```

### Logs

View container logs:

```bash
docker logs wololo
```

### Network Issues

If Wake-on-LAN or network discovery isn't working:

1. Ensure the container has proper network access
2. Consider using `--network host` for full network capabilities
3. Check that the target devices are on the same network segment

### Permission Issues

If you encounter permission issues with the config file:

```bash
# Ensure proper permissions
chmod 644 config.yaml
```

## Automated Builds

Container images are automatically built and published when:
- A new tag matching the pattern `v*.*.*` is pushed to the repository
- The build process creates multi-architecture images (amd64, arm64)
- Images are tagged with both the full version and major/minor versions

Example tags for version `v1.2.3`:
- `ghcr.io/sharmashobhit/wololo:v1.2.3`
- `ghcr.io/sharmashobhit/wololo:1.2.3`
- `ghcr.io/sharmashobhit/wololo:1.2`
- `ghcr.io/sharmashobhit/wololo:1`