# Plugin Development Guide

This guide explains how to create plugins for Pwnagotchi Rust.

## Plugin System Overview

Pwnagotchi uses an async trait-based plugin system that allows you to extend the agent's functionality through hooks into various events.

## Creating a Plugin

### Basic Plugin Structure

```rust
use async_trait::async_trait;
use pwnagotchi_plugins::Plugin;
use pwnagotchi_core::{AccessPoint, Station};

pub struct MyPlugin {
    // Plugin state
    config: MyPluginConfig,
    counter: u32,
}

impl MyPlugin {
    pub fn new(config: MyPluginConfig) -> Self {
        Self {
            config,
            counter: 0,
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "A sample plugin that demonstrates the plugin system"
    }

    async fn on_loaded(&mut self) {
        println!("{} v{} loaded", self.name(), self.version());
    }

    async fn on_ready(&mut self) {
        println!("{} is ready!", self.name());
    }
}
```

### Available Hooks

The `Plugin` trait provides many hooks for different events:

#### Lifecycle Hooks

```rust
async fn on_loaded(&mut self) {
    // Called when plugin is loaded
}

async fn on_unload(&mut self) {
    // Called before plugin is unloaded
}

async fn on_ready(&mut self) {
    // Called when agent is ready and starting
}

async fn on_starting(&mut self) {
    // Called when agent is starting up
}

async fn on_rebooting(&mut self) {
    // Called when agent is rebooting
}
```

#### WiFi Event Hooks

```rust
async fn on_wifi_update(&mut self, access_points: &[AccessPoint]) {
    // Called when WiFi AP list is updated
    println!("Found {} access points", access_points.len());
}

async fn on_association(&mut self, access_point: &AccessPoint) {
    // Called when agent sends association frame
    println!("Associating with {}", access_point.essid);
}

async fn on_deauthentication(
    &mut self,
    access_point: &AccessPoint,
    station: &Station
) {
    // Called when agent deauths a client
    println!("Deauthing {} from {}",
        station.mac, access_point.essid);
}

async fn on_channel_hop(&mut self, channel: u8) {
    // Called when agent changes channel
}
```

#### Handshake Hooks

```rust
async fn on_handshake(
    &mut self,
    filename: &str,
    access_point: &AccessPoint,
    station: &Station,
) {
    // Called when a handshake is captured
    println!("Handshake captured!");
    println!("  AP: {} ({})", access_point.essid, access_point.bssid);
    println!("  Station: {}", station.mac);
    println!("  File: {}", filename);

    // You could upload to a server, send notification, etc.
}
```

#### Epoch Hooks

```rust
async fn on_epoch(&mut self, epoch: u64, epoch_data: &Epoch) {
    // Called at the end of each epoch (main loop iteration)
    println!("Epoch {}: {} handshakes, {} associations",
        epoch,
        epoch_data.num_handshakes,
        epoch_data.num_associations);
}
```

#### Mood Hooks

```rust
async fn on_mood_change(&mut self, old_mood: Mood, new_mood: Mood) {
    // Called when agent mood changes
    println!("Mood changed: {:?} -> {:?}", old_mood, new_mood);
}
```

#### Mesh Hooks

```rust
async fn on_peer_detected(&mut self, peer_fingerprint: &str) {
    // Called when another pwnagotchi is detected
    println!("Peer detected: {}", peer_fingerprint);
}

async fn on_peer_lost(&mut self, peer_fingerprint: &str) {
    // Called when peer is no longer visible
    println!("Peer lost: {}", peer_fingerprint);
}
```

#### Timing Hooks

```rust
async fn on_wait(&mut self, seconds: u32) {
    // Called when agent is waiting
}

async fn on_sleep(&mut self, seconds: u32) {
    // Called when agent is sleeping
}
```

#### Network Hooks

```rust
async fn on_internet_available(&mut self) {
    // Called when internet connectivity is detected
    // Good for uploading data, checking for updates, etc.
}
```

#### Bettercap Hooks

```rust
async fn on_bettercap_event(&mut self, event_tag: &str, event_data: &Value) {
    // Called for all bettercap events
    // Useful for handling custom events
}
```

## Complete Plugin Example

### Handshake Counter Plugin

