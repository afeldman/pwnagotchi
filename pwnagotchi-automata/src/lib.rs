use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Epoch tracking for agent activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    pub epoch: u64,
    pub inactive_for: u32,
    pub active_for: u32,
    pub bored_for: u32,
    pub sad_for: u32,
    pub blind_for: u32,
    pub num_missed: u32,
    pub num_handshakes: u32,
    pub num_associations: u32,
    pub num_deauths: u32,
}

impl Epoch {
    pub fn new() -> Self {
        Self {
            epoch: 0,
            inactive_for: 0,
            active_for: 0,
            bored_for: 0,
            sad_for: 0,
            blind_for: 0,
            num_missed: 0,
            num_handshakes: 0,
            num_associations: 0,
            num_deauths: 0,
        }
    }

    pub fn any_activity(&self) -> bool {
        self.num_handshakes > 0 || self.num_associations > 0 || self.num_deauths > 0
    }

    pub fn next(&mut self) {
        self.epoch += 1;
        
        if self.any_activity() {
            self.active_for += 1;
            self.inactive_for = 0;
            self.bored_for = 0;
            self.sad_for = 0;
        } else {
            self.inactive_for += 1;
            self.active_for = 0;
            
            if self.inactive_for >= 10 {
                self.bored_for += 1;
            }
            if self.inactive_for >= 30 {
                self.sad_for += 1;
            }
        }

        // Reset counters
        self.num_missed = 0;
        self.num_handshakes = 0;
        self.num_associations = 0;
        self.num_deauths = 0;
    }

    pub fn track_miss(&mut self) {
        self.num_missed += 1;
    }

    pub fn track_handshake(&mut self) {
        self.num_handshakes += 1;
    }

    pub fn track_association(&mut self) {
        self.num_associations += 1;
    }

    pub fn track_deauth(&mut self) {
        self.num_deauths += 1;
    }
}

/// Agent mood/state representing the current emotional state.
///
/// The mood changes based on activity patterns, handshake captures, and peer interactions.
/// Each mood affects the agent's behavior and display.
///
/// # Mood Transitions
///
/// - `Starting` → `Ready`: Agent initialization complete
/// - `Ready` → `Bored`: No activity for `bored_num_epochs`
/// - `Bored` → `Sad`: Extended inactivity for `sad_num_epochs`
/// - `Sad` → `Angry`: Prolonged inactivity beyond sad threshold
/// - `Ready` → `Excited`: High activity (handshakes captured)
/// - `Ready` → `Grateful`: Good peer support (bond encounters)
/// - `Ready` → `Lonely`: No peer support detected
/// - Any → `Rebooting`: System restart triggered
///
/// # Examples
///
/// ```
/// use pwnagotchi_automata::Mood;
///
/// let mood = Mood::Ready;
/// match mood {
///     Mood::Excited => println!("Capturing handshakes!"),
///     Mood::Bored => println!("No activity..."),
///     Mood::Ready => println!("Operational"),
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Starting,
    Ready,
    Bored,
    Sad,
    Angry,
    Excited,
    Grateful,
    Lonely,
    Rebooting,
}

/// Personality configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PersonalityConfig {
    pub bond_encounters_factor: f32,
    pub bored_num_epochs: u32,
    pub sad_num_epochs: u32,
    pub excited_num_epochs: u32,
    pub max_misses_for_recon: u32,
    pub max_inactive_scale: u32,
    pub recon_inactive_multiplier: f32,
    pub recon_time: u32,
    pub channels: Vec<u8>,
    pub ap_ttl: u32,
    pub sta_ttl: u32,
    pub min_rssi: i32,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            bond_encounters_factor: 20000.0,
            bored_num_epochs: 15,
            sad_num_epochs: 25,
            excited_num_epochs: 10,
            max_misses_for_recon: 5,
            max_inactive_scale: 10,
            recon_inactive_multiplier: 2.0,
            recon_time: 30,
            channels: vec![],
            ap_ttl: 120,
            sta_ttl: 300,
            min_rssi: -200,
        }
    }
}

