use anyhow::{Context, Result};
use pwnagotchi_automata::{Automata, Mood, PersonalityConfig};
use pwnagotchi_bettercap::{BettercapClient, BettercapEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Configuration for the agent
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub main: MainConfig,
    pub bettercap: BettercapConfig,
    pub personality: PersonalityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MainConfig {
    pub iface: String,
    pub mon_start_cmd: Option<String>,
    pub no_restart: bool,
    pub mon_max_blind_epochs: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BettercapConfig {
    pub hostname: String,
    pub scheme: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub handshakes: PathBuf,
    pub silence: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            main: MainConfig {
                iface: "wlan0mon".to_string(),
                mon_start_cmd: None,
                no_restart: false,
                mon_max_blind_epochs: 50,
            },
            bettercap: BettercapConfig {
                hostname: "127.0.0.1".to_string(),
                scheme: "http".to_string(),
                port: 8081,
                username: "pwnagotchi".to_string(),
                password: "pwnagotchi".to_string(),
                handshakes: PathBuf::from("/root/handshakes"),
                silence: vec![],
            },
            personality: PersonalityConfig::default(),
        }
    }
}

/// WiFi Access Point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    pub bssid: String,
    pub essid: String,
    pub channel: u8,
    pub rssi: i32,
    pub encryption: String,
    pub clients: Vec<String>,
    pub sent: u64,
    pub received: u64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// WiFi Client Station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub mac: String,
    pub ap_bssid: String,
    pub rssi: i32,
    pub sent: u64,
    pub received: u64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Handshake capture event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub filename: String,
    pub ap_bssid: String,
    pub station_mac: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub handshake_type: String, // PMKID, full, half
}

/// Main agent implementation
pub struct Agent {
    config: AgentConfig,
    bettercap: BettercapClient,
    automata: Automata,
    access_points: HashMap<String, AccessPoint>,
    stations: HashMap<String, Station>,
    handshakes: Vec<Handshake>,
    current_channel: u8,
    total_aps: usize,
    aps_on_channel: usize,
    started_at: chrono::DateTime<chrono::Utc>,
}

impl Agent {
    /// Create new agent instance
    pub fn new(config: AgentConfig) -> Result<Self> {
        let bettercap = BettercapClient::new(
            &config.bettercap.hostname,
            &config.bettercap.scheme,
            config.bettercap.port,
            &config.bettercap.username,
            &config.bettercap.password,
        )?;

        let automata = Automata::new(config.personality.clone());

        // Create handshakes directory
        std::fs::create_dir_all(&config.bettercap.handshakes)
            .context("Failed to create handshakes directory")?;

        Ok(Self {
            config,
            bettercap,
            automata,
            access_points: HashMap::new(),
            stations: HashMap::new(),
            handshakes: Vec::new(),
            current_channel: 0,
            total_aps: 0,
            aps_on_channel: 0,
            started_at: chrono::Utc::now(),
        })
    }

