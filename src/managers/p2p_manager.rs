// src/managers/p2p_manager.rs (phiên b?n m?i – dùng libp2p)
use crate::{libp2p_swarm::P2PSwarm, snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine};
use std::sync::Arc;

pub fn spawn(snn: Arc<SNNCore>, bus: Arc<MessageBus>, crypto: Arc<CryptoEngine>) {
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        match P2PSwarm::new(bus_clone, crypto, snn).await {
            Ok(swarm) => {
                // K?t n?i internal bus ? swarm
                let sender = swarm.get_sender();
                let mut rx = bus.subscribe();
                tokio::spawn(async move {
                    while let Ok((target, data)) = rx.recv().await {
                        if target == "gossip" || target == "all" {
                            let _ = sender.send((target, data));
                        }
                    }
                });

                swarm.run().await;
            }
            Err(e) => {
                error!("Failed to start P2P swarm: {}", e);
            }
        }
    });

    tracing::info!("P2P Manager ? libp2p Swarm ACTIVE");
}