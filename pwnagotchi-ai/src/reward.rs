//! Reward function for Pwnagotchi reinforcement learning.
//!
//! Ported from Python implementation: `pwnagotchi/ai/reward.py`
//!
//! The reward balances multiple factors:
//! - **Positive rewards**: Handshakes, activity, channel exploration
//! - **Negative penalties**: Blindness, inactivity, missed interactions, emotions (sad/bored)

use serde::{Deserialize, Serialize};

/// Reward range: (-0.7, 1.02)
pub const REWARD_MIN: f32 = -0.7;
pub const REWARD_MAX: f32 = 1.02;

/// Small constant to avoid division by zero
const EPSILON: f32 = 1e-20;

/// Total number of WiFi channels
const NUM_CHANNELS: usize = 14;

/// State information for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochState {
    /// Number of handshakes captured
    pub num_handshakes: u32,
    /// Number of deauth attempts
    pub num_deauths: u32,
    /// Number of association attempts
    pub num_associations: u32,
    /// Number of channel hops
    pub num_hops: u32,
    /// Number of missed interactions
    pub missed_interactions: u32,
    /// Consecutive epochs with activity
    pub active_for_epochs: u32,
    /// Consecutive epochs without activity
    pub inactive_for_epochs: u32,
    /// Consecutive epochs without visible APs
    pub blind_for_epochs: u32,
    /// Consecutive epochs in sad state
    pub sad_for_epochs: u32,
    /// Consecutive epochs in bored state
    pub bored_for_epochs: u32,
}

/// Reward function for A2C reinforcement learning.
///
/// Calculates a reward value between -0.7 and 1.02 based on the agent's
/// performance during an epoch.
///
/// ## Formula
///
/// ```text
/// reward = h + a + c + b + i + m + s + l
///
/// where:
///   h = handshakes / total_interactions          (main reward)
///   a = 0.2 * (active_epochs / total_epochs)     (activity bonus)
///   c = 0.1 * (hops / total_channels)            (exploration bonus)
///   b = -0.3 * (blind_epochs / total_epochs)     (blindness penalty)
///   i = -0.2 * (inactive_epochs / total_epochs)  (inactivity penalty)
///   m = -0.3 * (missed / total_interactions)     (miss penalty)
///   s = -0.2 * (sad_epochs / total_epochs)       (sadness penalty)
///   l = -0.1 * (bored_epochs / total_epochs)     (boredom penalty)
/// ```
///
/// ## Example
///
/// ```rust
/// use pwnagotchi_ai::reward::{RewardFunction, EpochState};
///
/// let reward_fn = RewardFunction::new();
/// let state = EpochState {
///     num_handshakes: 5,
///     num_deauths: 10,
///     num_associations: 8,
///     num_hops: 11,
///     missed_interactions: 2,
///     active_for_epochs: 15,
///     inactive_for_epochs: 0,
///     blind_for_epochs: 0,
///     sad_for_epochs: 0,
///     bored_for_epochs: 0,
/// };
///
/// let reward = reward_fn.calculate(100, &state);
/// assert!(reward > 0.0); // Good performance should yield positive reward
/// ```
pub struct RewardFunction;

impl RewardFunction {
    /// Creates a new reward function instance
    pub fn new() -> Self {
        Self
    }

    /// Calculate reward for the given epoch and state
    ///
    /// # Arguments
    ///
    /// * `epoch_n` - Current epoch number (used for normalization)
    /// * `state` - State information from the epoch
    ///
    /// # Returns
    ///
    /// Reward value between -0.7 and 1.02
    pub fn calculate(&self, epoch_n: u32, state: &EpochState) -> f32 {
        let tot_epochs = (epoch_n as f32) + EPSILON;
        let tot_interactions = state.num_deauths
            .max(state.num_associations)
            .max(state.num_handshakes) as f32
            + EPSILON;

        // Positive rewards
        let h = (state.num_handshakes as f32) / tot_interactions;
        let a = 0.2 * ((state.active_for_epochs as f32) / tot_epochs);
        let c = 0.1 * ((state.num_hops as f32) / (NUM_CHANNELS as f32));

        // Negative penalties
        let b = -0.3 * ((state.blind_for_epochs as f32) / tot_epochs);
        let m = -0.3 * ((state.missed_interactions as f32) / tot_interactions);
        let i = -0.2 * ((state.inactive_for_epochs as f32) / tot_epochs);

        // Emotion penalties (only count if >= 5 epochs)
        let sad = if state.sad_for_epochs >= 5 {
            state.sad_for_epochs as f32
        } else {
            0.0
        };
        let bored = if state.bored_for_epochs >= 5 {
            state.bored_for_epochs as f32
        } else {
            0.0
        };
        let s = -0.2 * (sad / tot_epochs);
        let l = -0.1 * (bored / tot_epochs);

        h + a + c + b + i + m + s + l
    }

    /// Get the reward range
    pub fn range(&self) -> (f32, f32) {
        (REWARD_MIN, REWARD_MAX)
    }
}

impl Default for RewardFunction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_positive() {
        let reward_fn = RewardFunction::new();
        let state = EpochState {
            num_handshakes: 10,
            num_deauths: 20,
            num_associations: 15,
            num_hops: 12,
            missed_interactions: 1,
            active_for_epochs: 50,
            inactive_for_epochs: 0,
            blind_for_epochs: 0,
            sad_for_epochs: 0,
            bored_for_epochs: 0,
        };

        let reward = reward_fn.calculate(100, &state);
        assert!(reward > 0.0, "Good performance should yield positive reward");
    }

    #[test]
    fn test_reward_negative() {
        let reward_fn = RewardFunction::new();
        let state = EpochState {
            num_handshakes: 0,
            num_deauths: 5,
            num_associations: 3,
            num_hops: 1,
            missed_interactions: 10,
            active_for_epochs: 0,
            inactive_for_epochs: 80,
            blind_for_epochs: 50,
            sad_for_epochs: 10,
            bored_for_epochs: 8,
        };

        let reward = reward_fn.calculate(100, &state);
        assert!(reward < 0.0, "Poor performance should yield negative reward");
    }

    #[test]
    fn test_reward_within_range() {
        let reward_fn = RewardFunction::new();
        let state = EpochState {
            num_handshakes: 5,
            num_deauths: 10,
            num_associations: 8,
            num_hops: 7,
            missed_interactions: 2,
            active_for_epochs: 30,
            inactive_for_epochs: 20,
            blind_for_epochs: 5,
            sad_for_epochs: 3,
            bored_for_epochs: 2,
        };

        let reward = reward_fn.calculate(100, &state);
        let (min, max) = reward_fn.range();
        assert!(
            reward >= min && reward <= max,
            "Reward {} should be within range [{}, {}]",
            reward,
            min,
            max
        );
    }

    #[test]
    fn test_emotion_threshold() {
        let reward_fn = RewardFunction::new();

        // Sad for 4 epochs - should not count
        let state1 = EpochState {
            num_handshakes: 1,
            num_deauths: 2,
            num_associations: 2,
            num_hops: 3,
            missed_interactions: 0,
            active_for_epochs: 10,
            inactive_for_epochs: 0,
            blind_for_epochs: 0,
            sad_for_epochs: 4,
            bored_for_epochs: 0,
        };

        // Sad for 5 epochs - should count
        let state2 = EpochState {
            sad_for_epochs: 5,
            ..state1.clone()
        };

        let reward1 = reward_fn.calculate(100, &state1);
        let reward2 = reward_fn.calculate(100, &state2);

        assert!(
            reward2 < reward1,
            "5+ sad epochs should reduce reward more than 4 sad epochs"
        );
    }
}
