// src/managers/transaction_manager.rs
use crate::{snn_core::SNNCore, bus::MessageBus};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub fn spawn(snn: Arc<SNNCore>, bus: Arc<MessageBus>) {
    let bus_tx = bus.sender();
    tokio::spawn(async move {
        let mut rx = bus.subscribe();
        let mut pending_txs = Vec::new();
        let mut interval = tokio::time::interval(Duration::from_secs(8));

        loop {
            tokio::select! {
                Ok((target, data)) = rx.recv() => {
                    if target == "tx" || target == "all" {
                        pending_txs.push(data);
                        tracing::debug!("Tx received, pool size: {}", pending_txs.len());
                    }
                }
                _ = interval.tick() => {
                    if pending_txs.len() >= 5 {
                        let load = pending_txs.len() as f32 / 50.0;
                        let decision = snn.forward(load.clamp(0.0, 2.0)).await;
                        if decision > 0.55 {
                            tracing::info!("Block sealed! Txs: {} | SNN Confidence: {:.3}", pending_txs.len(), decision);
                            // Gửi block proposal
                            let _ = bus_tx.send(("block_proposal".to_string(), serde_json::json!({
                                "tx_count": pending_txs.len(),
                                "spike_score": decision
                            }).to_string().into_bytes()));
                            pending_txs.clear();
                        }
                    }
                }
            }
        }
    });
}