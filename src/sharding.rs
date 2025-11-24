// src/sharding.rs – Tự động chia shard theo kích thước mạng
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ShardingEngine {
    current_shard_count: Arc<RwLock<usize>>,
}

impl ShardingEngine {
    pub fn new() -> Self {
        Self { current_shard_count: Arc::new(RwLock::new(1)) }
    }

    pub async fn get_shard_id(&self, peer_id: u64, network_size: usize) -> usize {
        let shard_count = (network_size as f64).log10().ceil() as usize;
        let mut guard = self.current_shard_count.write().await;
        if *guard != shard_count {
            *guard = shard_count;
            println!("Auto-sharding: Network size {network_size} → {shard_count} shards");
        }
        (peer_id % shard_count as u64) as usize
    }
}