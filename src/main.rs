mod chain;
mod snn_core;
mod bus;
mod crypto;
mod managers;

use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() {
    init();
    println!("PAPPAP AI CHAIN SNN v0.2 – 22/11/2025");
    println!("   World's First Real Spiking Neural Network Blockchain\n");

    chain::PappapChain::new().await.run().await;
}
