# Distribution Files

This directory contains files for packaging and distribution of cloud-netconfig.

## Structure

```
distribution/
├── systemd/
│   └── cloud-netconfigd.service    # Systemd unit file
├── config/
│   ├── cloud-network.yaml          # Default configuration
│   └── examples/                   # Example configurations
│       ├── minimal.yaml            # Minimal setup
│       ├── multi-interface.yaml    # Multi-interface setup
│       ├── azure.yaml              # Azure-specific
│       ├── aws.yaml                # AWS EC2-specific (IMDSv2)
│       ├── gcp.yaml                # GCP-specific
│       └── production.yaml         # Production-hardened
└── README.md                       # This file
```

## Files

### systemd/cloud-netconfigd.service

Systemd service unit file with:
- Type=notify for proper startup signaling
- Watchdog support (60s)
- Security hardening (capabilities, sandboxing)
- Automatic restart on failure

**Installation**:
```bash
sudo install -m 0644 systemd/cloud-netconfigd.service /lib/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable cloud-netconfigd
```

### config/cloud-network.yaml

Default configuration file with all options documented.

**Installation**:
```bash
sudo mkdir -p /etc/cloud-network
sudo install -m 644 config/cloud-network.yaml /etc/cloud-network/
```

## Example Configurations

### minimal.yaml
Absolute minimal configuration. Uses defaults for everything except basic logging.

**Use case**: Quick testing, development

### multi-interface.yaml
Full configuration for instances with multiple network interfaces.

**Use case**:
- AWS EC2 instances with ENIs
- Azure VMs with multiple NICs
- GCP instances with multiple interfaces

### azure.yaml
Azure-optimized configuration with JSON logging for Azure Monitor.

**Features**:
- JSON logging for Azure Monitor integration
- Azure IMDS API version specification
- Proper retry configuration

### aws.yaml
AWS EC2-optimized configuration with IMDSv2 support.

**Features**:
- IMDSv2 for enhanced security
- JSON logging for CloudWatch integration
- Extended retry configuration
- Optional jumbo frame MTU (9001)

### gcp.yaml
Google Cloud Platform-optimized configuration.

**Features**:
- JSON logging for Cloud Logging
- Recursive metadata fetch
- GCP-specific interface naming (ens4, ens5)
- Optional MTU override for VPC default (1460)

### production.yaml
Production-hardened configuration with conservative settings.

**Features**:
- Reduced log verbosity (warn level)
- Longer refresh intervals
- Extended timeouts and retries
- Security-first defaults
- Minimal capability set

## Usage

1. **Choose a configuration**: Pick an example that matches your environment
2. **Copy to /etc/cloud-network/**:
   ```bash
   sudo cp config/examples/azure.yaml /etc/cloud-network/cloud-network.yaml
   ```
3. **Edit as needed**: Modify interface names, intervals, etc.
4. **Restart daemon**:
   ```bash
   sudo systemctl restart cloud-netconfigd
   ```

## Package Maintainers

When packaging cloud-netconfig:

1. Install the systemd unit:
   ```
   /lib/systemd/system/cloud-netconfigd.service
   ```

2. Install default config:
   ```
   /etc/cloud-network/cloud-network.yaml
   ```

3. Optionally install examples to:
   ```
   /usr/share/doc/cloud-netconfig/examples/
   ```

4. Create the user:
   ```bash
   useradd -r -s /usr/bin/nologin -d /run/cloud-network cloud-network
   ```

5. Enable the service (optional):
   ```bash
   systemctl enable cloud-netconfigd
   ```

## License

All files: LGPL-3.0-or-later
