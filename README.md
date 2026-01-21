# cloud-netconfig

[![CI](https://github.com/ssahani/cloud-netconfig/workflows/CI/badge.svg)](https://github.com/ssahani/cloud-netconfig/actions/workflows/ci.yml)
[![Security Audit](https://github.com/ssahani/cloud-netconfig/workflows/Security%20Audit/badge.svg)](https://github.com/ssahani/cloud-netconfig/actions/workflows/security-audit.yml)
[![License: LGPL-3.0-or-later](https://img.shields.io/badge/License-LGPL%203.0--or--later-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Release](https://img.shields.io/github/v/release/ssahani/cloud-netconfig)](https://github.com/ssahani/cloud-netconfig/releases)

**Automatic network configuration for cloud instances**

`cloud-netconfig` automatically configures network interfaces in cloud environments by fetching metadata from cloud provider APIs (Azure IMDS, AWS EC2 IMDS, GCP Metadata Service). It handles secondary IP addresses, routing tables, and policy-based routing for multi-interface cloud instances.

## Features

- 🌩️ **Multi-Cloud Support**: Azure, AWS (EC2), Google Cloud Platform, Alibaba Cloud, Oracle Cloud, DigitalOcean
- 🔄 **Automatic Configuration**: Fetches and applies network configuration from cloud metadata
- 🛣️ **Advanced Routing**: Policy-based routing for multi-interface instances
- 📡 **Event-Driven**: Monitors netlink events for dynamic reconfiguration
- 🔒 **Secure**: Runs as unprivileged user with minimal capabilities (CAP_NET_ADMIN)
- 🚀 **High Performance**: Written in Rust with async/await
- 📊 **HTTP API**: Local REST API for querying instance metadata
- ⚙️ **Systemd Integration**: Watchdog support, socket activation ready

## Installation

### From Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/ssahani/cloud-netconfig.git
cd cloud-netconfig

# Build
make build

# Install (requires root)
sudo make install
```

### Create User

```bash
sudo useradd -M -s /usr/bin/nologin cloud-network
```

### Enable Service

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now cloud-netconfigd
```

## Configuration

Configuration file: `/etc/cloud-network/cloud-network.yaml`

### Basic Example

```yaml
# Logging
logging:
  level: info
  format: text

# Server
server:
  listen:
    address: 127.0.0.1
    port: 5209

# Metadata refresh
metadata:
  refresh_interval: 300s
  request_timeout: 10s

# Network configuration
network:
  interfaces:
    enabled:
      - eth1
      - eth2

  routing:
    table_base: 9999
    policy_routing: true
```

### Full Configuration Options

See [distribution/etc/cloud-network/config.yaml](distribution/etc/cloud-network/config.yaml) for a complete annotated example with all available options.

#### Logging Section

```yaml
logging:
  level: info          # trace, debug, info, warn, error
  format: text         # text or json
  file: /var/log/...   # optional log file path
  timestamps: false    # include timestamps
```

#### Server Section

```yaml
server:
  listen:
    address: 127.0.0.1
    port: 5209
  # Optional TLS configuration
  tls:
    enabled: false
    cert_file: /path/to/cert.pem
    key_file: /path/to/key.pem
```

#### Metadata Section

```yaml
metadata:
  refresh_interval: 300s  # how often to refresh from cloud API
  request_timeout: 10s     # timeout for metadata requests
  retry:
    enabled: true
    max_attempts: 3
    backoff: 5s
```

#### Network Section

```yaml
network:
  # Supplementary interfaces to configure
  interfaces:
    enabled:
      - eth1
      - eth2
    # Or use patterns
    patterns:
      - eth*
      - ens*

  # Primary interface configuration
  primary:
    enabled: true
    interface: eth0  # optional: force specific primary

  # Routing configuration
  routing:
    table_base: 9999              # base routing table number
    policy_routing: true          # enable policy-based routing
    manage_default_routes: true   # manage default routes

  # MTU configuration
  mtu:
    auto_configure: true  # auto-configure from metadata
    override: 1500       # optional: override MTU
```

#### Cloud Provider Section

```yaml
cloud:
  auto_detect: true      # auto-detect cloud provider
  # provider: azure      # or force specific provider

  # Provider-specific settings
  azure:
    api_version: "2021-02-01"

  aws:
    imds_version: 1     # 1 or 2 (IMDSv2)
    token_ttl: 21600    # token TTL for IMDSv2 (seconds)

  gcp:
    recursive: true      # use recursive metadata fetch
```

#### Security Section

```yaml
security:
  user: cloud-network     # user to run as (drops from root)
  capabilities:
    - CAP_NET_ADMIN       # Linux capabilities to retain

  watchdog:
    enabled: true
    interval: 30s
```

#### State Management

```yaml
state:
  directory: /run/cloud-network  # runtime state directory
  persist_metadata: true          # save metadata to disk
  per_interface_files: true       # per-interface metadata files
```

#### Features

```yaml
features:
  network_events: true    # monitor netlink events
  cleanup_stale: true     # automatically remove stale config
  ipv6: false             # IPv6 support (future)
  health_check: true      # enable health check endpoint
```

## HTTP API

The daemon exposes a local HTTP API for querying metadata:

```bash
# Health check
curl http://127.0.0.1:5209/health

# Status
curl http://127.0.0.1:5209/api/status

# Cloud metadata
curl http://127.0.0.1:5209/api/cloud/status
```

## Command Line Tool

`cnctl` provides a CLI interface for managing and viewing cloud network configuration:

```bash
# Show all status (daemon, system, network)
cnctl status

# Show specific status
cnctl status system
cnctl status network

# Validate configuration file
cnctl apply --config /path/to/config.yaml --dry-run

# Reload daemon configuration
cnctl reload

# Show version
cnctl version
```

### Example Output

```bash
$ cnctl status
Daemon Status: running
     Provider: azure
      Version: 0.3.0

Cloud Provider: azure

          Name: my-vm-instance
      Location: westus
         VM Id: ca066e51-f9c1-45c8-aed2-2a1664450373
       VM Size: Standard_D2s_v3

Network Interfaces:

       Name: eth0
MAC Address: 00:22:48:04:fe:00
      State: Up
        MTU: 1500
 Private IP: 10.4.0.4/24

       Name: eth1
MAC Address: 00:0d:3a:5d:2d:66
      State: Up
        MTU: 1500
 Private IP: 10.4.0.5/24
```

## How It Works

### Multi-Interface Networking

In cloud environments, instances can have multiple network interfaces with secondary IP addresses. The challenge is that only the primary IP (from DHCP) is automatically configured - secondary IPs require manual configuration.

`cloud-netconfig` solves this by:

1. **Detecting** the cloud environment (Azure, AWS, GCP, etc.)
2. **Fetching** instance metadata from the cloud provider's metadata service
3. **Configuring** all IP addresses on all network interfaces
4. **Creating** policy-based routing rules for each IP address
5. **Setting up** routing tables for proper traffic flow
6. **Monitoring** network changes and reconfiguring automatically

### Policy-Based Routing

For each secondary IP address, `cloud-netconfig` creates:

- A custom routing table (base 9999 + interface index)
- Routing policy rules:
  - Traffic **from** the IP uses the custom table
  - Traffic **to** the IP uses the custom table
- Default route via the interface's gateway

This ensures responses go back through the correct interface.

### Example Routing Configuration

For interface `eth1` with IP `10.4.0.5/24` and gateway `10.4.0.1`:

```bash
# Routing table 10002
ip route add default via 10.4.0.1 dev eth1 table 10002

# Policy rules
ip rule add from 10.4.0.5 lookup 10002
ip rule add to 10.4.0.5 lookup 10002
```

## Cloud Provider Support

| Provider | Status | Metadata Service |
|----------|--------|------------------|
| **Azure** | ✅ Full | Azure Instance Metadata Service (IMDS) |
| **AWS EC2** | ✅ Full | EC2 Instance Metadata Service (IMDSv1/v2) |
| **GCP** | ✅ Full | GCP Metadata Server |
| **Alibaba Cloud** | ⚠️ Detection only | Not fully implemented |
| **Oracle Cloud** | ⚠️ Detection only | Not fully implemented |
| **DigitalOcean** | ⚠️ Detection only | Not fully implemented |

## Architecture

```
┌─────────────────────────────────────┐
│   cloud-netconfig daemon            │
│                                     │
│  ┌──────────────────────────────┐  │
│  │  Cloud Provider Detection    │  │
│  └──────────────────────────────┘  │
│             │                       │
│             ▼                       │
│  ┌──────────────────────────────┐  │
│  │  Metadata Fetcher            │  │
│  │  (Azure/AWS/GCP)             │  │
│  └──────────────────────────────┘  │
│             │                       │
│             ▼                       │
│  ┌──────────────────────────────┐  │
│  │  Network Configurator        │  │
│  │  • Addresses                 │  │
│  │  • Routes                    │  │
│  │  • Policy Rules              │  │
│  └──────────────────────────────┘  │
│             │                       │
│             ▼                       │
│  ┌──────────────────────────────┐  │
│  │  Netlink Interface           │  │
│  │  (rtnetlink)                 │  │
│  └──────────────────────────────┘  │
│                                     │
│  ┌──────────────────────────────┐  │
│  │  HTTP API Server (Axum)      │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
```

## Development

### Build

```bash
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Code Structure

```
src/
├── bin/
│   ├── cloud-network.rs    # Main daemon
│   └── cnctl.rs            # CLI tool
├── cloud/                  # Cloud detection
├── conf/                   # Configuration
├── network/                # Network management
├── parser/                 # Parsing utilities
├── provider/               # Cloud providers
│   ├── azure.rs
│   ├── ec2.rs
│   ├── gcp.rs
│   └── network.rs
├── system/                 # System operations
└── web/                    # HTTP utilities
```

## License

SPDX-License-Identifier: LGPL-3.0-or-later

This project is licensed under the GNU Lesser General Public License v3.0 or later.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## Troubleshooting

### Service not starting

Check logs:
```bash
sudo journalctl -u cloud-netconfigd -f
```

### Network not configured

1. Verify cloud detection:
```bash
cnctl status system
```

2. Check permissions:
```bash
sudo -u cloud-network capsh --print
```

3. Enable debug logging in `/etc/cloud-network/cloud-network.yaml`:
```yaml
logging:
  level: debug
```

### Secondary interfaces not working

Ensure interfaces are enabled in configuration:
```yaml
network:
  interfaces:
    enabled:
      - eth1
      - eth2
```

## Performance

- **Memory**: ~10-15 MB RSS
- **CPU**: <1% during normal operation
- **Startup**: <100ms from service start to ready
- **Metadata fetch**: ~50-200ms depending on cloud provider

## Security

- Runs as unprivileged user (`cloud-network`)
- Only retains `CAP_NET_ADMIN` capability
- No network access except to metadata endpoints
- State files stored in `/run` (tmpfs)
- No sensitive data logged

## Roadmap

- [ ] IPv6 support
- [ ] Enhanced retry logic with exponential backoff
- [ ] Prometheus metrics export
- [ ] Support for more cloud providers (Alibaba, Oracle, DigitalOcean)
- [ ] Integration tests with cloud provider mocks
- [ ] Hot reload of configuration
