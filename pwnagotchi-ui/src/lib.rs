//! UI system for Pwnagotchi display rendering.
//!
//! This crate provides a display abstraction and rendering system for e-ink displays.
//! It includes face expressions, status rendering, and a trait-based display system.
//!
//! # Architecture
//!
//! - **Display trait**: Generic interface for different display types
//! - **ViewState**: Current UI state (mood, status, statistics)
//! - **Face expressions**: ASCII art faces for different moods
//!
//! # Examples
//!
//! ## Using ViewState
//!
//! ```
//! use pwnagotchi_ui::{ViewState, faces};
//! use pwnagotchi_automata::Mood;
//!
//! let mut view = ViewState::default();
//!
//! // Update mood
//! view.set_mood(Mood::Excited);
//! assert_eq!(view.face, faces::EXCITED);
//!
//! // Update statistics
//! view.set_aps(15);
//! view.set_handshakes(3);
//! view.set_channel(6);
//! view.set_status("Scanning...".to_string());
//! ```
//!
//! ## Implementing Display
//!
//! ```no_run
//! use pwnagotchi_ui::{Display, ViewState};
//! use anyhow::Result;
//!
//! struct MyDisplay;
//!
//! impl Display for MyDisplay {
//!     fn init(&mut self) -> Result<()> {
//!         println!("Display initialized");
//!         Ok(())
//!     }
//!
//!     fn clear(&mut self) -> Result<()> {
//!         println!("Display cleared");
//!         Ok(())
//!     }
//!
//!     fn render(&mut self, view: &ViewState) -> Result<()> {
//!         println!("Rendering: {} - {} APs, {} handshakes",
//!             view.face, view.aps, view.handshakes);
//!         Ok(())
//!     }
//!
//!     fn update(&mut self) -> Result<()> {
//!         println!("Display updated");
//!         Ok(())
//!     }
//! }
//! ```

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use pwnagotchi_automata::Mood;

/// ASCII face expressions for different agent moods.
///
/// Each constant represents a different emotional state using Unicode characters.
///
/// # Examples
///
/// ```
/// use pwnagotchi_ui::faces;
/// use pwnagotchi_automata::Mood;
///
/// let face = match Mood::Excited {
///     Mood::Excited => faces::EXCITED,
///     Mood::Bored => faces::BORED,
///     Mood::Sad => faces::SAD,
///     _ => faces::HAPPY,
/// };
///
/// println!("Agent face: {}", face);
/// ```
pub mod faces {
    pub const HAPPY: &str = "(◕‿◕)";
    pub const SAD: &str = "(╥﹏╥)";
    pub const BORED: &str = "(-__-)";
    pub const EXCITED: &str = "(☆▽☆)";
    pub const ANGRY: &str = "(╬ಠ益ಠ)";
    pub const GRATEFUL: &str = "(✿◠‿◠)";
    pub const LONELY: &str = "(っ- ‸ – ς)";
    pub const COOL: &str = "(⌐■_■)";
    pub const SMART: &str = "(✜‿✜)";
}

/// Current state of the UI view.
///
/// Contains all information needed to render the current agent state
/// including mood, statistics, and status messages.
///
/// # Examples
///
/// ```
/// use pwnagotchi_ui::ViewState;
/// use pwnagotchi_automata::Mood;
///
/// let mut view = ViewState::default();
///
/// // Update view state
/// view.set_mood(Mood::Excited);
/// view.set_status("Capturing handshakes!".to_string());
/// view.set_aps(20);
/// view.set_handshakes(5);
/// view.set_channel(11);
/// view.set_uptime("02:30:15".to_string());
///
/// // Access view data
/// println!("Status: {}", view.status);
/// println!("Face: {}", view.face);
/// println!("Stats: {} APs, {} handshakes on channel {}",
///     view.aps, view.handshakes, view.channel);
/// ```
#[derive(Debug, Clone)]
pub struct ViewState {
    pub status: String,
    pub face: String,
    pub channel: String,
    pub aps: usize,
    pub handshakes: usize,
    pub uptime: String,
    pub mode: String,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            status: "Starting...".to_string(),
            face: faces::HAPPY.to_string(),
            channel: "0".to_string(),
            aps: 0,
            handshakes: 0,
            uptime: "00:00:00".to_string(),
            mode: "AUTO".to_string(),
        }
    }
}

impl ViewState {
    /// Update face based on mood
    pub fn set_mood(&mut self, mood: Mood) {
        self.face = match mood {
            Mood::Starting => faces::HAPPY.to_string(),
            Mood::Ready => faces::HAPPY.to_string(),
            Mood::Bored => faces::BORED.to_string(),
            Mood::Sad => faces::SAD.to_string(),
            Mood::Angry => faces::ANGRY.to_string(),
            Mood::Excited => faces::EXCITED.to_string(),
            Mood::Grateful => faces::GRATEFUL.to_string(),
            Mood::Lonely => faces::LONELY.to_string(),
            Mood::Rebooting => faces::COOL.to_string(),
        };
    }

    /// Update status message
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    /// Set channel display
    pub fn set_channel(&mut self, channel: u8) {
        self.channel = if channel == 0 {
            "*".to_string()
        } else {
            channel.to_string()
        };
    }

    /// Update AP count
    pub fn set_aps(&mut self, count: usize) {
        self.aps = count;
    }

