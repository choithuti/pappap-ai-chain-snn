mod chain;
mod snn_core;
mod auto_learn;
mod bus;
mod crypto;
mod managers;

use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() {
    init();
    println!("PAPPAP AI CHAIN SNN v0.3 – Made in Vietnam – 24/11/2025");
    println!("World's First Living, Self-Learning, Ethical Blockchain\n");

    chain::PappapChain::new().await.run().await;
}