    /// Wait for bettercap to be available
    async fn wait_bettercap(&self) -> Result<()> {
        loop {
            match self.bettercap.session().await {
                Ok(_) => {
                    info!("Bettercap API is available");
                    return Ok(());
                }
                Err(_) => {
                    info!("Waiting for bettercap API to be available...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Setup event filters
    async fn setup_events(&self) -> Result<()> {
        info!("Connecting to bettercap...");

        for tag in &self.config.bettercap.silence {
            let _ = self.bettercap.run(&format!("events.ignore {}", tag)).await;
        }

        Ok(())
    }

    /// Reset WiFi settings
    async fn reset_wifi_settings(&self) -> Result<()> {
        let iface = &self.config.main.iface;
        let cfg = &self.config.personality;

        self.bettercap.run(&format!("set wifi.interface {}", iface)).await?;
        self.bettercap.run(&format!("set wifi.ap.ttl {}", cfg.ap_ttl)).await?;
        self.bettercap.run(&format!("set wifi.sta.ttl {}", cfg.sta_ttl)).await?;
        self.bettercap.run(&format!("set wifi.rssi.min {}", cfg.min_rssi)).await?;
        self.bettercap.run(&format!("set wifi.handshakes.file {}", 
            self.config.bettercap.handshakes.display())).await?;
        self.bettercap.run("set wifi.handshakes.aggregate false").await?;

        Ok(())
    }

    /// Start monitor mode
    async fn start_monitor_mode(&self) -> Result<()> {
        let mon_iface = &self.config.main.iface;
        let restart = !self.config.main.no_restart;

        // Wait for monitor interface
        loop {
            let session = self.bettercap.session().await?;
            let has_mon = session.interfaces.iter().any(|i| i.name == *mon_iface);

            if has_mon {
                info!("Found monitor interface: {}", mon_iface);
                break;
            }

            if let Some(cmd) = &self.config.main.mon_start_cmd {
                info!("Starting monitor interface...");
                self.bettercap.run(&format!("!{}", cmd)).await?;
            } else {
                info!("Waiting for monitor interface {}...", mon_iface);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }

        info!("Handshakes will be collected inside {:?}", self.config.bettercap.handshakes);

        self.reset_wifi_settings().await?;

        // Start or restart wifi module
        if self.bettercap.is_module_running("wifi").await? && restart {
            debug!("Restarting wifi module...");
            self.bettercap.restart_module("wifi.recon").await?;
            self.bettercap.run("wifi.clear").await?;
        } else if !self.bettercap.is_module_running("wifi").await? {
            debug!("Starting wifi module...");
            self.bettercap.start_module("wifi.recon").await?;
        }

        Ok(())
    }

    /// Start reconnaissance
    async fn recon(&mut self) -> Result<()> {
        let recon_time = self.config.personality.recon_time;
        let channels = &self.config.personality.channels;

        self.current_channel = 0;

        if channels.is_empty() {
            debug!("RECON {}s", recon_time);
            self.bettercap.run("wifi.recon.channel clear").await?;
        } else {
            let channel_str = channels.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            debug!("RECON {}s ON CHANNELS {}", recon_time, channel_str);
            self.bettercap.run(&format!("wifi.recon.channel {}", channel_str)).await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(recon_time as u64)).await;

        Ok(())
    }

    /// Handle bettercap event
    async fn handle_event(&mut self, event: BettercapEvent) {
        debug!("Event: {} - {:?}", event.tag, event.data);

        match event.tag.as_str() {
            "wifi.ap.new" | "wifi.ap.lost" => {
                // Handle AP events
                self.update_access_points(&event);
            }
            "wifi.client.new" | "wifi.client.lost" | "wifi.client.probe" => {
                // Handle client events
                self.update_stations(&event);
            }
            "wifi.handshake" => {
                // Handle handshake capture
                self.on_handshake(&event);
            }
            _ => {}
        }
    }

    fn update_access_points(&mut self, event: &BettercapEvent) {
        // Parse and update AP list
        // Implementation depends on bettercap event structure
        self.total_aps = self.access_points.len();
    }

    fn update_stations(&mut self, event: &BettercapEvent) {
        // Parse and update station list
    }

    fn on_handshake(&mut self, event: &BettercapEvent) {
        info!("Handshake captured!");
        self.automata.epoch().track_handshake();
        
        // Parse handshake event and save to list
        // Trigger plugin hooks
    }

    /// Start the agent
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting pwnagotchi agent");

        self.wait_bettercap().await?;
        self.setup_events().await?;
        
        self.automata.set_starting();
        
        self.start_monitor_mode().await?;

        // Start event processing
        let mut events = self.bettercap.start_websocket().await?;

        self.automata.set_ready();

        // Main event loop
        tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                // Process event
                debug!("Received event: {:?}", event);
            }
        });

        // Main agent loop
        loop {
            // Recon cycle
            self.recon().await?;

            // Process epoch
            self.automata.next_epoch();

            // Update based on mood
            match self.automata.mood() {
                Mood::Bored => {
                    info!("Agent is bored, trying different channels...");
                }
                Mood::Sad => {
                    warn!("Agent is sad, no activity for a while");
                }
                Mood::Excited => {
                    info!("Agent is excited! Capturing handshakes!");
                }
                _ => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    pub fn mood(&self) -> Mood {
        self.automata.mood()
    }

    pub fn access_points(&self) -> &HashMap<String, AccessPoint> {
        &self.access_points
    }

    pub fn handshakes(&self) -> &[Handshake] {
        &self.handshakes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let config = AgentConfig::default();
        let agent = Agent::new(config);
        assert!(agent.is_ok());
    }
}
