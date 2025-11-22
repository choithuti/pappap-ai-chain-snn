// src/libp2p_swarm.rs
use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identity,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    PeerId, Swarm, Transport, Multiaddr, Noise, Yamux,
};
use libp2p::gossipsub::Event as GossipEvent;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::{bus::MessageBus, crypto::CryptoEngine, snn_core::SNNCore};
use std::sync::Arc;

// Custom Network Behaviour: Gossipsub + KeepAlive
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct PappapBehaviour {
    gossipsub: gossipsub::Behaviour,
    keep_alive: libp2p::ping::Behaviour,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Gossipsub(GossipEvent),
    Ping(libp2p::ping::Event),
}

impl From<gossipsub::Event> for MyBehaviourEvent {
    fn from(e: GossipEvent) -> Self { MyBehaviourEvent::Gossipsub(e) }
}
impl From<libp2p::ping::Event> for MyBehaviourEvent {
    fn from(e: libp2p::ping::Event) -> Self { MyBehaviourEvent::Ping(e) }
}

pub struct P2PSwarm {
    swarm: Swarm<PappapBehaviour>,
    rx_from_bus: mpsc::UnboundedReceiver<(String, Vec<u8>)>,
    bus_tx: MessageBus,
    crypto: Arc<CryptoEngine>,
    snn: Arc<SNNCore>,
    known_peers: HashSet<PeerId>,
}

impl P2PSwarm {
    pub async fn new(
        bus: MessageBus,
        crypto: Arc<CryptoEngine>,
        snn: Arc<SNNCore>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local Peer ID: {}", local_peer_id);

        let transport = libp2p::tcp::async_io::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(Noise::new(&local_key)?)
            .multiplex(Yamux::default())
            .timeout(Duration::from_secs(20))
            .boxed();

        let topic = IdentTopic::new("pappap-ai-chain-snn-2025");

        let mut gossip_cfg = gossipsub::ConfigBuilder::default();
        gossip_cfg.heartbeat_interval(Duration::from_secs(5));
        gossip_cfg.validation_mode(gossipsub::ValidationMode::Strict);
        let gossip_cfg = gossip_cfg.build()?;

        let mut behaviour = PappapBehaviour {
            gossipsub: gossipsub::Behaviour::new(
                MessageAuthenticity::Signed(local_key.clone()),
                gossip_cfg,
            )?,
            keep_alive: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
        };

        behaviour.gossipsub.subscribe(&topic)?;

        let mut swarm = SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id)
            .idle_connection_timeout(Duration::from_secs(60))
            .build();

        // Listen on all interfaces
        swarm.listen_on("/ip4/0.0.0.0/tcp/9000".parse()?)?;
        swarm.listen_on("/ip6/::/tcp/9000".parse()?)?;

        let (tx_to_swarm, rx_from_bus) = mpsc::unbounded_channel();

        // Auto-connect to bootstrap/bootnodes (có th? thêm sau)
        let bootnodes = vec![
            "/dns/bootnode.pappap.ai/tcp/9000/p2p/12D3KooW...".parse::<Multiaddr>().ok(),
        ];
        for addr in bootnodes.into_iter().flatten() {
            if let Err(e) = swarm.dial(addr.clone()) {
                warn!("Failed to dial bootnode {}: {}", addr, e);
            }
        }

        Ok(Self {
            swarm,
            rx_from_bus,
            bus_tx: bus,
            crypto,
            snn,
            known_peers: HashSet::new(),
        })
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<(String, Vec<u8>)> {
        self.rx_from_bus.sender()
    }

    pub async fn run(mut self) {
        info!("P2P Swarm started – PappapAIChain Testnet LIVE");
        loop {
            tokio::select! {
                // Nh?n tin t? các manager (qua bus ? swarm)
                Some((target, data)) = self.rx_from_bus.recv() => {
                    if target == "gossip" || target == "all" {
                        let encrypted = self.crypto.encrypt(&data);
                        let topic = IdentTopic::new("pappap-ai-chain-snn-2025");
                        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, encrypted) {
                            error!("Gossip publish failed: {}", e);
                        }
                    }
                }

                // Nh?n tin t? m?ng P2P
                event = self.swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {}", address);
                        }
                        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(GossipEvent::Message {
                            propagation_source,
                            message,
                            ..
                        })) => {
                            self.known_peers.insert(propagation_source);

                            // SNN ki?m tra gói tin tru?c khi gi?i mã
                            let size_score = message.data.len() as f32 / 2048.0;
                            let threat = self.snn.forward(size_score.clamp(0.0, 5.0)).await;

                            if threat > 0.94 {
                                warn!("Dropped malicious packet from {} (threat: {:.3})", propagation_source, threat);
                                continue;
                            }

                            match self.crypto.decrypt(&message.data) {
                                Ok(plain) => {
                                    // Forward vào internal bus
                                    let _ = self.bus_tx.send(("gossip".to_string(), plain));
                                }
                                Err(_) => {
                                    warn!("Failed to decrypt message from {}", propagation_source);
                                }
                            }
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            info!("Connected to peer: {}", peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            warn!("Disconnected from peer: {}", peer_id);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}