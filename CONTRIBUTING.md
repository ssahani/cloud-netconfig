# Contributing to cloud-netconfig

Thank you for your interest in contributing to cloud-netconfig! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue on GitHub with:

- A clear, descriptive title
- Steps to reproduce the problem
- Expected behavior
- Actual behavior
- Your environment (OS, cloud provider, cloud-netconfig version)
- Relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are welcome! Please open an issue with:

- A clear description of the proposed feature
- Use cases and benefits
- Potential implementation approach (if you have one)

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following the coding standards below
3. **Add tests** if applicable
4. **Update documentation** if you've changed APIs or configuration
5. **Ensure tests pass**: `cargo test`
6. **Check formatting**: `cargo fmt --all -- --check`
7. **Run clippy**: `cargo clippy --all-features -- -D warnings`
8. **Commit your changes** with a clear commit message
9. **Push to your fork** and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Linux system (required for netlink operations)
- Git

### Building

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/cloud-netconfig.git
cd cloud-netconfig

# Build
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running Locally

```bash
# Build
cargo build --release

# Create test config
mkdir -p /tmp/cloud-network
cp distribution/config/cloud-network.yaml /tmp/cloud-network/

# Run (requires root for network operations)
sudo target/release/cloud-netconfigd --config /tmp/cloud-network/cloud-network.yaml
```

## Coding Standards

### Rust Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` to format code
- Address all `clippy` warnings: `cargo clippy --all-features -- -D warnings`
- Write idiomatic Rust code

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Add comments for complex logic
- Use proper error handling (avoid unwrap/expect in production code)

### Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments where helpful
- Update README.md for user-facing changes
- Update configuration documentation for new options

### Testing

- Write unit tests for new functionality
- Add integration tests for cloud provider implementations
- Aim for good test coverage
- Test error cases, not just happy paths

### Commits

- Write clear, concise commit messages
- Use present tense ("Add feature" not "Added feature")
- Reference issues and PRs when relevant
- Keep commits focused on a single change

Example commit message:
```
Add support for custom MTU configuration

- Add mtu section to config
- Implement MTU configuration in network module
- Add tests for MTU handling

Fixes #123
```

## Adding Cloud Provider Support

To add support for a new cloud provider:

1. **Create provider module**: `src/provider/newprovider.rs`
2. **Implement CloudProvider trait**:
   ```rust
   #[async_trait::async_trait]
   impl CloudProvider for NewProvider {
       async fn fetch_cloud_metadata(&mut self) -> Result<()> { ... }
       async fn configure_network_from_cloud_meta(&self, env: &mut Environment) -> Result<()> { ... }
       // ... other methods
   }
   ```
3. **Add detection logic**: Update `src/cloud/mod.rs`
4. **Add configuration**: Update `src/conf/mod.rs`
5. **Add example config**: Create `distribution/config/examples/newprovider.yaml`
6. **Update README**: Add provider to supported cloud providers table
7. **Add tests**: Create tests for metadata parsing and network configuration

## Project Structure

```
cloud-netconfig/
├── src/
│   ├── bin/
│   │   ├── cloud-netconfigd.rs  # Main daemon
│   │   └── cnctl.rs             # CLI tool
│   ├── cloud/                   # Cloud provider detection
│   ├── conf/                    # Configuration parsing
│   ├── network/                 # Network configuration (netlink)
│   ├── provider/                # Cloud provider implementations
│   │   ├── azure.rs
│   │   ├── ec2.rs
│   │   ├── gcp.rs
│   │   └── network.rs
│   ├── system/                  # System operations (users, capabilities)
│   └── web/                     # HTTP utilities
├── distribution/                # Distribution files
│   ├── config/                  # Configuration examples
│   └── systemd/                 # Systemd unit files
└── tests/                       # Integration tests
```

## License

By contributing to cloud-netconfig, you agree that your contributions will be licensed under the LGPL-3.0-or-later license.

All source files must include the SPDX license header:

```rust
// SPDX-License-Identifier: LGPL-3.0-or-later
```

## Questions?

If you have questions about contributing, feel free to:

- Open a GitHub issue with the "question" label
- Start a discussion in GitHub Discussions

Thank you for contributing to cloud-netconfig!
