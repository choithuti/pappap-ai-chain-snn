use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use crate::{snn_core::SNNCore, bus::MessageBus, crypto::CryptoEngine, managers};
use std::sync::Arc;

pub struct PappapChain {
    snn: Arc<SNNCore>,
    bus: Arc<MessageBus>,
    crypto: Arc<CryptoEngine>,
}

impl PappapChain {
    pub async fn new() -> Self {
        let snn = Arc::new(SNNCore::new());
        let bus = Arc::new(MessageBus::new());
        let key: [u8; 32] = *b"pappap2025snnblockchainkey32b!\0\0";
        let crypto = Arc::new(CryptoEngine::new(&key));

        let neurons = snn.neuron_count().await;
        let power = snn.power().await;
        println!("SNN Initialized: {neurons} neurons | Power: {power:.1}");

        Self { snn, bus, crypto }
    }

    pub async fn run(self) {
        tokio::spawn(managers::start_all(self.snn.clone(), self.bus.clone(), self.crypto.clone()));

        let snn_clone = self.snn.clone();
        tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(snn_clone.clone()))
                    .service(web::resource("/api/prompt").route(web::post().to(prompt_handler)))
                    .service(web::resource("/api/status").route(web::get().to(status_handler)))
            })
            .bind(("0.0.0.0", 8080))
            .unwrap()
            .run()
            .await
            .unwrap();
        });

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            interval.tick().await;
            let rate = self.snn.forward(1.0).await;
            let neurons = self.snn.neuron_count().await;
            println!("SNN Spike Rate: {rate:.4}  │  Active: ~{:.0}", rate * neurons as f32);
        }
    }
}

async fn prompt_handler(
    snn: web::Data<Arc<SNNCore>>,
    req: web::Json<serde_json::Value>,
) -> impl Responder {
    let prompt = req["prompt"].as_str().unwrap_or("hello");
    let (lang, response) = snn.detect_and_translate(prompt).await;
    let tts = snn.text_to_speech(&response, &lang);
    let neurons = snn.neuron_count().await;

    HttpResponse::Ok().json(serde_json::json!({
        "response": response,
        "language": lang,
        "tts": tts,
        "neurons": neurons,
        "status": "GENESIS NODE ALIVE"
    }))
}

async fn status_handler(snn: web::Data<Arc<SNNCore>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "PAPPAP AI CHAIN SNN IS ALIVE",
        "neurons": snn.neuron_count().await,
        "power": snn.power().await,
        "message": "Made in Vietnam – 22/11/2025"
    }))
}
