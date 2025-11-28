//! Epoch tracking and state management
//!
//! Tracks WiFi environment observations and agent actions during training epochs

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Number of WiFi channels (2.4GHz: 1-14)
pub const NUM_CHANNELS: usize = 14;

/// Complete epoch data including observations and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochData {
    /// Current epoch number
    pub epoch: u32,
    /// Epoch duration in seconds
    pub duration: f64,
    /// Access points histogram by channel
    pub aps_histogram: [f32; NUM_CHANNELS],
    /// Client stations histogram by channel
    pub sta_histogram: [f32; NUM_CHANNELS],
    /// Peer units histogram by channel
    pub peers_histogram: [f32; NUM_CHANNELS],
    /// Number of handshakes captured
    pub num_handshakes: u32,
    /// Number of deauth attempts
    pub num_deauths: u32,
    /// Number of association attempts
    pub num_assocs: u32,
    /// Number of channel hops
    pub num_hops: u32,
    /// Number of missed interactions
    pub num_missed: u32,
    /// Consecutive epochs with activity
    pub active_for: u32,
    /// Consecutive epochs without activity
    pub inactive_for: u32,
    /// Consecutive epochs without visible APs
    pub blind_for: u32,
    /// Consecutive epochs in sad state
    pub sad_for: u32,
    /// Consecutive epochs in bored state
    pub bored_for: u32,
    /// Number of peers detected
    pub num_peers: u32,
    /// Total bond factor with peers
    pub tot_bond_factor: f32,
    /// Average bond factor with peers
    pub avg_bond_factor: f32,
    /// CPU load (0.0 - 1.0)
    pub cpu_load: f32,
    /// Memory usage (0.0 - 1.0)
    pub mem_usage: f32,
    /// Temperature in Celsius
    pub temperature: f32,
    /// Calculated reward
    pub reward: f32,
}

/// Epoch tracker for reinforcement learning
///
/// Manages the state of a single training epoch, including:
/// - WiFi environment observations (APs, clients, peers by channel)
/// - Agent actions (deauths, associations, channel hops)
/// - Emotional state (bored, sad, active)
/// - System metrics (CPU, memory, temperature)
///
/// ## Example
///
/// ```rust
/// use pwnagotchi_ai::epoch::Epoch;
///
/// let mut epoch = Epoch::new();
///
/// // Track events during the epoch
/// epoch.track_deauth(2);
/// epoch.track_association(1);
/// epoch.track_handshake(1);
/// epoch.track_channel_hop();
///
/// // Get epoch data for reward calculation
/// let data = epoch.next();
/// println!("Epoch {} completed with {} handshakes", data.epoch, data.num_handshakes);
/// ```
pub struct Epoch {
    /// Current epoch number
    epoch: u32,
    /// When the current epoch started
    epoch_started: Instant,
    /// Duration of last epoch
    epoch_duration: Duration,

    // Activity tracking
    /// Did deauth in current channel?
    did_deauth: bool,
    /// Total deauths in epoch
    num_deauths: u32,
    /// Did associate in current channel?
    did_associate: bool,
    /// Total associations in epoch
    num_assocs: u32,
    /// Did capture handshakes?
    did_handshakes: bool,
    /// Total handshakes in epoch
    num_handshakes: u32,
    /// Total channel hops
    num_hops: u32,
    /// Total missed interactions
    num_missed: u32,
    /// Any activity at all?
    any_activity: bool,

    // State tracking
    /// Consecutive inactive epochs
    inactive_for: u32,
    /// Consecutive active epochs
    active_for: u32,
    /// Consecutive blind epochs
    blind_for: u32,
    /// Consecutive sad epochs
    sad_for: u32,
    /// Consecutive bored epochs
    bored_for: u32,

    // Peer tracking
    num_peers: u32,
    tot_bond_factor: f32,

    // Observation histograms
    aps_histogram: [f32; NUM_CHANNELS],
    sta_histogram: [f32; NUM_CHANNELS],
    peers_histogram: [f32; NUM_CHANNELS],
}

impl Epoch {
    /// Create a new epoch tracker
    pub fn new() -> Self {
        Self {
            epoch: 0,
            epoch_started: Instant::now(),
            epoch_duration: Duration::ZERO,
            did_deauth: false,
            num_deauths: 0,
            did_associate: false,
            num_assocs: 0,
            did_handshakes: false,
            num_handshakes: 0,
            num_hops: 0,
            num_missed: 0,
            any_activity: false,
            inactive_for: 0,
            active_for: 0,
            blind_for: 0,
            sad_for: 0,
            bored_for: 0,
            num_peers: 0,
            tot_bond_factor: 0.0,
            aps_histogram: [0.0; NUM_CHANNELS],
            sta_histogram: [0.0; NUM_CHANNELS],
            peers_histogram: [0.0; NUM_CHANNELS],
        }
    }

    /// Track a deauth event
    pub fn track_deauth(&mut self, count: u32) {
        self.num_deauths += count;
        self.did_deauth = true;
        self.any_activity = true;
    }

    /// Track an association event
    pub fn track_association(&mut self, count: u32) {
        self.num_assocs += count;
        self.did_associate = true;
        self.any_activity = true;
    }

    /// Track a handshake capture
    pub fn track_handshake(&mut self, count: u32) {
        self.num_handshakes += count;
        self.did_handshakes = true;
        self.any_activity = true;
    }

    /// Track a channel hop
    pub fn track_channel_hop(&mut self) {
        self.num_hops += 1;
        // Reset per-channel flags
        self.did_deauth = false;
        self.did_associate = false;
    }

    /// Track a missed interaction
    pub fn track_miss(&mut self, count: u32) {
        self.num_missed += count;
    }

