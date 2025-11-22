use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
use rand::RngCore;

pub struct CryptoEngine {
    cipher: Aes256Gcm,
}

impl CryptoEngine {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        Self { cipher: Aes256Gcm::new(key) }
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        let ciphertext = self.cipher.encrypt(Nonce::from_slice(&nonce), data).unwrap();
        [nonce.to_vec(), ciphertext].concat()
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, ()> {
        if data.len() < 12 { return Err(()); }
        let (nonce, ciphertext) = data.split_at(12);
        self.cipher.decrypt(Nonce::from_slice(nonce), ciphertext).map_err(|_| ())
    }
}
