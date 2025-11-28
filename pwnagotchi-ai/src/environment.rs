//! RL environment interface
//!
//! Provides OpenAI Gym-like interface for training

use crate::epoch::{Epoch, EpochData, NUM_CHANNELS};
use crate::reward::{EpochState, RewardFunction};
use anyhow::Result;

/// Action space - personality parameters to optimize
#[derive(Debug, Clone)]
pub struct Action {
    /// Recon time in seconds
    pub recon_time: u32,
    /// Min RSSI threshold
    pub min_rssi: i32,
    /// Channels to scan
    pub channels: Vec<u32>,
    /// Deauth enabled
    pub deauth: bool,
    /// Associate enabled
    pub associate: bool,
    /// Bored threshold epochs
    pub bored_num_epochs: u32,
    /// Sad threshold epochs
    pub sad_num_epochs: u32,
}

/// Observation from environment
#[derive(Debug, Clone)]
pub struct Observation {
    /// APs by channel (normalized)
    pub aps_histogram: [f32; NUM_CHANNELS],
    /// Clients by channel (normalized)
    pub sta_histogram: [f32; NUM_CHANNELS],
    /// Peers by channel (normalized)
    pub peers_histogram: [f32; NUM_CHANNELS],
}

/// RL Environment for WiFi pwning
///
/// Implements Gym-like interface:
/// - `step(action)` -> (observation, reward, done)
/// - `reset()` -> observation
pub struct Environment {
    epoch: Epoch,
    reward_fn: RewardFunction,
}

impl Environment {
    /// Create new environment
    pub fn new() -> Self {
        Self {
            epoch: Epoch::new(),
            reward_fn: RewardFunction::new(),
        }
    }

    /// Execute one step with the given action
    ///
    /// Returns: (observation, reward, done)
    pub async fn step(&mut self, _action: Action) -> Result<(Observation, f32, bool)> {
        // TODO: Implement actual step logic
        // - Apply action parameters
        // - Wait for epoch to complete
        // - Calculate reward
        // - Return new observation

        let obs = Observation {
            aps_histogram: [0.0; NUM_CHANNELS],
            sta_histogram: [0.0; NUM_CHANNELS],
            peers_histogram: [0.0; NUM_CHANNELS],
        };

        Ok((obs, 0.0, false))
    }

    /// Reset environment for new episode
    pub fn reset(&mut self) -> Observation {
        self.epoch = Epoch::new();
        Observation {
            aps_histogram: [0.0; NUM_CHANNELS],
            sta_histogram: [0.0; NUM_CHANNELS],
            peers_histogram: [0.0; NUM_CHANNELS],
        }
    }

    /// Get current epoch data
    pub fn get_epoch_data(&self) -> Option<EpochData> {
        None // TODO: Return actual epoch data
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
