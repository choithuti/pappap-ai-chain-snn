use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use crate::{snn_core::SNNCore, auto_learn::auto_learn_and_answer};
use std::sync::Arc;

pub struct PappapChain {
    snn: Arc<SNNCore>,
}

impl PappapChain {
    pub async fn new() -> Self {
        let snn = Arc::new(SNNCore::new());
        let neurons = snn.neuron_count().await;
        let power = snn.power().await;
        println!("SNN Initialized: {neurons} neurons | Power: {power:.1}");
        Self { snn }
    }

    pub async fn run(self) {
        let snn_clone = self.snn.clone();
        tokio::spawn(async move {
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(snn_clone.clone()))
                    .service(web::resource("/api/prompt").route(web::post().to(prompt_handler)))
                    .service(web::resource("/api/status").route(web::get().to(status_handler)))
                    .service(web::resource("/api/wallet/balance").route(web::get().to(wallet_balance_handler)))
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
    let prompt = req["prompt"].as_str().unwrap_or("hello").trim();

    // ETHICS & LAW CHECK
    if !snn.check_ethics_and_law(prompt).await {
        return HttpResponse::Ok().json(serde_json::json!({
            "response": "Yêu cầu vi phạm đạo đức hoặc pháp luật Việt Nam (Luật An ninh mạng 2018). Pappap AI từ chối xử lý.",
            "status": "rejected_by_ethics"
        }));
    }

    // SELF-LEARNING IF COMPLEX
    let is_complex = prompt.contains("luật") || prompt.contains("bài tập") || prompt.contains("giải") || prompt.len() > 80;
    if is_complex {
        let learned = auto_learn_and_answer(prompt).await;
        return HttpResponse::Ok().json(serde_json::json!({
            "response": learned,
            "self_learned": true,
            "neurons": snn.neuron_count().await
        }));
    }

    let (lang, response) = snn.detect_and_translate(prompt).await;
    let tts = snn.text_to_speech(&response, &lang);
    HttpResponse::Ok().json(serde_json::json!({
        "response": response,
        "language": lang,
        "tts": tts,
        "neurons": snn.neuron_count().await,
        "status": "GENESIS NODE ALIVE – Ethical & Self-Learning"
    }))
}

async fn status_handler(snn: web::Data<Arc<SNNCore>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "PAPPAP AI CHAIN SNN v0.3 – ETHICAL & SELF-LEARNING",
        "neurons": snn.neuron_count().await,
        "power": snn.power().await,
        "compliance": "Vietnam Law Compliant"
    }))
}

async fn wallet_balance_handler(_: web::Data<Arc<SNNCore>>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "address": "MAPLE0276_GENESIS_001",
        "balance": "120001287644.42000000",
        "total_neurons": 561920,
        "compliance": "Vietnam Law Compliant"
    }))
}
