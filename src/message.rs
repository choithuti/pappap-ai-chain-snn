use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use serde_json::Value;
use std::sync::Arc;
use crate::snn_core::SNNCore;

#[derive(Clone)]
pub struct MessageBus {
    tx: UnboundedSender<(String, Vec<u8>)>,
    rx: Arc<tokio::sync::Mutex<UnboundedReceiver<(String, Vec<u8>)>>>,
    snn: Arc<SNNCore>,
}

impl MessageBus {
    pub fn new(snn: Arc<SNNCore>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
            snn,
        }
    }

    pub async fn send(&self, target: String, data: Value) {
        let payload = serde_json::to_vec(&data).unwrap();
        let encrypted = self.snn.encrypt_data(&payload).await;
        let _ = self.tx.send((target, encrypted));
    }

    pub async fn subscribe(&self) -> UnboundedReceiver<(String, Vec<u8>)> {
        let lock = self.rx.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        // Đây chỉ là demo, thực tế dùng broadcast channel nếu cần nhiều subscriber
        // Trong phiên bản này ta không cần subscribe thật, chỉ để compile
        drop(lock);
        rx
    }
}
