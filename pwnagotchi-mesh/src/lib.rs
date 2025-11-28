use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Mesh peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPeer {
    pub fingerprint: String,
    pub name: String,
    pub identity: String,
    pub public_key: Vec<u8>,
    pub pwnd_run: u32,
    pub pwnd_tot: u32,
    pub uptime: u64,
    pub version: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Mesh advertisement message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAdvertisement {
    pub fingerprint: String,
    pub name: String,
    pub identity: String,
    pub pwnd_run: u32,
    pub pwnd_tot: u32,
    pub uptime: u64,
    pub version: String,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

/// Identity/keypair for mesh networking
pub struct MeshIdentity {
    keypair: Keypair,
    fingerprint: String,
    name: String,
}

impl MeshIdentity {
    /// Generate new identity
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

    /// Load identity from keys
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

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature: Signature = self.keypair.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Verify a signature
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