/// Peer information for mesh networking
#[derive(Debug, Clone)]
pub struct Peer {
    pub fingerprint: String,
    pub encounters: u32,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// State machine for agent behavior and mood management.
///
/// The automata tracks activity across epochs and transitions between moods
/// based on configuration parameters and peer interactions.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use pwnagotchi_automata::{Automata, PersonalityConfig, Mood};
///
/// let config = PersonalityConfig::default();
/// let mut automata = Automata::new(config);
///
/// // Initialize
/// automata.set_starting();
/// automata.set_ready();
///
/// assert_eq!(automata.mood(), Mood::Ready);
/// ```
///
/// ## Tracking Activity
///
/// ```
/// use pwnagotchi_automata::{Automata, PersonalityConfig};
///
/// let config = PersonalityConfig::default();
/// let mut automata = Automata::new(config);
/// automata.set_ready();
///
/// // Simulate activity
/// {
///     let epoch = automata.epoch_mut();
///     epoch.track_handshake();
///     epoch.track_association();
/// }
///
/// // Process epoch - mood may change to Excited
/// automata.next_epoch();
/// ```
///
/// ## Peer Management
///
/// ```
/// use pwnagotchi_automata::{Automata, PersonalityConfig};
///
/// let config = PersonalityConfig::default();
/// let mut automata = Automata::new(config);
///
/// // Add peer
/// automata.add_peer("aa:bb:cc:dd:ee:ff".to_string());
///
/// // Update peer encounter
/// automata.update_peer("aa:bb:cc:dd:ee:ff".to_string());
///
/// // Check peers
/// let peers = automata.peers();
/// assert_eq!(peers.len(), 1);
/// ```
pub struct Automata {
    config: PersonalityConfig,
    epoch: Epoch,
    mood: Mood,
    peers: HashMap<String, Peer>,
}

impl Automata {
    /// Creates a new automata with the given personality configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Personality configuration defining behavior parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use pwnagotchi_automata::{Automata, PersonalityConfig};
    ///
    /// // Use default config
    /// let automata = Automata::new(PersonalityConfig::default());
    ///
    /// // Custom config
    /// let mut config = PersonalityConfig::default();
    /// config.bored_num_epochs = 10;
    /// config.recon_time = 60;
    /// let custom_automata = Automata::new(config);
    /// ```
    pub fn new(config: PersonalityConfig) -> Self {
        Self {
            config,
            epoch: Epoch::new(),
            mood: Mood::Starting,
            peers: HashMap::new(),
        }
    }

    /// Returns a reference to the current epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pwnagotchi_automata::{Automata, PersonalityConfig};
    /// # let automata = Automata::new(PersonalityConfig::default());
    /// let epoch = automata.epoch();
    /// println!("Epoch: {}, Handshakes: {}", epoch.epoch, epoch.num_handshakes);
    /// ```
    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    /// Returns the current mood.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pwnagotchi_automata::{Automata, PersonalityConfig, Mood};
    /// # let automata = Automata::new(PersonalityConfig::default());
    /// match automata.mood() {
    ///     Mood::Ready => println!("Agent is ready"),
    ///     Mood::Excited => println!("Agent is excited!"),
    ///     _ => {}
    /// }
    /// ```
    pub fn mood(&self) -> Mood {
        self.mood
    }

    /// Sets the mood to Starting.
    ///
    /// Called during agent initialization.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pwnagotchi_automata::{Automata, PersonalityConfig, Mood};
    /// let mut automata = Automata::new(PersonalityConfig::default());
    /// automata.set_starting();
    /// assert_eq!(automata.mood(), Mood::Starting);
    /// ```
    pub fn set_starting(&mut self) {
        self.mood = Mood::Starting;
        info!("Agent state: Starting");
    }

    /// Sets the mood to Ready.
    ///
    /// Called when agent is ready to begin operations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pwnagotchi_automata::{Automata, PersonalityConfig, Mood};
    /// let mut automata = Automata::new(PersonalityConfig::default());
    /// automata.set_ready();
    /// assert_eq!(automata.mood(), Mood::Ready);
    /// ```
    pub fn set_ready(&mut self) {
        self.mood = Mood::Ready;
        info!("Agent state: Ready");
    }

    pub fn on_miss(&mut self, who: &str) {
        info!("It looks like {} is not in range anymore", who);
        self.epoch.track_miss();
    }

    pub fn in_good_mood(&self) -> bool {
        self.has_support_network_for(1.0)
    }

    fn has_support_network_for(&self, factor: f32) -> bool {
        let total_encounters: u32 = self.peers.values().map(|p| p.encounters).sum();
        let support_factor = total_encounters as f32 / self.config.bond_encounters_factor;
        support_factor >= factor
    }

    pub fn set_grateful(&mut self) {
        self.mood = Mood::Grateful;
        info!("Agent state: Grateful");
    }

    pub fn set_lonely(&mut self) {
        if !self.has_support_network_for(1.0) {
            self.mood = Mood::Lonely;
            info!("Agent state: Lonely");
        } else {
            info!("Unit is grateful instead of lonely");
            self.set_grateful();
        }
    }

    pub fn set_bored(&mut self) {
        let factor = self.epoch.inactive_for as f32 / self.config.bored_num_epochs as f32;
        if !self.has_support_network_for(factor) {
            self.mood = Mood::Bored;
            warn!("{} epochs with no activity -> bored", self.epoch.inactive_for);
        } else {
            info!("Unit is grateful instead of bored");
            self.set_grateful();
        }
    }

