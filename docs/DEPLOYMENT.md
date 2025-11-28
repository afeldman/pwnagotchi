# Deployment Guide

This guide covers deploying Pwnagotchi Rust on a Raspberry Pi.

## Prerequisites

### Hardware

- Raspberry Pi Zero W, Zero 2W, Pi 3, Pi 4, or Pi 5
- microSD card (8GB minimum, 16GB+ recommended)
- USB cable for power/data
- Optional: Waveshare e-ink display

### Software

- Raspberry Pi OS Lite (64-bit for Pi Zero 2W/3/4/5, 32-bit for Zero W)
- Bettercap
- Rust toolchain

## Installation

### 1. Prepare Raspberry Pi OS

```bash
# Update system
sudo apt update
sudo apt upgrade -y

# Install dependencies
sudo apt install -y git build-essential libpcap-dev libssl-dev pkg-config
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install Bettercap

```bash
# Add repository
echo "deb https://pkgs.bettercap.org/ any any" | sudo tee -a /etc/apt/sources.list
curl https://pkgs.bettercap.org/public.key | sudo apt-key add -

# Install
sudo apt update
sudo apt install -y bettercap

# Verify installation
bettercap -version
```

### 4. Clone and Build Pwnagotchi

```bash
# Clone repository
git clone https://github.com/jayofelony/pwnagotchi.git
cd pwnagotchi
git checkout rust

# Build release version
cargo build --release

# Install binary
sudo cp target/release/pwnagotchi /usr/local/bin/
```

### 5. Configuration

```bash
# Create config directory
sudo mkdir -p /etc/pwnagotchi

# Copy example config
sudo cp config.example.toml /etc/pwnagotchi/config.toml

# Edit configuration
sudo nano /etc/pwnagotchi/config.toml
```

Example configuration:

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

### 6. Setup WiFi Monitor Mode

```bash
# Create startup script
sudo tee /usr/local/bin/start-monitor.sh << 'EOF'
#!/bin/bash
iw phy phy0 interface add wlan0mon type monitor
ip link set wlan0mon up
EOF

sudo chmod +x /usr/local/bin/start-monitor.sh
```

### 7. Setup Bettercap Service

```bash
# Create bettercap config
sudo tee /etc/bettercap/caplets/pwnagotchi-manual.cap << 'EOF'
set api.rest.username pwnagotchi
set api.rest.password pwnagotchi
set api.rest.address 127.0.0.1
set api.rest.port 8081

api.rest on
wifi.recon on
EOF

# Create systemd service
sudo tee /etc/systemd/system/bettercap.service << 'EOF'
[Unit]
Description=Bettercap API Service
After=network.target

[Service]
Type=simple
ExecStartPre=/usr/local/bin/start-monitor.sh
ExecStart=/usr/bin/bettercap -iface wlan0mon -caplet pwnagotchi-manual
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable bettercap
sudo systemctl start bettercap
```

### 8. Setup Pwnagotchi Service

```bash
# Create systemd service
sudo tee /etc/systemd/system/pwnagotchi.service << 'EOF'
[Unit]
Description=Pwnagotchi Agent
After=bettercap.service
Requires=bettercap.service

[Service]
Type=simple
ExecStart=/usr/local/bin/pwnagotchi -c /etc/pwnagotchi/config.toml start
Restart=always
RestartSec=10
User=root
WorkingDirectory=/root

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable pwnagotchi
sudo systemctl start pwnagotchi
```

## Verification

### Check Services

```bash
# Check bettercap status
sudo systemctl status bettercap

# Check pwnagotchi status
sudo systemctl status pwnagotchi

# View logs
sudo journalctl -u pwnagotchi -f
sudo journalctl -u bettercap -f
```

### Test Bettercap API

```bash
curl -u pwnagotchi:pwnagotchi http://127.0.0.1:8081/api/session
```

### Check Handshakes

```bash
ls -lh /root/handshakes/
```

## USB Gadget Mode

### Enable USB Ethernet

```bash
# Edit boot config
echo "dtoverlay=dwc2" | sudo tee -a /boot/config.txt

# Edit modules
echo "dwc2" | sudo tee -a /etc/modules
echo "g_ether" | sudo tee -a /etc/modules

