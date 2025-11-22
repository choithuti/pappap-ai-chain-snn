// src/managers/consensus_engine.rs
use crate::{snn_core::SNNCore, bus::MessageBus};
use std::sync::Arc;

pub fn spawn(snn: Arc<SNNCore>, bus: Arc<MessageBus>) {
    let bus_tx = bus.sender();
    tokio::spawn(async move {
        let mut rx = bus.subscribe();

        while let Ok((target, data)) = rx.recv().await {
            if target == "block_proposal" {
                let proposal: serde_json::Value = serde_json::from_slice(&data).unwrap_or_default();
                let score = proposal["spike_score"].as_f64().unwrap_or(0.0) as f32;

                // SNN vote dựa trên spike score + random neuromodulation
                let vote_input = score + rand::random::<f32>() * 0.1;
                let approval = snn.forward(vote_input).await;

                if approval > 0.68 {
                    tracing::info!("Consensus APPROVED block (SNN vote: {:.3})", approval);
                    let _ = bus_tx.send(("block_finalized".to_string(), data));
                } else {
                    tracing::warn!("Consensus REJECTED (score too low: {:.3})", approval);
                }
            }
        }
    });
}
