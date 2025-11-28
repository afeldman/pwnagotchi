//! Neural network architecture for A2C
//!
//! Actor-Critic networks with LSTM layers for temporal dependencies

use burn::prelude::*;

// TODO: Implement A2C network architecture using burn framework
// This will include:
// - LSTM layers for sequence processing
// - Actor network (policy)
// - Critic network (value function)
// - Forward pass implementation

/// Neural network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Input dimension (observation space)
    pub input_dim: usize,
    /// Hidden layer dimensions
    pub hidden_dims: Vec<usize>,
    /// Output dimension (action space)
    pub output_dim: usize,
    /// LSTM hidden size
    pub lstm_hidden_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            input_dim: 42,      // 3 histograms * 14 channels
            hidden_dims: vec![128, 64],
            output_dim: 20,     // Personality parameters
            lstm_hidden_size: 256,
        }
    }
}