# Configure USB interface
sudo tee -a /etc/network/interfaces << 'EOF'
auto usb0
iface usb0 inet static
    address 10.0.0.2
    netmask 255.255.255.0
    gateway 10.0.0.1
EOF

# Reboot
sudo reboot
```

### Connect from Host

On your computer, run the connection sharing script:

```bash
# Linux
sudo ./scripts/linux_connection_share.sh enx00e04c680378 wlp2s0

# macOS
sudo ./scripts/macos_connection_share.sh en0 10.0.0.1
```

Then SSH to the Pi:

```bash
ssh pi@10.0.0.2
```

## E-ink Display (Optional)

### Waveshare 2.13" V2

```bash
# Enable SPI
sudo raspi-config
# Interface Options → SPI → Enable

# Install BCM2835 library
wget http://www.airspayce.com/mikem/bcm2835/bcm2835-1.71.tar.gz
tar zxvf bcm2835-1.71.tar.gz
cd bcm2835-1.71/
./configure
make
sudo make check
sudo make install
```

TODO: Rust GPIO driver implementation

## Backup & Restore

### Create Backup

From your computer:

```bash
# Build backup tool
cargo build --release -p pwnagotchi-tools

# Create backup
./target/release/pwn-backup backup -n 10.0.0.2 -u pi -o backup.tgz
```

### Restore Backup

```bash
./target/release/pwn-backup restore -n 10.0.0.2 -u pi -b backup.tgz
```

## Updates

### Update Pwnagotchi

```bash
cd ~/pwnagotchi
git pull
cargo build --release
sudo systemctl stop pwnagotchi
sudo cp target/release/pwnagotchi /usr/local/bin/
sudo systemctl start pwnagotchi
```

### Update Bettercap

```bash
sudo apt update
sudo apt upgrade bettercap
sudo systemctl restart bettercap
```

## Troubleshooting

### Pwnagotchi Not Starting

```bash
# Check logs
sudo journalctl -u pwnagotchi -n 50

# Test manually
sudo /usr/local/bin/pwnagotchi -d -c /etc/pwnagotchi/config.toml start
```

### Bettercap Not Responding

```bash
# Check if running
ps aux | grep bettercap

# Test API
curl -u pwnagotchi:pwnagotchi http://127.0.0.1:8081/api/session

# Restart service
sudo systemctl restart bettercap
```

### WiFi Interface Issues

```bash
# Check interface
ip link show

# Check monitor mode
iw dev

# Manually create monitor interface
sudo iw phy phy0 interface add wlan0mon type monitor
sudo ip link set wlan0mon up
```

### No Handshakes Captured

```bash
# Check WiFi is scanning
curl -u pwnagotchi:pwnagotchi http://127.0.0.1:8081/api/session/wifi

# Check for access points
sudo iw dev wlan0mon scan

# Check bettercap WiFi module
sudo bettercap -iface wlan0mon
> wifi.recon on
> wifi.show
```

## Performance Tuning

### Optimize for Battery Life

```toml
[personality]
recon_time = 60  # Longer recon intervals
ap_ttl = 300     # Longer AP TTL
sta_ttl = 600    # Longer station TTL
```

### Optimize for Capture Rate

```toml
[personality]
recon_time = 15  # Shorter recon intervals
channels = [1, 6, 11]  # Focus on main channels
```

## Security

### Change Default Credentials

```toml
[bettercap]
username = "custom_user"
password = "secure_password_here"
```

### SSH Hardening

```bash
# Disable password auth
sudo nano /etc/ssh/sshd_config
# Set: PasswordAuthentication no

# Use SSH keys only
ssh-copy-id pi@10.0.0.2

# Restart SSH
sudo systemctl restart ssh
```

## Monitoring

### Web Dashboard (Future)

TODO: Web UI implementation

### Remote Logging

```bash
# Forward logs to remote syslog
sudo nano /etc/rsyslog.conf
# Add: *.* @@remote-server:514
```

## Support

- GitHub Issues: https://github.com/jayofelony/pwnagotchi/issues
- Discord: https://discord.gg/pwnagotchi
- Wiki: https://github.com/jayofelony/pwnagotchi/wiki