```rust
use async_trait::async_trait;
use pwnagotchi_plugins::Plugin;
use pwnagotchi_core::{AccessPoint, Station};
use pwnagotchi_automata::Epoch;
use std::collections::HashMap;

pub struct HandshakeCounter {
    total_handshakes: u32,
    handshakes_per_ap: HashMap<String, u32>,
}

impl HandshakeCounter {
    pub fn new() -> Self {
        Self {
            total_handshakes: 0,
            handshakes_per_ap: HashMap::new(),
        }
    }
}

#[async_trait]
impl Plugin for HandshakeCounter {
    fn name(&self) -> &str {
        "handshake-counter"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Counts handshakes per access point"
    }

    async fn on_ready(&mut self) {
        println!("Handshake Counter started");
    }

    async fn on_handshake(
        &mut self,
        _filename: &str,
        access_point: &AccessPoint,
        _station: &Station,
    ) {
        self.total_handshakes += 1;

        *self.handshakes_per_ap
            .entry(access_point.bssid.clone())
            .or_insert(0) += 1;

        let ap_count = self.handshakes_per_ap
            .get(&access_point.bssid)
            .unwrap_or(&0);

        println!("Handshake #{} (AP: {}, Total for this AP: {})",
            self.total_handshakes,
            access_point.essid,
            ap_count);
    }

    async fn on_epoch(&mut self, epoch: u64, _epoch_data: &Epoch) {
        if epoch % 10 == 0 {
            println!("--- Handshake Statistics (Epoch {}) ---", epoch);
            println!("Total handshakes: {}", self.total_handshakes);
            println!("Unique APs: {}", self.handshakes_per_ap.len());

            let mut sorted: Vec<_> = self.handshakes_per_ap.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));

            println!("Top 5 APs:");
            for (bssid, count) in sorted.iter().take(5) {
                println!("  {}: {} handshakes", bssid, count);
            }
        }
    }
}
```

### Web Notification Plugin

```rust
use async_trait::async_trait;
use pwnagotchi_plugins::Plugin;
use pwnagotchi_core::{AccessPoint, Station};
use reqwest::Client;
use serde_json::json;

pub struct WebhookNotifier {
    webhook_url: String,
    client: Client,
}

impl WebhookNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Plugin for WebhookNotifier {
    fn name(&self) -> &str {
        "webhook-notifier"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Sends webhook notifications on handshake capture"
    }

    async fn on_handshake(
        &mut self,
        filename: &str,
        access_point: &AccessPoint,
        station: &Station,
    ) {
        let payload = json!({
            "event": "handshake",
            "ap": {
                "essid": access_point.essid,
                "bssid": access_point.bssid,
                "channel": access_point.channel,
            },
            "station": station.mac,
            "filename": filename,
        });

        if let Err(e) = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
        {
            eprintln!("Failed to send webhook: {}", e);
        }
    }
}
```

## Registering Plugins

To use your plugins, register them with the plugin manager:

```rust
use pwnagotchi_plugins::PluginManager;

let mut plugin_manager = PluginManager::new();

// Register plugins
plugin_manager.register(Box::new(HandshakeCounter::new()));
plugin_manager.register(Box::new(
    WebhookNotifier::new("https://example.com/webhook".to_string())
));

// Trigger hooks
plugin_manager.trigger_ready().await;
plugin_manager.trigger_handshake(filename, &ap, &station).await;
```

## Plugin Configuration

You can add configuration to your plugins:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MyPluginConfig {
    pub enabled: bool,
    pub api_key: String,
    pub update_interval: u32,
}

pub struct MyPlugin {
    config: MyPluginConfig,
}

impl MyPlugin {
    pub fn from_config(config: MyPluginConfig) -> Self {
        Self { config }
    }
}
```

Then load from TOML:

```toml
# config.toml
[plugins.my-plugin]
enabled = true
api_key = "your-api-key"
update_interval = 60
```

## Best Practices

### 1. Keep Hooks Fast

Hooks are called synchronously in the event loop. Avoid blocking operations:

```rust
// Bad - blocks the event loop
async fn on_handshake(&mut self, ...) {
    std::thread::sleep(Duration::from_secs(5)); // DON'T DO THIS
}

// Good - spawn a task for long operations
async fn on_handshake(&mut self, ...) {
    let data = prepare_data();
    tokio::spawn(async move {
        upload_data(data).await;
    });
}
```

### 2. Handle Errors Gracefully

```rust
async fn on_handshake(&mut self, ...) {
    if let Err(e) = self.send_notification().await {
        eprintln!("Notification failed: {}", e);
        // Continue operation, don't panic
    }
}
```

### 3. Use Logging

```rust
use tracing::{info, warn, error};

async fn on_ready(&mut self) {
    info!("{} v{} initialized", self.name(), self.version());
}

async fn on_handshake(&mut self, ...) {
    info!("Handshake captured from {}", ap.essid);
}
```

### 4. Test Your Plugin

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin() {
        let mut plugin = MyPlugin::new();
        plugin.on_ready().await;
        // Test plugin behavior
    }
}
```

## Plugin Ideas

Here are some ideas for useful plugins:

- **GPS Logger**: Log GPS coordinates with handshakes
- **Discord Notifier**: Send Discord messages on events
- **Statistics**: Track and report detailed statistics
- **Auto-Upload**: Upload handshakes to WPA-sec or similar
- **LED Controller**: Control LED patterns based on mood
- **Sound Effects**: Play sounds for different events
- **Screen Saver**: Custom display modes
- **Auto-Update**: Check and install updates
- **Backup**: Periodic backup to cloud storage

## Example Plugins

See the `pwnagotchi-plugins/src/` directory for the built-in example plugin.

## Resources

- [Plugin Trait Documentation](../pwnagotchi-plugins/src/lib.rs)
- [Core Types](../pwnagotchi-core/src/lib.rs)
- [Async Trait Documentation](https://docs.rs/async-trait/)
