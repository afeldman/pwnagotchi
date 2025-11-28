//! Async training loop for A2C agent

use crate::a2c::A2CAgent;
use crate::environment::Environment;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Number of epochs per training episode
    pub epochs_per_episode: u32,
    /// Model save path
    pub model_path: PathBuf,
    /// Training interval (seconds)
    pub training_interval: u64,
    /// Laziness factor (0.0 = always train, 1.0 = never train)
    pub laziness: f32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs_per_episode: 10,
            model_path: PathBuf::from("/root/.pwnagotchi-ai/model.bin"),
            training_interval: 300, // 5 minutes
            laziness: 0.3,
        }
    }
}

/// Async trainer for background learning
pub struct AsyncTrainer {
    agent: Arc<RwLock<A2CAgent>>,
    environment: Arc<RwLock<Environment>>,
    config: TrainingConfig,
    is_training: Arc<RwLock<bool>>,
}

impl AsyncTrainer {
    /// Create new async trainer
    pub fn new(config: TrainingConfig) -> Result<Self> {
        let agent = A2CAgent::new()?;
        let environment = Environment::new();

        Ok(Self {
            agent: Arc::new(RwLock::new(agent)),
            environment: Arc::new(RwLock::new(environment)),
            config,
            is_training: Arc::new(RwLock::new(false)),
        })
    }

    /// Start background training loop
    pub async fn start(&self) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(self.config.training_interval));

        loop {
            ticker.tick().await;

            // Check if should train (based on laziness)
            if rand::random::<f32>() < self.config.laziness {
                tracing::debug!("Skipping training due to laziness");
                continue;
            }

            // Set training flag
            *self.is_training.write().await = true;

            // Run training episode
            match self.train_episode().await {
                Ok(loss) => {
                    tracing::info!("Training episode completed with loss: {}", loss);
                    self.save_model().await?;
                }
                Err(e) => {
                    tracing::error!("Training error: {}", e);
                }
            }

            // Clear training flag
            *self.is_training.write().await = false;
        }
    }

    /// Train for one episode
    async fn train_episode(&self) -> Result<f32> {
        // TODO: Implement actual training loop
        // 1. Collect experience for N epochs
        // 2. Train agent on collected experience
        // 3. Return average loss
        Ok(0.0)
    }

    /// Save model to disk
    async fn save_model(&self) -> Result<()> {
        let agent = self.agent.read().await;
        agent.save(self.config.model_path.to_str().unwrap())?;
        Ok(())
    }

    /// Check if currently training
    pub async fn is_training(&self) -> bool {
        *self.is_training.read().await
    }

    /// Get reference to agent
    pub fn agent(&self) -> Arc<RwLock<A2CAgent>> {
        Arc::clone(&self.agent)
    }
}
