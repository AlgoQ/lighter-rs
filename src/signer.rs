//! Cryptographic signing and key management for Lighter Protocol

use crate::constants::{PRIVATE_KEY_LENGTH, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::errors::{LighterError, Result};
use crate::utils::hex_to_bytes;

/// Trait for signing messages
pub trait Signer {
    fn sign(&self, hashed_message: &[u8]) -> Result<Vec<u8>>;
}

/// Trait for key management operations
pub trait KeyManager: Signer {
    fn pub_key(&self) -> &[u8];
    fn pub_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH];
    fn prv_key_bytes(&self) -> Vec<u8>;
}

/// Implementation of key manager using Poseidon cryptography
pub struct PoseidonKeyManager {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl PoseidonKeyManager {
    pub fn new(private_key_bytes: &[u8]) -> Result<Self> {
        // Accept both 32-byte (256-bit) and 40-byte keys
        // 32 bytes is standard for many cryptographic keys
        // 40 bytes is the Lighter protocol specification
        if private_key_bytes.len() != 32 && private_key_bytes.len() != PRIVATE_KEY_LENGTH {
            return Err(LighterError::InvalidPrivateKeyLength {
                expected: PRIVATE_KEY_LENGTH,
                actual: private_key_bytes.len(),
            });
        }

        let public_key = Self::derive_public_key(private_key_bytes)?;

        Ok(Self {
            private_key: private_key_bytes.to_vec(),
            public_key,
        })
    }

    pub fn from_hex(hex_private_key: &str) -> Result<Self> {
        let bytes = hex_to_bytes(hex_private_key)?;
        Self::new(&bytes)
    }

    fn derive_public_key(_private_key: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement actual Schnorr public key derivation with Poseidon crypto
        Ok(vec![0u8; PUBLIC_KEY_LENGTH])
    }
}

impl Signer for PoseidonKeyManager {
    fn sign(&self, hashed_message: &[u8]) -> Result<Vec<u8>> {
        if hashed_message.len() != 40 {
            return Err(LighterError::CryptoError(format!(
                "Invalid hashed message length: expected 40, got {}",
                hashed_message.len()
            )));
        }
        // TODO: Implement actual Schnorr signing with Poseidon crypto
        Ok(vec![0u8; SIGNATURE_LENGTH])
    }
}

impl KeyManager for PoseidonKeyManager {
    fn pub_key(&self) -> &[u8] {
        &self.public_key
    }

    fn pub_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        let mut result = [0u8; PUBLIC_KEY_LENGTH];
        result.copy_from_slice(&self.public_key[..PUBLIC_KEY_LENGTH]);
        result
    }

    fn prv_key_bytes(&self) -> Vec<u8> {
        self.private_key.clone()
    }
}

pub fn new_key_manager(hex_key: &str) -> Result<Box<dyn KeyManager>> {
    Ok(Box::new(PoseidonKeyManager::from_hex(hex_key)?))
}
