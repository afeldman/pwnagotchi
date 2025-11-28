//! # Pwnagotchi AI - Reinforcement Learning System
//!
//! Pure Rust implementation of Advantage Actor-Critic (A2C) reinforcement learning
//! for WiFi capture optimization, inspired by the original Python Pwnagotchi.
//!
//! ## Architecture
//!
//! The system uses an A2C (Advantage Actor-Critic) algorithm with:
//! - **Actor**: Policy network that decides which actions to take
//! - **Critic**: Value network that evaluates how good the current state is
//! - **LSTM**: Recurrent layers for temporal dependencies
//!
//! ## Training Loop
//!
//! 1. **Observation**: Collect WiFi environment state (APs, channels, handshakes)
//! 2. **Action**: Adjust personality parameters (recon_time, channels, etc.)
//! 3. **Reward**: Calculate based on handshakes captured, activity, emotions
//! 4. **Learn**: Update policy and value networks using A2C loss
//!
//! ## Example
//!
//! ```rust,no_run
//! use pwnagotchi_ai::{A2CAgent, Environment, RewardFunction};
//!
//! # tokio_test::block_on(async {
//! let mut agent = A2CAgent::new()?;
//! let mut env = Environment::new();
//!
//! // Training loop
//! for epoch in 0..1000 {
//!     let observation = env.observe().await?;
//!     let action = agent.predict(&observation)?;
//!     let reward = env.step(action).await?;
//!     agent.train(observation, action, reward)?;
//! }
//! # Ok::<(), anyhow::Error>(())
//! # });
//! ```

pub mod a2c;
pub mod environment;
pub mod epoch;
pub mod reward;
pub mod network;
pub mod trainer;

pub use a2c::A2CAgent;
pub use environment::Environment;
pub use epoch::{Epoch, EpochData};
pub use reward::RewardFunction;
