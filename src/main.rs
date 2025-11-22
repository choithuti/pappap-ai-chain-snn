mod chain;
mod config;
mod crypto;
mod bus;
mod snn_core;
mod libp2p_swarm;
mod managers;

use chain::PappapChain;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() {
    init();
    println!("PappapAIChain SNN - November 21, 2025");
    println!("   World's First Real Spiking Neural Network Blockchain\n");

    let chain = PappapChain::new().await;
    chain.run().await;
}