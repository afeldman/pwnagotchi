//! Cryptographic mesh networking for Pwnagotchi peer-to-peer communication.
//!
//! This crate implements Ed25519-based mesh networking that allows multiple
//! Pwnagotchi units to discover and communicate with each other securely.
//!
//! # Features
//!
//! - **Ed25519 cryptography**: Digital signatures for peer authentication
//! - **Peer discovery**: Advertisement and detection of nearby units
//! - **Identity management**: Generate and load cryptographic identities
//! - **Signature verification**: Ensure message authenticity
//!
//! # Examples
//!
//! ## Creating an Identity
//!
//! ```
//! use pwnagotchi_mesh::MeshIdentity;
//!
//! // Generate new identity
//! let identity = MeshIdentity::generate("MyPwnagotchi");
//! 
//! println!("Fingerprint: {}", identity.fingerprint());
//! println!("Name: {}", identity.name());
//! ```
//!
//! ## Signing and Verifying Messages
//!
//! ```
//! use pwnagotchi_mesh::MeshIdentity;
//!
//! let identity = MeshIdentity::generate("Unit1");
//!
//! // Sign a message
//! let message = b"Hello, mesh!";
//! let signature = identity.sign(message);
//!
//! // Verify signature
//! let is_valid = MeshIdentity::verify(
//!     message,
//!     &signature,
//!     identity.public_key()
//! );
//!
//! assert!(is_valid);
//! ```
//!
//! ## Creating Advertisements
//!
//! ```
//! use pwnagotchi_mesh::{MeshIdentity, MeshAdvertiser};
//!
//! let identity = MeshIdentity::generate("MyUnit");
//! let advertiser = MeshAdvertiser::new(identity, "1.0.0".to_string());
//!
//! // Create advertisement with current stats
//! let ad = advertiser.create_advertisement(10, 100, 3600);
//! 
//! println!("Advertisement from {}", ad.name);
//! ```

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Information about a mesh network peer.
///
/// Contains identity, statistics, and connectivity information about
/// a discovered Pwnagotchi unit.
///
/// # Examples
///
/// ```
/// # use pwnagotchi_mesh::MeshPeer;
/// let peer = MeshPeer {
///     fingerprint: "0123456789abcdef".to_string(),
///     name: "OtherUnit".to_string(),
///     identity: "other_unit".to_string(),
///     public_key: vec![0u8; 32],
///     pwnd_run: 5,
///     pwnd_tot: 50,
///     uptime: 7200,
///     version: "1.0.0".to_string(),
///     last_seen: chrono::Utc::now(),
/// };
///
/// println!("Peer {} has captured {} handshakes", peer.name, peer.pwnd_run);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPeer {
    /// Short hex fingerprint derived from public key
    pub fingerprint: String,
    /// Human-readable name
    pub name: String,
    /// Unique identity string
    pub identity: String,
    /// Ed25519 public key bytes
    pub public_key: Vec<u8>,
    /// Handshakes captured this run
    pub pwnd_run: u32,
    /// Total handshakes captured
    pub pwnd_tot: u32,
    /// Uptime in seconds
    pub uptime: u64,
    /// Software version
    pub version: String,
    /// Last time this peer was seen
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Advertisement message broadcast by mesh peers.
///
/// Contains peer information and a cryptographic signature for authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAdvertisement {
    /// Peer fingerprint
    pub fingerprint: String,
    /// Peer name
    pub name: String,
    /// Peer identity
    pub identity: String,
    /// Handshakes this run
    pub pwnd_run: u32,
    /// Total handshakes
    pub pwnd_tot: u32,
    /// Uptime in seconds
    pub uptime: u64,
    /// Software version
    pub version: String,
    /// Unix timestamp
    pub timestamp: i64,
    /// Ed25519 signature
    pub signature: Vec<u8>,
}

/// Cryptographic identity for mesh networking.
///
/// Contains an Ed25519 keypair used for signing advertisements
/// and verifying peer authenticity.
///
/// # Examples
///
/// ## Generating a New Identity
///
/// ```
/// use pwnagotchi_mesh::MeshIdentity;
///
/// let identity = MeshIdentity::generate("MyPwnagotchi");
/// println!("Fingerprint: {}", identity.fingerprint());
/// ```
///
/// ## Loading an Existing Identity
///
/// ```no_run
/// use pwnagotchi_mesh::MeshIdentity;
///
/// let secret = [0u8; 32]; // Load from file
/// let public = [0u8; 32]; // Load from file
///
/// let identity = MeshIdentity::from_keys(&secret, &public, "MyPwnagotchi")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MeshIdentity {
    keypair: Keypair,
    fingerprint: String,
    name: String,
}

impl MeshIdentity {
    /// Generates a new random identity.
    ///
    /// Creates a new Ed25519 keypair using a cryptographically secure random number generator.
    /// The fingerprint is derived from the first 8 bytes of the public key.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for this unit
    ///
    /// # Examples
    ///
    /// ```
    /// use pwnagotchi_mesh::MeshIdentity;
    ///
    /// let identity = MeshIdentity::generate("MyPwnagotchi");
    /// assert_eq!(identity.name(), "MyPwnagotchi");
    /// assert_eq!(identity.fingerprint().len(), 16); // 8 bytes as hex
    /// ```
    pub fn generate(name: &str) -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let fingerprint = hex::encode(&keypair.public.as_bytes()[..8]);

