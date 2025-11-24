// src/libp2p_p2p.rs – Full libp2p mesh + auto-discovery
use libp2p::{
    gossipsub, identity, kad::{Kademlia, KademliaConfig}, mdns, noise, swarm::{SwarmBuilder, SwarmEvent}, tcp, yamux, PeerId, Swarm, Multiaddr
};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct P2PNode {
    swarm: Swarm<libp2p::swarm::Behaviour<BehaviourEvent>>,
    rx: mpsc::UnboundedReceiver<(String, Vec<u8>)>,
}

#[derive(libp2p::NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
pub struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: Kademlia<libp2p::kad::store::MemoryStore>,
    mdns: mdns::tokio::Behaviour,
    keep_alive: libp2p::ping::Behaviour,
}

#[derive(Debug)]
pub enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kademlia(libp2p::kad::KademliaEvent),
    Mdns(mdns::Event),
    Ping(libp2p::ping::Event),
}

impl From<gossipsub::Event> for BehaviourEvent { fn from(e: gossipsub::Event) -> Self { Self::Gossipsub(e) } }
impl From<libp2p::kad::KademliaEvent> for BehaviourEvent { fn from(e: libp2p::kad::KademliaEvent) -> Self { Self::Kademlia(e) } }
impl From<mdns::Event> for BehaviourEvent { fn from(e: mdns::Event) -> Self { Self::Mdns(e) } }
impl From<libp2p::ping::Event> for BehaviourEvent { fn from(e: libp2p::ping::Event) -> Self { Self::Ping(e) } }

impl P2PNode {
    pub async fn new() -> Self {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local Peer ID: {local_peer_id}");

        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .build()
            .unwrap();

        let mut behaviour = Behaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossipsub_config,
            ).unwrap(),
            kademlia: Kademlia::new(local_peer_id, libp2p::kad::store::MemoryStore::new(local_peer_id)),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id).unwrap(),
            keep_alive: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
        };

        behaviour.gossipsub.subscribe(&gossipsub::IdentTopic::new("pappap-snn-v0.4")).unwrap();

        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("P2P Listening on {address}");
                        }
                        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                            // Xử lý tin nhắn từ mạng
                            println!("Received P2P message from {}", message.source.unwrap_or_default());
                        }
                        _ => {}
                    }
                    Some((target, data)) = rx.recv() => {
                        if target == "global" || target == "all" {
                            swarm.behaviour_mut().gossipsub.publish(gossipsub::IdentTopic::new("pappap-snn-v0.4"), data).unwrap();
                        }
                    }
                }
            }
        });

        Self { swarm, rx }
    }
}