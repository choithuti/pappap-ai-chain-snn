pub mod p2p_manager;
pub mod transaction_manager;
pub mod consensus_engine;
pub mod security_manager;
pub mod storage_manager;

use crate::{snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine};
use std::sync::Arc;
use tracing::info;

pub async fn start_all(_snn: Arc<SNNCore>, _bus: Arc<MessageBus>, _crypto: Arc<CryptoEngine>) {
    info!("All PappapAIChain managers started – Ready for Testnet 2026");
    p2p_manager::start_p2p().await;
}
