# rbuilder-operator

Specific implementation (based on the public rbuilder) of a block builder to be used on a TDX context.

## Installation

### Debian/Ubuntu Packages

For Debian/Ubuntu systems, you can install rbuilder-operator using the pre-built `.deb` packages:

```bash
# Download the latest release
wget https://github.com/flashbots/rbuilder-operator/releases/latest/download/rbuilder-operator-VERSION-x86_64-unknown-linux-gnu-release.deb

# Install the package
sudo dpkg -i rbuilder-operator-VERSION-x86_64-unknown-linux-gnu-release.deb

# Install dependencies if needed
sudo apt-get install -f

# Enable and start the service
sudo systemctl enable rbuilder-operator
sudo systemctl start rbuilder-operator

# Check the service status
sudo systemctl status rbuilder-operator
```

The package includes:
- The `rbuilder` binary installed to `/usr/bin/rbuilder`
- A systemd service unit for easy management
- Documentation and license files

### Binary Releases

Pre-compiled binaries are available for download from the [releases page](https://github.com/flashbots/rbuilder-operator/releases).

### Building from Source

#### Prerequisites

- Rust toolchain (latest stable)
- Protocol Buffers compiler (`protobuf-compiler`)
- CMake
- libclang development files

#### Build

```bash
# Clone the repository
git clone https://github.com/flashbots/rbuilder-operator.git
cd rbuilder-operator

# Build the project
make build

# For reproducible builds (recommended for production)
make build-reproducible
```

#### Build Debian Package

To build a Debian package locally:

```bash
# Install cargo-deb
cargo install cargo-deb

# Build the package
make build-deb-x86_64-unknown-linux-gnu VERSION=v0.1.0
```

## Configuration

Create a configuration file at `/etc/rbuilder-operator/config.toml` (when using the systemd service) or specify a custom path.

See the [example configuration](config-live-example.toml) for available options.

## Usage

### With systemd service (recommended for production)

```bash
# Start the service
sudo systemctl start rbuilder-operator

# View logs
sudo journalctl -u rbuilder-operator -f
```

### Manual execution

```bash
rbuilder --config /path/to/config.toml
```

## Development

### Testing

```bash
make test
```

### Linting

```bash
make lint
```

### Format code

```bash
make fmt
```