    /// Update handshake count
    pub fn set_handshakes(&mut self, count: usize) {
        self.handshakes = count;
    }

    /// Update uptime display
    pub fn set_uptime(&mut self, uptime: String) {
        self.uptime = uptime;
    }
}

/// Display trait for e-ink screen implementations.
///
/// Implement this trait to support different display hardware.
/// The trait provides methods for initialization, clearing, rendering, and updating.
///
/// # Examples
///
/// See the crate-level documentation for a complete implementation example.
pub trait Display {
    /// Initializes the display hardware.
    ///
    /// Called once during startup to configure the display.
    ///
    /// # Errors
    ///
    /// Returns an error if hardware initialization fails.
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Clears the display buffer.
    ///
    /// Removes all content from the display, typically setting all pixels to white.
    ///
    /// # Errors
    ///
    /// Returns an error if the display cannot be cleared.
    fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Render the view state
    fn render(&mut self, state: &ViewState) -> Result<(), Box<dyn std::error::Error>>;

    /// Update the display (refresh)
    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Mock display for testing
pub struct MockDisplay {
    state: ViewState,
}

impl MockDisplay {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
        }
    }

    pub fn state(&self) -> &ViewState {
        &self.state
    }
}

impl Display for MockDisplay {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = ViewState::default();
        Ok(())
    }

    fn render(&mut self, state: &ViewState) -> Result<(), Box<dyn std::error::Error>> {
        self.state = state.clone();
        println!("=== Pwnagotchi Display ===");
        println!("Status: {}", state.status);
        println!("Face: {}", state.face);
        println!(
            "Channel: {} | APs: {} | Handshakes: {}",
            state.channel, state.aps, state.handshakes
        );
        println!("Uptime: {} | Mode: {}", state.uptime, state.mode);
        println!("========================");
        Ok(())
    }

    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Mock update - just print current state
        self.render(&self.state.clone())
    }
}

impl Default for MockDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// View manager
pub struct View {
    display: Box<dyn Display>,
    state: ViewState,
}

impl View {
    pub fn new(display: Box<dyn Display>) -> Self {
        Self {
            display,
            state: ViewState::default(),
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.display.init()
    }

    pub fn set_status(&mut self, status: &str) {
        self.state.set_status(status.to_string());
    }

    pub fn set_mood(&mut self, mood: Mood) {
        self.state.set_mood(mood);
    }

    pub fn set_channel(&mut self, channel: u8) {
        self.state.set_channel(channel);
    }

    pub fn set_aps(&mut self, count: usize) {
        self.state.set_aps(count);
    }

    pub fn set_handshakes(&mut self, count: usize) {
        self.state.set_handshakes(count);
    }

    pub fn set_uptime(&mut self, uptime: String) {
        self.state.set_uptime(uptime);
    }

    pub fn update(&mut self, force: bool) -> Result<(), Box<dyn std::error::Error>> {
        if force {
            self.display.clear()?;
        }
        self.display.render(&self.state)?;
        self.display.update()
    }

    pub fn on_starting(&mut self) {
        self.set_status("Starting...");
        self.set_mood(Mood::Starting);
        let _ = self.update(true);
    }

    pub fn on_ready(&mut self) {
        self.set_status("Ready");
        self.set_mood(Mood::Ready);
        let _ = self.update(true);
    }

    pub fn on_bored(&mut self) {
        self.set_status("Bored...");
        self.set_mood(Mood::Bored);
        let _ = self.update(true);
    }

    pub fn on_sad(&mut self) {
        self.set_status("Sad :(");
        self.set_mood(Mood::Sad);
        let _ = self.update(true);
    }

    pub fn on_excited(&mut self) {
        self.set_status("Excited!");
        self.set_mood(Mood::Excited);
        let _ = self.update(true);
    }

    pub fn on_lonely(&mut self) {
        self.set_status("Lonely...");
        self.set_mood(Mood::Lonely);
        let _ = self.update(true);
    }

    pub fn on_grateful(&mut self) {
        self.set_status("Grateful");
        self.set_mood(Mood::Grateful);
        let _ = self.update(true);
    }

    pub fn on_angry(&mut self) {
        self.set_status("Angry!");
        self.set_mood(Mood::Angry);
        let _ = self.update(true);
    }

    pub fn on_rebooting(&mut self) {
        self.set_status("Rebooting...");
        self.set_mood(Mood::Rebooting);
        let _ = self.update(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_state() {
        let mut state = ViewState::default();
        assert_eq!(state.status, "Starting...");

        state.set_mood(Mood::Excited);
        assert_eq!(state.face, faces::EXCITED);

        state.set_channel(6);
        assert_eq!(state.channel, "6");

        state.set_aps(10);
        assert_eq!(state.aps, 10);
    }

    #[test]
    fn test_mock_display() {
        let mut display = MockDisplay::new();
        display.init().unwrap();

        let state = ViewState {
            status: "Test".to_string(),
            face: faces::HAPPY.to_string(),
            channel: "6".to_string(),
            aps: 5,
            handshakes: 2,
            uptime: "01:23:45".to_string(),
            mode: "AUTO".to_string(),
        };

        display.render(&state).unwrap();
        assert_eq!(display.state().aps, 5);
    }
}
