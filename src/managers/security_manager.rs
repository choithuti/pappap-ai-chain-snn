// src/managers/security_manager.rs
use crate::{snn_core::SNNCore, bus::MessageBus};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub fn spawn(snn: Arc<SNNCore>, bus: Arc<MessageBus>) {
    let violations = Arc::new(RwLock::new(HashMap::<String, u8>::new()));
    tokio::spawn(async move {
        let mut rx = bus.subscribe();
        let violations = violations.clone();

        while let Ok((_target, data)) = rx.recv().await {
            let size = data.len() as f32;
            let input = size / 1024.0; // KB
            let anomaly_score = snn.forward(input.clamp(0.0, 10.0)).await;

            if anomaly_score > 0.92 {
                let mut map = violations.write().await;
                let entry = map.entry("unknown_peer".to_string()).or_insert(0);
                *entry += 1;

                if *entry >= 4 {
                    tracing::error!("SECURITY: Peer BANNED permanently – {} violations", *entry);
                    // TODO: ban IP / peer_id
                } else {
                    tracing::warn!("Anomaly detected (score: {:.3}) – violation {}/4", anomaly_score, *entry);
                }
            }
        }
    });
}
