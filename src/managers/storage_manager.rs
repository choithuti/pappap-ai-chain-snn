// src/managers/storage_manager.rs
use crate::{snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine};
use std::sync::Arc;
use chrono::Utc;

pub fn spawn(snn: Arc<SNNCore>, bus: Arc<MessageBus>, crypto: Arc<CryptoEngine>) {
    tokio::spawn(async move {
        let mut rx = bus.subscribe();
        std::fs::create_dir_all("chaindata/blocks").unwrap_or(());

        while let Ok((target, data)) = rx.recv().await {
            if target == "block_finalized" {
                let encrypted = crypto.encrypt(&data);
                let filename = format!("chaindata/blocks/block_{}.blk", Utc::now().timestamp());
                if std::fs::write(&filename, encrypted).is_ok() {
                    tracing::info!("Block permanently stored: {}", filename);
                    let _ = snn.forward(2.0).await; // Reward spike!
                }
            }
        }
    });
}
