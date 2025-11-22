use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::{Aead, AeadCore}};
use rand::RngCore;

pub struct CryptoEngine {
    cipher: Aes256Gcm,
}

impl CryptoEngine {
    pub fn new(key: &[u8; 32]) -> Self {
        Self { cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key)) }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), data).expect("encrypt fail");
        [nonce.as_slice(), &ciphertext].concat()
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
        if data.len() < 12 { return Err(aes_gcm::Error); }
        let (nonce, ciphertext) = data.split_at(12);
        self.cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
    }
}