    pub fn set_sad(&mut self) {
        let factor = self.epoch.inactive_for as f32 / self.config.sad_num_epochs as f32;
        if !self.has_support_network_for(factor) {
            self.mood = Mood::Sad;
            warn!("{} epochs with no activity -> sad", self.epoch.inactive_for);
        } else {
            info!("Unit is grateful instead of sad");
            self.set_grateful();
        }
    }

    pub fn set_angry(&mut self, factor: f32) {
        if !self.has_support_network_for(factor) {
            self.mood = Mood::Angry;
            warn!("{} epochs with no activity -> angry", self.epoch.inactive_for);
        } else {
            info!("Unit is grateful instead of angry");
            self.set_grateful();
        }
    }

    pub fn set_excited(&mut self) {
        self.mood = Mood::Excited;
        warn!("{} epochs with activity -> excited", self.epoch.active_for);
    }

    pub fn set_rebooting(&mut self) {
        self.mood = Mood::Rebooting;
        info!("Agent state: Rebooting");
    }

    pub fn is_stale(&self) -> bool {
        self.epoch.num_missed > self.config.max_misses_for_recon
    }

    pub fn any_activity(&self) -> bool {
        self.epoch.any_activity()
    }

    /// Processes the current epoch and transitions to the next.
    ///
    /// This method analyzes activity metrics and adjusts the mood based on:
    /// - Activity levels (inactive_for, active_for)
    /// - Missed interactions
    /// - Peer support network
    /// - Configuration thresholds
    ///
    /// Possible mood transitions:
    /// - High activity → `Excited`
    /// - Good peer support → `Grateful`
    /// - Inactivity → `Bored` → `Sad` → `Angry`
    /// - Many misses → `Lonely` → `Angry`
    ///
    /// # Examples
    ///
    /// ```
    /// use pwnagotchi_automata::{Automata, PersonalityConfig, Mood};
    ///
    /// let mut automata = Automata::new(PersonalityConfig::default());
    /// automata.set_ready();
    ///
    /// // Simulate activity
    /// for _ in 0..5 {
    ///     {
    ///         let epoch = automata.epoch_mut();
    ///         epoch.track_handshake();
    ///     }
    ///     automata.next_epoch();
    /// }
    ///
    /// // After several active epochs, mood may change to Excited
    /// // depending on configuration
    /// ```
    pub fn next_epoch(&mut self) {
        debug!("automata.next_epoch()");

        let was_stale = self.is_stale();
        let did_miss = self.epoch.num_missed;

        self.epoch.next();

        // After X misses during an epoch, set status to lonely or angry
        if was_stale {
            let factor = did_miss as f32 / self.config.max_misses_for_recon as f32;
            if factor >= 2.0 {
                self.set_angry(factor);
            } else {
                warn!("Agent missed {} interactions -> lonely", did_miss);
                self.set_lonely();
            }
        }
        // After X times being bored, set status to sad or angry
        else if self.epoch.sad_for > 0 {
            let factor = self.epoch.inactive_for as f32 / self.config.sad_num_epochs as f32;
            if factor >= 2.0 {
                self.set_angry(factor);
            } else {
                self.set_sad();
            }
        }
        // After X times being inactive, set status to bored
        else if self.epoch.bored_for > 0 {
            self.set_bored();
        }
        // After X times being active, set status to happy/excited
        else if self.epoch.active_for >= self.config.excited_num_epochs {
            self.set_excited();
        } else if self.epoch.active_for >= 5 && self.has_support_network_for(5.0) {
            self.set_grateful();
        }
    }

    pub fn add_peer(&mut self, fingerprint: String) {
        let peer = self.peers.entry(fingerprint.clone()).or_insert_with(|| Peer {
            fingerprint: fingerprint.clone(),
            encounters: 0,
            last_seen: chrono::Utc::now(),
        });
        peer.encounters += 1;
        peer.last_seen = chrono::Utc::now();
    }

    pub fn remove_stale_peers(&mut self, max_age: chrono::Duration) {
        let now = chrono::Utc::now();
        self.peers.retain(|_, peer| now - peer.last_seen < max_age);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_tracking() {
        let mut epoch = Epoch::new();
        assert_eq!(epoch.epoch, 0);
        assert!(!epoch.any_activity());

        epoch.track_handshake();
        assert!(epoch.any_activity());

        epoch.next();
        assert_eq!(epoch.epoch, 1);
        assert_eq!(epoch.active_for, 1);
    }

    #[test]
    fn test_mood_transitions() {
        let config = PersonalityConfig::default();
        let mut automata = Automata::new(config);

        automata.set_starting();
        assert_eq!(automata.mood(), Mood::Starting);

        automata.set_ready();
        assert_eq!(automata.mood(), Mood::Ready);

        automata.set_bored();
        assert_eq!(automata.mood(), Mood::Bored);
    }
}
