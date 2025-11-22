// src/managers/transaction_manager.rs – ĐÃ FIX 100%
use tracing::info;
use tokio::time::{self, Duration};  // ĐÃ THÊM DÒNG NÀY

pub async fn start_transaction_manager() {
    info!("Transaction Manager: Đã khởi động – sẵn sàng xử lý giao dịch Testnet 2026");

    let mut interval = time::interval(Duration::from_secs(8));
    loop {
        interval.tick().await;
        info!("Transaction heartbeat – 8s/block");
        // TODO: Thêm xử lý giao dịch thật ở v0.4
    }
}
