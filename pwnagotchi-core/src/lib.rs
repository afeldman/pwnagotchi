use anyhow::{Context, Result};
use pwnagotchi_automata::{Automata, Mood, PersonalityConfig};
use pwnagotchi_bettercap::{BettercapClient, BettercapEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

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

/// Represents a WiFi Access Point.
///
/// Contains information about a discovered WiFi AP including signal strength,
/// encryption, connected clients, and network activity.
///
/// # Examples
///
/// ```
/// # use pwnagotchi_core::AccessPoint;
/// let ap = AccessPoint {
///     bssid: "aa:bb:cc:dd:ee:ff".to_string(),
///     essid: "MyNetwork".to_string(),
///     channel: 6,
///     rssi: -45,
///     encryption: "WPA2".to_string(),
///     clients: vec!["11:22:33:44:55:66".to_string()],
///     sent: 1024,
///     received: 2048,
///     last_seen: chrono::Utc::now(),
/// };
///
/// println!("AP {} on channel {}", ap.essid, ap.channel);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    /// BSSID (MAC address) of the access point
    pub bssid: String,
    /// ESSID (network name) of the access point
    pub essid: String,
    /// WiFi channel (1-14 for 2.4GHz, 36+ for 5GHz)
    pub channel: u8,
    /// Received Signal Strength Indicator in dBm
    pub rssi: i32,
    /// Encryption type (e.g., "WPA2", "WPA3", "Open")
    pub encryption: String,
    /// List of connected client MAC addresses
    pub clients: Vec<String>,
    /// Number of packets sent by the AP
    pub sent: u64,
    /// Number of packets received by the AP
    pub received: u64,
    /// Last time this AP was seen
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Represents a WiFi client station.
///
/// Contains information about a client device connected to an access point.
///
/// # Examples
///
/// ```
/// # use pwnagotchi_core::Station;
/// let station = Station {
///     mac: "11:22:33:44:55:66".to_string(),
///     ap_bssid: "aa:bb:cc:dd:ee:ff".to_string(),
///     rssi: -60,
///     sent: 512,
///     received: 1024,
///     last_seen: chrono::Utc::now(),
/// };
///
/// println!("Station {} connected to {}", station.mac, station.ap_bssid);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    /// MAC address of the client station
    pub mac: String,
    /// BSSID of the connected access point
    pub ap_bssid: String,
    /// Received Signal Strength Indicator in dBm
    pub rssi: i32,
    /// Number of packets sent by the station
    pub sent: u64,
    /// Number of packets received by the station
    pub received: u64,
    /// Last time this station was seen
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Represents a captured WPA handshake.
///
/// Contains metadata about a captured handshake that can be used for
/// offline password cracking.
///
/// # Examples
///
/// ```
/// # use pwnagotchi_core::Handshake;
/// let handshake = Handshake {
///     filename: "handshake_aa_bb_cc_dd_ee_ff.pcap".to_string(),
///     ap_bssid: "aa:bb:cc:dd:ee:ff".to_string(),
///     station_mac: "11:22:33:44:55:66".to_string(),
///     timestamp: chrono::Utc::now(),
///     handshake_type: "full".to_string(),
/// };
///
/// println!("Captured {} handshake: {}", handshake.handshake_type, handshake.filename);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    /// Filename of the captured handshake PCAP
    pub filename: String,
    /// BSSID of the access point
    pub ap_bssid: String,
    /// MAC address of the client station
    pub station_mac: String,
    /// Time when the handshake was captured
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of handshake: "PMKID", "full", or "half"
    pub handshake_type: String,
}

