// src/crypto_pqc.rs – Post-Quantum Crypto
use pqcrypto_kyber::kyber768::*;
use pqcrypto_dilithium::dilithium3::*;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit, AeadCore, aead::Aead};

pub struct PQCrypto {
    kyber_sk: SecretKey,
}

impl PQCrypto {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Self { kyber_sk: sk }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let (ct, ss) = encapsulate(&public_from_secret(&self.kyber_sk));
        let key = Key::from_slice(&ss);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());
        let ciphertext = cipher.encrypt(&nonce, data).unwrap();
        [ct.as_bytes(), nonce.as_slice(), &ciphertext].concat()
    }
}