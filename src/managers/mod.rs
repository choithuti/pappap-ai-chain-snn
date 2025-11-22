// src/managers/mod.rs
pub mod transaction_manager;
pub mod consensus_engine;
pub mod p2p_manager;
pub mod security_manager;
pub mod storage_manager;

use crate::{snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine};
use std::sync::Arc;

pub fn start_all(snn: Arc<SNNCore>, bus: Arc<MessageBus>, crypto: Arc<CryptoEngine>) {
    transaction_manager::spawn(snn.clone(), bus.clone());
    consensus_engine::spawn(snn.clone(), bus.clone());
    p2p_manager::spawn(snn.clone(), bus.clone(), crypto.clone());
    security_manager::spawn(snn.clone(), bus.clone());
    storage_manager::spawn(snn.clone(), bus.clone(), crypto);
    tracing::info!("All PappapAIChain managers started – Ready for Testnet 2026");
}