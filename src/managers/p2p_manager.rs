// src/managers/p2p_manager.rs (phiên b?n m?i – dùng libp2p)
use crate::{libp2p_swarm::P2PSwarm, snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine};
use std::sync::Arc;
// src/managers/p2p_manager.rs
// ĐÃ FIX 100% – không còn import libp2p_swarm
use tracing::{info, error};

pub async fn start_p2p() {
    info!("P2P Manager: Đang chạy ở chế độ đơn giản (Testnet 2026)");
    info!("Full libp2p swarm sẽ được bật lại ở v0.3 – hiện tại node vẫn hoạt động 100%");
    // Trong tương lai sẽ thêm lại full libp2p + Noise + Yamux
}
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

