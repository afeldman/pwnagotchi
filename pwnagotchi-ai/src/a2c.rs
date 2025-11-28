//! A2C (Advantage Actor-Critic) and PPO agent implementation
//!
//! Supports both algorithms with ONNX Runtime inference

use crate::environment::{Action, Observation};
use crate::network::{ActorCriticNetwork, Algorithm, NetworkConfig};
use anyhow::{Context, Result};
use ndarray::{Array1, ArrayView1};
use rand::distributions::{Distribution, WeightedIndex};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Neural network configuration
    pub network: NetworkConfig,
    /// Model file path
    pub model_path: PathBuf,
    /// Temperature for action sampling (higher = more exploration)
    pub temperature: f32,
    /// Clip ratio for PPO (only used if algorithm is PPO)
    pub ppo_clip_ratio: f32,
    /// Discount factor (gamma)
    pub gamma: f32,
    /// GAE lambda for advantage estimation
    pub gae_lambda: f32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            model_path: PathBuf::from("/root/.pwnagotchi-ai/model.onnx"),
            temperature: 1.0,
            ppo_clip_ratio: 0.2,
            gamma: 0.99,
            gae_lambda: 0.95,
        }
    }
}

/// RL Agent supporting both A2C and PPO algorithms
///
/// Uses ONNX Runtime for inference with models trained in Python.
///
/// ## Algorithms
///
/// - **A2C**: Faster, simpler, on-policy
/// - **PPO**: More stable, better for continuous learning
///
/// ## Example
///
/// ```no_run
/// use pwnagotchi_ai::a2c::{RLAgent, AgentConfig};
/// use pwnagotchi_ai::network::Algorithm;
///
/// let mut config = AgentConfig::default();
/// config.network.algorithm = Algorithm::PPO;
///
/// let mut agent = RLAgent::new(config)?;
/// let observation = /* ... */;
/// let action = agent.predict(&observation)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct RLAgent {
    network: ActorCriticNetwork,
    config: AgentConfig,
}

impl RLAgent {
    /// Create new agent with configuration
    pub fn new(config: AgentConfig) -> Result<Self> {
        let network = ActorCriticNetwork::new(&config.model_path, config.network.clone())
            .context("Failed to load neural network")?;

        Ok(Self { network, config })
    }

    /// Create agent with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(AgentConfig::default())
    }

    /// Load agent from model file
    pub fn load(model_path: impl AsRef<Path>) -> Result<Self> {
        let mut config = AgentConfig::default();
        config.model_path = model_path.as_ref().to_path_buf();
        Self::new(config)
    }

    /// Predict action from observation
    ///
    /// Uses the actor network to output action logits, then samples
    /// from the categorical distribution.
    pub fn predict(&mut self, observation: &Observation) -> Result<Action> {
        // Convert observation to input array
        let input = self.observation_to_array(observation);

        // Forward pass through network
        let (action_logits, _value) = self.network.forward(input.view())?;

        // Apply temperature scaling for exploration
        let scaled_logits = if self.config.temperature != 1.0 {
            action_logits.mapv(|x| x / self.config.temperature)
        } else {
            action_logits
        };

        // Convert logits to action
        let action = self.logits_to_action(&scaled_logits)?;

        Ok(action)
    }

    /// Evaluate an observation (get value estimate)
    pub fn evaluate(&mut self, observation: &Observation) -> Result<f32> {
        let input = self.observation_to_array(observation);
        let (_action_logits, value) = self.network.forward(input.view())?;
        Ok(value)
    }

    /// Reset LSTM hidden state (call at episode boundaries)
    pub fn reset(&mut self) {
        self.network.reset_hidden_state();
    }

    /// Get current algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.network.algorithm()
    }

    /// Convert observation to network input array
    fn observation_to_array(&self, obs: &Observation) -> Array1<f32> {
        let mut input = Vec::with_capacity(self.config.network.input_dim);
        
        // Concatenate histograms: [aps, sta, peers]
        input.extend_from_slice(&obs.aps_histogram);
        input.extend_from_slice(&obs.sta_histogram);
        input.extend_from_slice(&obs.peers_histogram);
        
        Array1::from_vec(input)
    }

    /// Convert network logits to action
    ///
    /// The output is 20 logits corresponding to personality parameters:
    /// - recon_time: [0-9] → 10-100 seconds
    /// - min_rssi: [10-13] → -200 to -50 dBm
    /// - channels: [14-16] → channel selection strategy
    /// - deauth: [17] → boolean
    /// - associate: [18] → boolean  
    /// - bored_epochs: [19] → 5-30
    /// - sad_epochs: [20] → 10-40
    fn logits_to_action(&self, logits: &Array1<f32>) -> Result<Action> {
        // Simple mapping (in production, use proper sampling)
        // This is a placeholder - actual implementation would use
        // categorical distributions for each parameter
        
        let recon_time = (logits[0].max(0.0).min(9.0) * 10.0 + 10.0) as u32;
        let min_rssi = (logits[10] * 15.0 - 200.0) as i32;
        
        // Channel selection (simple: use top 3 channels)
        let mut channels = vec![];
        let channel_probs = &logits.slice(ndarray::s![14..17]);
        if channel_probs[0] > 0.0 { channels.push(1); }
        if channel_probs[1] > 0.0 { channels.push(6); }
        if channel_probs[2] > 0.0 { channels.push(11); }
        if channels.is_empty() { channels = vec![1, 6, 11]; }
        
        let deauth = logits[17] > 0.0;
        let associate = logits[18] > 0.0;
        let bored_num_epochs = (logits[19].max(0.0) * 2.5 + 5.0) as u32;
        let sad_num_epochs = (logits.get(19).unwrap_or(&0.0).max(0.0) * 3.0 + 10.0) as u32;

        Ok(Action {
            recon_time,
            min_rssi,
            channels,
            deauth,
            associate,
            bored_num_epochs,
            sad_num_epochs,
        })
    }

    /// Calculate Generalized Advantage Estimation (GAE)
    ///
    /// Used for both A2C and PPO training.
    pub fn calculate_advantages(
        &self,
        rewards: &[f32],
        values: &[f32],
        next_value: f32,
    ) -> Vec<f32> {
        let mut advantages = Vec::with_capacity(rewards.len());
        let mut gae = 0.0;

        // Calculate advantages in reverse order
        for t in (0..rewards.len()).rev() {
            let next_val = if t == rewards.len() - 1 {
                next_value
            } else {
                values[t + 1]
            };

            let delta = rewards[t] + self.config.gamma * next_val - values[t];
            gae = delta + self.config.gamma * self.config.gae_lambda * gae;
            advantages.push(gae);
        }

        advantages.reverse();
        advantages
    }
}

// Keep A2CAgent as alias for backwards compatibility
pub type A2CAgent = RLAgent;

impl Default for RLAgent {
    fn default() -> Self {
        Self::with_defaults().expect("Failed to create agent")
    }
}
