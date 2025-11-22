use config::{Config, File};
use zeroize::Zeroize;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub listen_addr: String,
    pub p2p_port: u16,
    pub encryption_key: [u8; 32],
}

impl AppConfig {
    pub fn load() -> Self {
        let s = Config::builder()
            .add_source(File::with_name("Config").required(false))
            .add_source(config::Environment::with_prefix("PAPPAP"))
            .build()
            .unwrap();

        let mut key = s.get::<String>("encryption_key")
            .unwrap_or_else(|_| "pappap2025snnblockchainkey32b!".to_string());

        let mut key_bytes = [0u8; 32];
        let bytes = key.as_bytes();
        key_bytes[..bytes.len().min(32)].copy_from_slice(&bytes[..bytes.len().min(32)]);
        key.zeroize();

        Self {
            listen_addr: s.get::<String>("listen_addr").unwrap_or("0.0.0.0:8080".into()),
            p2p_port: s.get::<u16>("p2p_port").unwrap_or(9000),
            encryption_key: key_bytes,
        }
    }
}