    /// Update observation histograms
    ///
    /// # Arguments
    ///
    /// * `aps` - Access points by channel
    /// * `clients` - Client stations by channel
    /// * `peers` - Peer units by channel
    pub fn observe(&mut self, aps: &[usize], clients: &[usize], peers: &[usize]) {
        // Normalize histograms
        let max_aps = *aps.iter().max().unwrap_or(&1) as f32;
        let max_clients = *clients.iter().max().unwrap_or(&1) as f32;
        let max_peers = *peers.iter().max().unwrap_or(&1) as f32;

        for (i, &count) in aps.iter().enumerate().take(NUM_CHANNELS) {
            self.aps_histogram[i] = (count as f32) / max_aps;
        }
        for (i, &count) in clients.iter().enumerate().take(NUM_CHANNELS) {
            self.sta_histogram[i] = (count as f32) / max_clients;
        }
        for (i, &count) in peers.iter().enumerate().take(NUM_CHANNELS) {
            self.peers_histogram[i] = (count as f32) / max_peers;
        }
    }

    /// Update peer statistics
    pub fn update_peers(&mut self, num_peers: u32, bond_factor: f32) {
        self.num_peers = num_peers;
        self.tot_bond_factor += bond_factor;
    }

    /// Advance to the next epoch and return data for the completed epoch
    ///
    /// This calculates reward, updates state counters, and resets for the next epoch.
    pub fn next(&mut self, reward: f32) -> EpochData {
        let now = Instant::now();
        self.epoch_duration = now - self.epoch_started;

        // Update activity state
        if !self.any_activity && !self.did_handshakes {
            self.inactive_for += 1;
            self.active_for = 0;
        } else {
            self.active_for += 1;
            self.inactive_for = 0;
            self.sad_for = 0;
            self.bored_for = 0;
        }

        // Check if blind (no APs visible)
        let has_aps = self.aps_histogram.iter().any(|&v| v > 0.0);
        if !has_aps {
            self.blind_for += 1;
        } else {
            self.blind_for = 0;
        }

        // Calculate average bond factor
        let avg_bond = if self.num_peers > 0 {
            self.tot_bond_factor / (self.num_peers as f32)
        } else {
            0.0
        };

        // Get system metrics (placeholder - would be real in production)
        let cpu_load = 0.5; // Would query actual CPU
        let mem_usage = 0.6; // Would query actual memory
        let temperature = 45.0; // Would query actual temp

        let data = EpochData {
            epoch: self.epoch,
            duration: self.epoch_duration.as_secs_f64(),
            aps_histogram: self.aps_histogram,
            sta_histogram: self.sta_histogram,
            peers_histogram: self.peers_histogram,
            num_handshakes: self.num_handshakes,
            num_deauths: self.num_deauths,
            num_assocs: self.num_assocs,
            num_hops: self.num_hops,
            num_missed: self.num_missed,
            active_for: self.active_for,
            inactive_for: self.inactive_for,
            blind_for: self.blind_for,
            sad_for: self.sad_for,
            bored_for: self.bored_for,
            num_peers: self.num_peers,
            tot_bond_factor: self.tot_bond_factor,
            avg_bond_factor: avg_bond,
            cpu_load,
            mem_usage,
            temperature,
            reward,
        };

        // Reset for next epoch
        self.epoch += 1;
        self.epoch_started = now;
        self.did_deauth = false;
        self.num_deauths = 0;
        self.did_associate = false;
        self.num_assocs = 0;
        self.did_handshakes = false;
        self.num_handshakes = 0;
        self.num_hops = 0;
        self.num_missed = 0;
        self.any_activity = false;
        self.num_peers = 0;
        self.tot_bond_factor = 0.0;

        data
    }

    /// Get current epoch number
    pub fn current_epoch(&self) -> u32 {
        self.epoch
    }

    /// Update emotional state
    pub fn update_emotions(&mut self, sad: bool, bored: bool) {
        if sad {
            self.sad_for += 1;
        }
        if bored {
            self.bored_for += 1;
        }
    }
}

impl Default for Epoch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_tracking() {
        let mut epoch = Epoch::new();
        assert_eq!(epoch.current_epoch(), 0);

        epoch.track_deauth(5);
        epoch.track_association(3);
        epoch.track_handshake(2);
        epoch.track_channel_hop();

        let data = epoch.next(0.5);
        assert_eq!(data.num_deauths, 5);
        assert_eq!(data.num_assocs, 3);
        assert_eq!(data.num_handshakes, 2);
        assert_eq!(data.num_hops, 1);
        assert_eq!(data.active_for, 1);
        assert_eq!(epoch.current_epoch(), 1);
    }

    #[test]
    fn test_activity_state() {
        let mut epoch = Epoch::new();

        // First epoch with activity
        epoch.track_handshake(1);
        let data1 = epoch.next(0.5);
        assert_eq!(data1.active_for, 1);
        assert_eq!(data1.inactive_for, 0);

        // Second epoch without activity
        let data2 = epoch.next(0.0);
        assert_eq!(data2.active_for, 0);
        assert_eq!(data2.inactive_for, 1);
    }

    #[test]
    fn test_observation_histogram() {
        let mut epoch = Epoch::new();

        let aps = vec![5, 10, 3, 0, 0, 8, 0, 0, 0, 0, 12, 0, 0, 0];
        let clients = vec![2, 5, 1, 0, 0, 3, 0, 0, 0, 0, 4, 0, 0, 0];
        let peers = vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0];

        epoch.observe(&aps, &clients, &peers);

        // Channel 11 (index 10) has most APs (12)
        assert!(epoch.aps_histogram[10] > epoch.aps_histogram[0]);
    }
}