        Self {
            keypair,
            fingerprint,
            name: name.to_string(),
        }
    }

    /// Loads an identity from existing key bytes.
    ///
    /// # Arguments
    ///
    /// * `secret` - 32-byte Ed25519 secret key
    /// * `public` - 32-byte Ed25519 public key
    /// * `name` - Human-readable name
    ///
    /// # Returns
    ///
    /// Returns the identity or an error if the keys are invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Secret key is not 32 bytes or invalid
    /// - Public key is not 32 bytes or invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pwnagotchi_mesh::MeshIdentity;
    ///
    /// // Load keys from file
    /// let secret = std::fs::read("secret.key")?;
    /// let public = std::fs::read("public.key")?;
    ///
    /// let identity = MeshIdentity::from_keys(&secret, &public, "SavedUnit")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_keys(secret: &[u8], public: &[u8], name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let secret_key = SecretKey::from_bytes(secret)?;
        let public_key = PublicKey::from_bytes(public)?;
        let keypair = Keypair {
            secret: secret_key,
            public: public_key,
        };
        let fingerprint = hex::encode(&public[..8]);

        Ok(Self {
            keypair,
            fingerprint,
            name: name.to_string(),
        })
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn public_key(&self) -> &[u8] {
        self.keypair.public.as_bytes()
    }

    /// Signs a message using this identity's private key.
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes to sign
    ///
    /// # Returns
    ///
    /// Returns the Ed25519 signature as a 64-byte vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use pwnagotchi_mesh::MeshIdentity;
    ///
    /// let identity = MeshIdentity::generate("Signer");
    /// let message = b"Important data";
    /// let signature = identity.sign(message);
    ///
    /// assert_eq!(signature.len(), 64);
    /// ```
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature: Signature = self.keypair.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Verifies a signature against a message and public key.
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes that were signed
    /// * `signature` - The 64-byte Ed25519 signature
    /// * `public_key` - The 32-byte Ed25519 public key
    ///
    /// # Returns
    ///
    /// Returns `true` if the signature is valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use pwnagotchi_mesh::MeshIdentity;
    ///
    /// let identity = MeshIdentity::generate("Verifier");
    /// let message = b"Test message";
    /// 
    /// let signature = identity.sign(message);
    ///
    /// // Verify with correct key
    /// assert!(identity.verify(message, &signature, identity.public_key()));
    ///
    /// // Verify with wrong message
    /// assert!(!identity.verify(b"Wrong message", &signature, identity.public_key()));
    /// ```
    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        if let Ok(pk) = PublicKey::from_bytes(public_key) {
            if let Ok(sig) = Signature::from_bytes(signature) {
                return pk.verify(message, &sig).is_ok();
            }
        }
        false
    }
}

/// Mesh advertiser - broadcasts presence to other units
pub struct MeshAdvertiser {
    identity: MeshIdentity,
    peers: HashMap<String, MeshPeer>,
}

impl MeshAdvertiser {
    pub fn new(identity: MeshIdentity) -> Self {
        Self {
            identity,
            peers: HashMap::new(),
        }
    }

    /// Create advertisement message
    pub fn create_advertisement(
        &self,
        pwnd_run: u32,
        pwnd_tot: u32,
        uptime: u64,
        version: &str,
    ) -> MeshAdvertisement {
        let timestamp = chrono::Utc::now().timestamp();
        let message = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.identity.fingerprint(),
            self.identity.name(),
            pwnd_run,
            pwnd_tot,
            uptime,
            version,
            timestamp
        );

        let signature = self.identity.sign(message.as_bytes());

        MeshAdvertisement {
            fingerprint: self.identity.fingerprint().to_string(),
            name: self.identity.name().to_string(),
            identity: hex::encode(self.identity.public_key()),
            pwnd_run,
            pwnd_tot,
            uptime,
            version: version.to_string(),
            timestamp,
            signature,
        }
    }

    /// Process received advertisement
    pub fn process_advertisement(&mut self, adv: MeshAdvertisement) -> bool {
        // Verify signature
        let message = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            adv.fingerprint, adv.name, adv.pwnd_run, adv.pwnd_tot, adv.uptime, adv.version, adv.timestamp
        );

        let public_key = match hex::decode(&adv.identity) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        if !self.identity.verify(message.as_bytes(), &adv.signature, &public_key) {
            debug!("Invalid signature from peer {}", adv.fingerprint);
            return false;
        }

        info!("Discovered peer: {} ({})", adv.name, adv.fingerprint);

        // Add or update peer
        let peer = MeshPeer {
            fingerprint: adv.fingerprint.clone(),
            name: adv.name,
            identity: adv.identity,
            public_key,
            pwnd_run: adv.pwnd_run,
            pwnd_tot: adv.pwnd_tot,
            uptime: adv.uptime,
            version: adv.version,
            last_seen: chrono::Utc::now(),
        };

        self.peers.insert(adv.fingerprint, peer);
        true
    }

    /// Get all known peers
    pub fn peers(&self) -> &HashMap<String, MeshPeer> {
        &self.peers
    }

    /// Remove stale peers
    pub fn cleanup_peers(&mut self, max_age_secs: i64) {
        let now = chrono::Utc::now();
        self.peers.retain(|_, peer| {
            (now - peer.last_seen).num_seconds() < max_age_secs
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = MeshIdentity::generate("test");
        assert_eq!(identity.name(), "test");
        assert!(!identity.fingerprint().is_empty());
    }

    #[test]
    fn test_signature() {
        let identity = MeshIdentity::generate("test");
        let message = b"Hello, mesh!";
        let signature = identity.sign(message);
        
        assert!(identity.verify(message, &signature, identity.public_key()));
    }

    #[test]
    fn test_advertisement() {
        let identity = MeshIdentity::generate("test");
        let advertiser = MeshAdvertiser::new(identity);
        
        let adv = advertiser.create_advertisement(5, 100, 3600, "2.8.9");
        
        assert_eq!(adv.pwnd_run, 5);
        assert_eq!(adv.pwnd_tot, 100);
        assert_eq!(adv.version, "2.8.9");
    }
}
