//! Neural network architecture for RL agents
//!
//! Supports both A2C and PPO algorithms with ONNX Runtime inference

use anyhow::{Context, Result};
use ndarray::{Array1, Array2, ArrayView1};
use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    /// Advantage Actor-Critic (faster, on-policy)
    A2C,
    /// Proximal Policy Optimization (more stable, on-policy)
    PPO,
}

/// Neural network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Algorithm to use
    pub algorithm: Algorithm,
    /// Input dimension (observation space)
    pub input_dim: usize,
    /// Hidden layer dimensions
    pub hidden_dims: Vec<usize>,
    /// Output dimension (action space)
    pub output_dim: usize,
    /// LSTM hidden size
    pub lstm_hidden_size: usize,
    /// LSTM number of layers
    pub lstm_num_layers: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::PPO,  // PPO is more stable
            input_dim: 42,              // 3 histograms * 14 channels
            hidden_dims: vec![128, 64],
            output_dim: 20,             // Personality parameters
            lstm_hidden_size: 256,
            lstm_num_layers: 2,
        }
    }
}

/// Actor-Critic network using ONNX Runtime
///
/// This network runs inference using ONNX models for production deployment.
/// Models can be trained in Python (PyTorch/TensorFlow) and exported to ONNX.
///
/// ## Architecture
///
/// ```text
/// Input (42) → LSTM (256x2) → MLP (128→64) → ┬─→ Actor (20 actions)
///                                              └─→ Critic (1 value)
/// ```
///
/// ## ONNX Model Format
///
/// Expected inputs:
/// - `observation`: [batch_size, seq_len, input_dim]
/// - `lstm_h`: [num_layers, batch_size, hidden_size] (optional, for stateful)
/// - `lstm_c`: [num_layers, batch_size, hidden_size] (optional, for stateful)
///
/// Expected outputs:
/// - `action_logits`: [batch_size, output_dim]
/// - `value`: [batch_size, 1]
/// - `lstm_h_new`: [num_layers, batch_size, hidden_size] (optional)
/// - `lstm_c_new`: [num_layers, batch_size, hidden_size] (optional)
pub struct ActorCriticNetwork {
    session: Arc<Session>,
    config: NetworkConfig,
    // LSTM hidden states (for stateful inference)
    lstm_h: Option<Array2<f32>>,
    lstm_c: Option<Array2<f32>>,
}

impl ActorCriticNetwork {
    /// Create network from ONNX model file
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to ONNX model file
    /// * `config` - Network configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pwnagotchi_ai::network::{ActorCriticNetwork, NetworkConfig};
    ///
    /// let config = NetworkConfig::default();
    /// let network = ActorCriticNetwork::new(
    ///     "/root/.pwnagotchi-ai/model.onnx",
    ///     config
    /// )?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(model_path: impl AsRef<Path>, config: NetworkConfig) -> Result<Self> {
        let environment = Environment::builder()
            .with_name("pwnagotchi-ai")
            .build()?
            .into_arc();

        let session = SessionBuilder::new(&environment)?
            .with_execution_providers([ExecutionProvider::CPU])?
            .with_model_from_file(model_path)
            .context("Failed to load ONNX model")?;

        Ok(Self {
            session: Arc::new(session),
            config,
            lstm_h: None,
            lstm_c: None,
        })
    }

    /// Forward pass through the network
    ///
    /// Returns (action_logits, value_estimate)
    pub fn forward(&mut self, observation: ArrayView1<f32>) -> Result<(Array1<f32>, f32)> {
        // Reshape observation to [1, 1, input_dim] for batch and sequence
        let obs_shape = (1, 1, self.config.input_dim);
        let obs_data: Vec<f32> = observation.to_vec();
        let obs_array = Array::from_shape_vec(obs_shape, obs_data)?;

        // Prepare inputs
        let mut inputs = vec![
            Value::from_array(self.session.allocator(), &obs_array.view())?
        ];

        // Add LSTM states if available (for stateful inference)
        if let (Some(h), Some(c)) = (&self.lstm_h, &self.lstm_c) {
            inputs.push(Value::from_array(self.session.allocator(), &h.view())?);
            inputs.push(Value::from_array(self.session.allocator(), &c.view())?);
        }

        // Run inference
        let outputs = self.session.run(inputs)?;

        // Extract action logits and value
        let action_logits: Array2<f32> = outputs[0].try_extract()?;
        let action_logits = action_logits.row(0).to_owned();

        let value: Array2<f32> = outputs[1].try_extract()?;
        let value = value[[0, 0]];

        // Update LSTM states if model outputs them
        if outputs.len() > 2 {
            self.lstm_h = Some(outputs[2].try_extract()?);
            self.lstm_c = Some(outputs[3].try_extract()?);
        }

        Ok((action_logits, value))
    }

    /// Reset LSTM hidden states
    pub fn reset_hidden_state(&mut self) {
        self.lstm_h = None;
        self.lstm_c = None;
    }

    /// Get algorithm type
    pub fn algorithm(&self) -> Algorithm {
        self.config.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.algorithm, Algorithm::PPO);
        assert_eq!(config.input_dim, 42);
        assert_eq!(config.output_dim, 20);
    }

    #[test]
    fn test_algorithm_serialization() {
        let algo = Algorithm::A2C;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, r#""A2C"#);

        let algo2: Algorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(algo, algo2);
    }
}
