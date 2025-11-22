use actix_web::{web, App, HttpServer, HttpResponse};
use crate::{snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine, config::AppConfig};
use std::sync::Arc;

pub struct PappapChain {
    snn: Arc<SNNCore>,
    bus: Arc<MessageBus>,
    crypto: Arc<CryptoEngine>,
    config: AppConfig,
}

impl PappapChain {
    pub async fn new() -> Self {
        let config = AppConfig::load();
        let crypto = Arc::new(CryptoEngine::new(&config.encryption_key));
        let snn = Arc::new(SNNCore::new());
        let bus = Arc::new(MessageBus::new());

        println!("SNN Initialized: {} neurons | Power: {:.1}", 
                 snn.neuron_count(), snn.power());

        Self { snn, bus, crypto, config }
    }

    pub async fn run(self) {
        let snn = self.snn.clone();
        let bus = self.bus.clone();
        let crypto = self.crypto.clone();

        // Spawn tất cả managers
        crate::managers::start_all(snn.clone(), bus.clone(), crypto.clone());

        // HTTP API
        let api_snn = snn.clone();
        tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(api_snn.clone()))
                    .service(web::resource("/api/prompt").route(web::post().to(prompt_handler)))
                    .service(web::resource("/api/status").route(web::get().to(status_handler)))
            })
            .bind(&self.config.listen_addr)
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        // Main SNN tick (100ms)
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let rate = snn.forward(1.0).await;
            println!("SNN Spike Rate: {:.4}  │  Active: ~{:.0}", 
                     rate, rate * snn.neuron_count() as f32);
        }
    }
}