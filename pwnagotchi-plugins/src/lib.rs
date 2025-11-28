use async_trait::async_trait;
use pwnagotchi_automata::{Epoch, Mood};
use pwnagotchi_core::{AccessPoint, Handshake, Station};
use serde_json::Value;

/// Plugin trait for extending agent functionality
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str;

    /// Called when the plugin is loaded
    async fn on_loaded(&mut self) {}

    /// Called when the plugin is unloaded
    async fn on_unload(&mut self) {}

    /// Called when the agent is ready
    async fn on_ready(&mut self) {}

    /// Called when the agent starts
    async fn on_starting(&mut self) {}

    /// Called when the agent is rebooting
    async fn on_rebooting(&mut self) {}

    /// Called when a handshake is captured
    async fn on_handshake(
        &mut self,
        _filename: &str,
        _access_point: &AccessPoint,
        _station: &Station,
    ) {
    }

    /// Called on each epoch
    async fn on_epoch(&mut self, _epoch: u64, _epoch_data: &Epoch) {}

    /// Called when the agent mood changes
    async fn on_mood_change(&mut self, _old_mood: Mood, _new_mood: Mood) {}

    /// Called when WiFi list is updated
    async fn on_wifi_update(&mut self, _access_points: &[AccessPoint]) {}

    /// Called when the agent is waiting
    async fn on_wait(&mut self, _seconds: u32) {}

    /// Called when the agent is sleeping
    async fn on_sleep(&mut self, _seconds: u32) {}

    /// Called when the agent is sending an association frame
    async fn on_association(&mut self, _access_point: &AccessPoint) {}

    /// Called when the agent is deauthenticating a client
    async fn on_deauthentication(&mut self, _access_point: &AccessPoint, _station: &Station) {}

    /// Called when the agent hops to a new channel
    async fn on_channel_hop(&mut self, _channel: u8) {}

    /// Called when a peer is detected
    async fn on_peer_detected(&mut self, _peer_fingerprint: &str) {}

    /// Called when a peer is lost
    async fn on_peer_lost(&mut self, _peer_fingerprint: &str) {}

    /// Called when internet connectivity is available
    async fn on_internet_available(&mut self) {}

    /// Called on bettercap events
    async fn on_bettercap_event(&mut self, _event_tag: &str, _event_data: &Value) {}
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// Call on_ready for all plugins
    pub async fn trigger_ready(&mut self) {
        for plugin in &mut self.plugins {
            plugin.on_ready().await;
        }
    }

    /// Call on_handshake for all plugins
    pub async fn trigger_handshake(
        &mut self,
        filename: &str,
        access_point: &AccessPoint,
        station: &Station,
    ) {
        for plugin in &mut self.plugins {
            plugin.on_handshake(filename, access_point, station).await;
        }
    }

    /// Call on_epoch for all plugins
    pub async fn trigger_epoch(&mut self, epoch: u64, epoch_data: &Epoch) {
        for plugin in &mut self.plugins {
            plugin.on_epoch(epoch, epoch_data).await;
        }
    }

    /// Call on_wifi_update for all plugins
    pub async fn trigger_wifi_update(&mut self, access_points: &[AccessPoint]) {
        for plugin in &mut self.plugins {
            plugin.on_wifi_update(access_points).await;
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example plugin implementation
pub struct ExamplePlugin;

#[async_trait]
impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Example plugin demonstrating the plugin system"
    }

    async fn on_ready(&mut self) {
        println!("Example plugin is ready!");
    }

    async fn on_handshake(
        &mut self,
        filename: &str,
        access_point: &AccessPoint,
        _station: &Station,
    ) {
        println!(
            "Handshake captured! File: {}, AP: {} ({})",
            filename, access_point.essid, access_point.bssid
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(ExamplePlugin));
        
        assert_eq!(manager.plugins().len(), 1);
        
        manager.trigger_ready().await;
    }
}
