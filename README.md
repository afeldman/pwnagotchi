# Pwnagotchi - Rust Edition

WiFi handshake capture tool for Raspberry Pi, rewritten in Rust for improved performance, memory safety, and reliability.

## Overview

This is a complete Rust rewrite of the [pwnagotchi](https://pwnagotchi.org/) project. Pwnagotchi is a tool that leverages [bettercap](https://www.bettercap.org/) to capture WPA handshakes from surrounding WiFi networks.

## Features

- **Memory Safe**: Rewritten in Rust for guaranteed memory safety
- **High Performance**: Async/await with tokio for efficient I/O
- **Modular Design**: Clean separation into multiple crates
- **Plugin System**: Extensible via trait-based plugins
- **Mesh Networking**: Communicate with other pwnagotchi units
- **E-ink Display Support**: Visual feedback on e-ink screens
- **State Machine**: Mood-based behavior (bored, excited, sad, etc.)

## Architecture

### Crates

- **pwnagotchi-core**: Main agent logic, WiFi monitoring, handshake detection
- **pwnagotchi-bettercap**: HTTP/WebSocket client for bettercap API
- **pwnagotchi-automata**: State machine for agent behavior
- **pwnagotchi-mesh**: Mesh networking with cryptographic identity
- **pwnagotchi-ui**: Display rendering for e-ink screens
- **pwnagotchi-plugins**: Plugin system with async traits
- **pwnagotchi-cli**: Command-line interface

## Building

### Prerequisites

- Rust 1.75 or later
- Bettercap installed and running
- Raspberry Pi with WiFi interface in monitor mode

### Compile

```bash
cargo build --release
```

### Install

```bash
cargo install --path pwnagotchi-cli
```

## Usage

### Start the agent

```bash
pwnagotchi start
```

### With custom configuration

```bash
pwnagotchi -c config.toml start
```

### Check configuration

```bash
pwnagotchi check-config config.toml
```

### Show version

```bash
pwnagotchi version
```

## Configuration

Create a `config.toml` file:

```toml
[main]
iface = "wlan0mon"
mon_start_cmd = "iw phy phy0 interface add wlan0mon type monitor"
no_restart = false
mon_max_blind_epochs = 50

[bettercap]
hostname = "127.0.0.1"
scheme = "http"
port = 8081
username = "pwnagotchi"
password = "pwnagotchi"
handshakes = "/root/handshakes"
silence = []

[personality]
bond_encounters_factor = 20000.0
bored_num_epochs = 15
sad_num_epochs = 25
excited_num_epochs = 10
max_misses_for_recon = 5
max_inactive_scale = 10
recon_inactive_multiplier = 2.0
recon_time = 30
channels = []
ap_ttl = 120
sta_ttl = 300
min_rssi = -200
```

## Supported Hardware

- Raspberry Pi Zero W (32-bit)
- Raspberry Pi Zero 2W (64-bit)
- Raspberry Pi 3 (64-bit)
- Raspberry Pi 4 (64-bit)
- Raspberry Pi 5 (64-bit)

## Differences from Python Version

### Improvements

- **Memory Safety**: No more segfaults or memory leaks
- **Performance**: Faster event processing with async I/O
- **Type Safety**: Compile-time type checking
- **Concurrency**: Native async/await with tokio
- **Error Handling**: Proper Result types with anyhow

### Missing Features (TODO)

- [ ] Full AI integration (removed for stability in original)
- [ ] Web UI (currently CLI only)
- [ ] Complete plugin ecosystem
- [ ] GPIO display drivers (mock display only)
- [ ] Advanced mesh protocol features

## Plugin Development

Create a plugin by implementing the `Plugin` trait:

```rust
use async_trait::async_trait;
use pwnagotchi_plugins::Plugin;
use pwnagotchi_core::{AccessPoint, Station};

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "My custom plugin" }

    async fn on_handshake(
        &mut self,
        filename: &str,
        ap: &AccessPoint,
        station: &Station,
    ) {
        println!("Handshake captured: {}", filename);
    }
}
```

## Development

### Run tests

```bash
cargo test --all
```

### Run with debug logging

```bash
cargo run -- -d start
```

### Check code

```bash
cargo clippy --all
cargo fmt --all
```

## Utility Tools

### Backup & Restore

The `pwn-backup` tool provides SSH-based backup and restore functionality:

```bash
# Build the tool
cargo build --release -p pwnagotchi-tools

# Backup from device
./target/release/pwn-backup backup -n 10.0.0.2 -u pi -o backup.tgz

# Restore to device
./target/release/pwn-backup restore -n 10.0.0.2 -u pi -b backup.tgz

# Auto-find latest backup
./target/release/pwn-backup restore -n 10.0.0.2 -u pi
```

**Note**: Make sure your SSH key is added to ssh-agent for authentication.

### Connection Sharing Scripts

Helper scripts for sharing internet connection with your Pwnagotchi over USB:

#### Linux

```bash
sudo ./scripts/linux_connection_share.sh [usb_interface] [upstream_interface]
# Example: sudo ./scripts/linux_connection_share.sh enx00e04c680378 wlp2s0
```

#### macOS

```bash
sudo ./scripts/macos_connection_share.sh [upstream_interface] [usb_ip]
# Example: sudo ./scripts/macos_connection_share.sh en0 10.0.0.1
```

#### Windows

```powershell
.\scripts\win_connection_share.ps1
```

#### OpenBSD

```bash
sudo ./scripts/openbsd_connection_share.sh
```

## License

This Rust port maintains the GPL3 license of the original project.

## Credits

- Original Pwnagotchi: [@evilsocket](https://github.com/evilsocket)
- Current Python maintainer: [@jayofelony](https://github.com/jayofelony)
- Rust port: Community effort

## Contributing

Contributions welcome! Please open issues or pull requests on GitHub.

## Links

- [Original Pwnagotchi](https://pwnagotchi.org/)
- [Bettercap](https://www.bettercap.org/)
- [Hashcat](https://hashcat.net/hashcat/)
