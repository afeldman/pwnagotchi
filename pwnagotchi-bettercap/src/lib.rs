//! Bettercap REST API client library.
//!
//! This crate provides a Rust client for interacting with the Bettercap REST API and WebSocket event stream.
//! It supports session management, module control, and real-time event processing.
//!
//! # Examples
//!
//! ```no_run
//! use pwnagotchi_bettercap::BettercapClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create a client
//!     let client = BettercapClient::new(
//!         "localhost",
//!         "http",
//!         8081,
//!         "pwnagotchi",
//!         "pwnagotchi"
//!     )?;
//!
//!     // Get session info
//!     let session = client.session().await?;
//!     println!("Session active: {}", session.active);
//!
//!     // Run a command
//!     client.run("wifi.recon on").await?;
//!
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use futures::StreamExt;
use reqwest::{Client as HttpClient, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

/// Represents a Bettercap session with interface and module information.
///
/// Contains the current state of the Bettercap session including all available
/// network interfaces and loaded modules.
///
/// # Examples
///
/// ```no_run
/// # use pwnagotchi_bettercap::BettercapClient;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
/// let session = client.session().await?;
///
/// println!("Session started at: {}", session.started_at);
/// println!("Number of interfaces: {}", session.interfaces.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub interfaces: Vec<Interface>,
    pub modules: HashMap<String, ModuleInfo>,
    pub started_at: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interface {
    pub name: String,
    pub index: u32,
    pub mac: String,
    pub ip: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleInfo {
    pub name: String,
    pub running: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Event received from Bettercap WebSocket stream.
///
/// Events are sent by Bettercap in real-time as various actions occur
/// (e.g., new APs discovered, handshakes captured, deauth frames sent).
///
/// # Examples
///
/// ```no_run
/// # use pwnagotchi_bettercap::BettercapClient;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
/// let mut events = client.start_websocket().await?;
///
/// while let Some(event) = events.recv().await {
///     match event.tag.as_str() {
///         "wifi.ap.new" => println!("New AP: {:?}", event.data),
///         "wifi.client.handshake" => println!("Handshake: {:?}", event.data),
///         _ => println!("Event: {}", event.tag),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BettercapEvent {
    /// Event type tag (e.g., "wifi.ap.new", "wifi.client.handshake")
    pub tag: String,
    /// Event payload containing structured data
    pub data: Value,
    /// Timestamp when the event occurred
    pub time: String,
}

/// Bettercap REST API client for HTTP and WebSocket communication.
///
/// Provides methods to interact with Bettercap's REST API including:
/// - Session management
/// - Command execution
/// - Module control (start, stop, restart)
/// - Real-time WebSocket event streaming
///
/// # Examples
///
/// ## Basic Usage
///
/// ```no_run
/// # use pwnagotchi_bettercap::BettercapClient;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = BettercapClient::new(
///     "127.0.0.1",
///     "http",
///     8081,
///     "pwnagotchi",
///     "pwnagotchi"
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// ## Running Commands
///
/// ```no_run
/// # use pwnagotchi_bettercap::BettercapClient;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
/// // Start WiFi reconnaissance
/// client.run("wifi.recon on").await?;
///
/// // Set WiFi interface
/// client.run("set wifi.interface wlan0mon").await?;
/// # Ok(())
/// # }
/// ```
pub struct BettercapClient {
    http: HttpClient,
    base_url: Url,
    ws_url: Url,
    username: String,
    password: String,
}

impl BettercapClient {
    /// Creates a new Bettercap client.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname or IP address of the Bettercap server
    /// * `scheme` - HTTP scheme: "http" or "https"
    /// * `port` - Port number (typically 8081)
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the client or an error if URL parsing fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// // Connect to local Bettercap instance
    /// let client = BettercapClient::new(
    ///     "127.0.0.1",
    ///     "http",
    ///     8081,
    ///     "pwnagotchi",
    ///     "pwnagotchi"
    /// )?;
    ///
    /// // Connect with HTTPS
    /// let secure = BettercapClient::new(
    ///     "192.168.1.100",
    ///     "https",
    ///     8443,
    ///     "admin",
    ///     "password"
    /// )?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(
        hostname: &str,
        scheme: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let base = format!("{}://{}:{}/api", scheme, hostname, port);
        let ws_base = format!(
            "{}://{}:{}",
            if scheme == "https" { "wss" } else { "ws" },
            hostname,
            port
        );

        Ok(Self {
            http: HttpClient::builder().build()?,
            base_url: Url::parse(&base)?,
            ws_url: Url::parse(&ws_base)?,
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    /// Gets current session information from Bettercap.
    ///
    /// Retrieves details about the active Bettercap session including network interfaces,
    /// loaded modules, and session status.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Session` or an error if the request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// let session = client.session().await?;
    ///
    /// println!("Session ID: {}", session.id);
    /// println!("Started at: {}", session.started_at);
    /// println!("Active: {}", session.active);
    ///
    /// for iface in &session.interfaces {
    ///     println!("Interface: {} ({})", iface.name, iface.mac);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn session(&self) -> Result<Session> {
        let url = self.base_url.join("session")?;

        let response = self
            .http
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        let session: Session = response.json().await?;
        Ok(session)
    }

    /// Executes a Bettercap command.
    ///
    /// Sends a command to the Bettercap REST API for execution. Commands can control
    /// WiFi modules, configure settings, or perform actions like starting reconnaissance.
    ///
    /// # Arguments
    ///
    /// * `command` - The Bettercap command string to execute (e.g., "wifi.recon on")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `CommandResult` with success status and optional error message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// // Start WiFi reconnaissance
    /// client.run("wifi.recon on").await?;
    ///
    /// // Configure interface
    /// client.run("set wifi.interface wlan0mon").await?;
    ///
    /// // Clear WiFi data
    /// client.run("wifi.clear").await?;
    ///
    /// // Stop reconnaissance
    /// client.run("wifi.recon off").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(&self, command: &str) -> Result<CommandResult> {
        let url = self.base_url.join("session")?;

        let mut cmd = HashMap::new();
        cmd.insert("cmd", command);

        debug!("Running bettercap command: {}", command);

        let response = self
            .http
            .post(url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&cmd)
            .send()
            .await?;

        let result: CommandResult = response.json().await?;

        if !result.success {
            warn!("Command failed: {:?}", result.error);
        }

        Ok(result)
    }

    /// Checks if a Bettercap module is currently running.
    ///
    /// # Arguments
    ///
    /// * `module` - The module name to check (e.g., "wifi", "ble", "ticker")
    ///
    /// # Returns
    ///
    /// Returns `true` if the module is running, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// if client.is_module_running("wifi").await? {
    ///     println!("WiFi module is active");
    /// }
    ///
    /// if !client.is_module_running("ble").await? {
    ///     println!("BLE module is not running");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_module_running(&self, module: &str) -> Result<bool> {
        let session = self.session().await?;
        Ok(session
            .modules
            .get(module)
            .map(|m| m.running)
            .unwrap_or(false))
    }

    /// Starts a Bettercap module.
    ///
    /// # Arguments
    ///
    /// * `module` - The module name to start (e.g., "wifi.recon", "ble.recon")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the command fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// // Start WiFi reconnaissance
    /// client.start_module("wifi.recon").await?;
    ///
    /// // Start BLE scanning
    /// client.start_module("ble.recon").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_module(&self, module: &str) -> Result<()> {
        info!("Starting module: {}", module);
        self.run(&format!("{} on", module)).await?;
        Ok(())
    }

    /// Stop a bettercap module
    pub async fn stop_module(&self, module: &str) -> Result<()> {
        info!("Stopping module: {}", module);
        self.run(&format!("{} off", module)).await?;
        Ok(())
    }

    /// Restarts a Bettercap module.
    ///
    /// Stops the module, waits 500ms, then starts it again. Useful for resetting module state.
    ///
    /// # Arguments
    ///
    /// * `module` - The module name to restart
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if stop/start fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// // Restart WiFi reconnaissance
    /// client.restart_module("wifi.recon").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restart_module(&self, module: &str) -> Result<()> {
        info!("Restarting module: {}", module);
        self.stop_module(module).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start_module(module).await?;
        Ok(())
    }

    /// Starts WebSocket event stream from Bettercap.
    ///
    /// Opens a WebSocket connection to receive real-time events from Bettercap.
    /// Events include new APs, handshakes, deauths, and other WiFi/BLE activities.
    ///
    /// The connection automatically reconnects if it drops.
    ///
    /// # Returns
    ///
    /// Returns an `UnboundedReceiver` that yields `BettercapEvent` messages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_bettercap::BettercapClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;
    /// let mut events = client.start_websocket().await?;
    ///
    /// // Process events
    /// while let Some(event) = events.recv().await {
    ///     match event.tag.as_str() {
    ///         "wifi.ap.new" => {
    ///             println!("New AP discovered: {:?}", event.data);
    ///         }
    ///         "wifi.client.handshake" => {
    ///             println!("Handshake captured: {:?}", event.data);
    ///         }
    ///         "wifi.client.probe" => {
    ///             println!("Client probe: {:?}", event.data);
    ///         }
    ///         _ => {
    ///             println!("Event {}: {:?}", event.tag, event.data);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_websocket(&self) -> Result<mpsc::UnboundedReceiver<BettercapEvent>> {
        let ws_url = self.ws_url.join("/api/events")?;
        let (tx, rx) = mpsc::unbounded_channel();

        let url_str = ws_url.to_string();

        tokio::spawn(async move {
            loop {
                info!("Connecting to bettercap websocket: {}", url_str);

                match connect_async(&url_str).await {
                    Ok((ws_stream, _)) => {
                        info!("Websocket connected");
                        let (_write, mut read) = ws_stream.split();

                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    match serde_json::from_str::<BettercapEvent>(&text) {
                                        Ok(event) => {
                                            if tx.send(event).is_err() {
                                                error!("Event receiver dropped");
                                                return;
                                            }
                                        }
                                        Err(e) => {
                                            debug!("Failed to parse event: {}", e);
                                        }
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    warn!("Websocket closed by server");
                                    break;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Websocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to websocket: {}", e);
                    }
                }

                // Reconnect after delay
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BettercapClient::new("localhost", "http", 8081, "user", "pass");
        assert!(client.is_ok());
    }
}
