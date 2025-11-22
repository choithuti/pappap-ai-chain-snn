use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MessageBus {
    tx: broadcast::Sender<(String, Vec<u8>)>,
}

impl MessageBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub fn sender(&self) -> broadcast::Sender<(String, Vec<u8>)> {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(String, Vec<u8>)> {
        self.tx.subscribe()
    }

    pub fn send(&self, target: String, data: Vec<u8>) {
        let _ = self.tx.send((target, data));
    }
}
