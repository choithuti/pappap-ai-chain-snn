// src/wallet_mobile.rs – API cho Mobile Wallet
use actix_web::{post, web, HttpResponse};

#[post("/mobile/wallet/create")]
async fn create_wallet() -> impl Responder {
    let (address, mnemonic) = Wallet::new();
    HttpResponse::Ok().json(serde_json::json!({
        "address": address,
        "mnemonic": mnemonic,
        "message": "Ví Pappap AI Chain đã tạo – Made in Vietnam"
    }))
} 