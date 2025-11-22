use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct SNNCore {
    inner: Arc<SNNInner>,
}

struct SNNInner {
    neurons: Vec<Neuron>,
    rng: parking_lot::RwLock<ChaCha20Rng>,
    config: SNNConfig,
    spike_notify: Notify,
}

#[derive(Clone, Copy)]
struct Neuron {
    potential: f32,
    threshold: f32,
    leak: f32,
    last_spike: i64,
}

#[derive(Clone)]
pub struct SNNConfig {
    pub neuron_count: usize,
    pub power: f64,
}

impl SNNCore {
    pub fn new() -> Self {
        let cores = num_cpus::get() as f64;
        let ram_gb = sys_info::mem_info().map(|m| m.total as f64 / 1e9).unwrap_or(8.0);
        let multiplier = if cfg!(feature = "high-neuron-mode") { 8.0 } else { 1.0 };
        let neuron_count = ((8000.0 * cores * ram_gb * multiplier) as usize).max(5000);

        let mut rng = ChaCha20Rng::from_entropy();
        let neurons: Vec<_> = (0..neuron_count).map(|_| Neuron {
            potential: -70.0,
            threshold: -55.0 + rng.gen_range(-10.0..10.0),
            leak: 0.94,
            last_spike: 0,
        }).collect();

        Self {
            inner: Arc::new(SNNInner {
                neurons,
                rng: parking_lot::RwLock::new(rng),
                config: SNNConfig { neuron_count, power: cores * ram_gb },
                spike_notify: Notify::new(),
            }),
        }
    }

    pub fn neuron_count(&self) -> usize { self.inner.config.neuron_count }
    pub fn power(&self) -> f64 { self.inner.config.power }

    pub async fn forward(&self, input_strength: f32) -> f32 {
        let mut rng = self.inner.rng.write();
        let now = chrono::Utc::now().timestamp_millis();
        let mut spikes = 0u32;

        for n in self.inner.neurons.iter_mut() {
            n.potential = n.potential * n.leak + input_strength * rng.gen_range(0.8..1.6);
            if n.potential > n.threshold {
                spikes += 1;
                n.potential = -70.0;
                n.last_spike = now;
            }
        }

        let rate = spikes as f32 / self.inner.config.neuron_count as f32;
        self.inner.spike_notify.notify_waiters();
        rate
    }

    pub async fn detect_and_translate(&self, text: &str) -> (String, String) {
        let lang = if text.contains("chào") || text.contains("xin") || text.contains("Việt") { "vi" }
                   else { "en" };

        let response = if lang == "vi" {
            "Xin chào! Tôi là PappapAIChain SNN – blockchain sống đầu tiên trên thế giới"
        } else {
            "Hello! I am PappapAIChain SNN – the world's first living blockchain"
        };

        (lang.to_string(), response.to_string())
    }

    pub fn text_to_speech(&self, text: &str, lang: &str) -> String {
        format!("TTS [{}]: {}", lang, text)
    }
}