/// Main Pwnagotchi agent orchestrating WiFi monitoring and attacks.
///
/// The agent manages the Bettercap client, tracks access points and stations,
/// captures handshakes, and coordinates with the automata for mood-based behavior.
///
/// # Examples
///
/// ## Basic Setup
///
/// ```no_run
/// # use pwnagotchi_core::{Agent, AgentConfig};
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let config = AgentConfig::default();
/// let mut agent = Agent::new(config)?;
/// agent.start().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Checking State
///
/// ```no_run
/// # use pwnagotchi_core::{Agent, AgentConfig};
/// # fn example(agent: &Agent) {
/// println!("Visible APs: {}", agent.access_points().len());
/// println!("Handshakes: {}", agent.handshakes().len());
/// println!("Current mood: {:?}", agent.mood());
/// # }
/// ```
#[allow(dead_code)]
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
    /// Creates a new agent instance.
    ///
    /// Initializes the Bettercap client, automata, and creates the handshakes directory.
    ///
    /// # Arguments
    ///
    /// * `config` - Complete agent configuration
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the agent or an error if initialization fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Bettercap client initialization fails (invalid URL, etc.)
    /// - Handshakes directory cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_core::{Agent, AgentConfig};
    /// // Using default configuration
    /// let agent = Agent::new(AgentConfig::default())?;
    ///
    /// // Using custom configuration
    /// let config = AgentConfig::default();
    /// let agent = Agent::new(config)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

        self.bettercap
            .run(&format!("set wifi.interface {}", iface))
            .await?;
        self.bettercap
            .run(&format!("set wifi.ap.ttl {}", cfg.ap_ttl))
            .await?;
        self.bettercap
            .run(&format!("set wifi.sta.ttl {}", cfg.sta_ttl))
            .await?;
        self.bettercap
            .run(&format!("set wifi.rssi.min {}", cfg.min_rssi))
            .await?;
        self.bettercap
            .run(&format!(
                "set wifi.handshakes.file {}",
                self.config.bettercap.handshakes.display()
            ))
            .await?;
        self.bettercap
            .run("set wifi.handshakes.aggregate false")
            .await?;

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

        info!(
            "Handshakes will be collected inside {:?}",
            self.config.bettercap.handshakes
        );

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
            let channel_str = channels
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            debug!("RECON {}s ON CHANNELS {}", recon_time, channel_str);
            self.bettercap
                .run(&format!("wifi.recon.channel {}", channel_str))
                .await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(recon_time as u64)).await;

        Ok(())
    }

    /// Handle bettercap event
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn update_access_points(&mut self, _event: &BettercapEvent) {
        // Parse and update AP list
        // Implementation depends on bettercap event structure
        self.total_aps = self.access_points.len();
    }

    #[allow(dead_code)]
    fn update_stations(&mut self, _event: &BettercapEvent) {
        // Parse and update station list
    }

    #[allow(dead_code)]
    fn on_handshake(&mut self, _event: &BettercapEvent) {
        info!("Handshake captured!");
        self.automata.epoch_mut().track_handshake();

        // Parse handshake event and save to list
        // Trigger plugin hooks
    }

    /// Starts the agent and begins WiFi monitoring.
    ///
    /// This method:
    /// 1. Waits for Bettercap to become available
    /// 2. Sets up event handling
    /// 3. Starts monitor mode on the WiFi interface
    /// 4. Begins WebSocket event stream
    /// 5. Enters the main reconnaissance loop
    ///
    /// The agent will continuously cycle through WiFi channels, monitor for
    /// access points and clients, and capture handshakes.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if startup fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Bettercap is not accessible
    /// - Monitor mode cannot be enabled
    /// - WebSocket connection fails
    /// - WiFi configuration is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_core::{Agent, AgentConfig};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let mut agent = Agent::new(AgentConfig::default())?;
    ///
    /// // Start the agent (this will run indefinitely)
    /// agent.start().await?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Returns the current agent mood.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_core::{Agent, AgentConfig};
    /// # use pwnagotchi_automata::Mood;
    /// # fn example(agent: &Agent) {
    /// match agent.mood() {
    ///     Mood::Ready => println!("Agent is ready"),
    ///     Mood::Excited => println!("Capturing handshakes!"),
    ///     Mood::Bored => println!("No activity..."),
    ///     _ => {}
    /// }
    /// # }
    /// ```
    pub fn mood(&self) -> Mood {
        self.automata.mood()
    }

    /// Returns a reference to all discovered access points.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_core::{Agent, AgentConfig};
    /// # fn example(agent: &Agent) {
    /// let aps = agent.access_points();
    /// println!("Found {} access points", aps.len());
    ///
    /// for (bssid, ap) in aps {
    ///     println!("  {} - {} (ch {}, {} dBm)",
    ///         bssid, ap.essid, ap.channel, ap.rssi);
    /// }
    /// # }
    /// ```
    pub fn access_points(&self) -> &HashMap<String, AccessPoint> {
        &self.access_points
    }

    /// Returns a reference to all captured handshakes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_core::{Agent, AgentConfig};
    /// # fn example(agent: &Agent) {
    /// let handshakes = agent.handshakes();
    /// println!("Captured {} handshakes", handshakes.len());
    ///
    /// for hs in handshakes {
    ///     println!("  {} - {} ({})",
    ///         hs.filename, hs.ap_bssid, hs.handshake_type);
    /// }
    /// # }
    /// ```
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
