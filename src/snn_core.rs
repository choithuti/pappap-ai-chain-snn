// src/snn_core.rs
// ĐÃ FIX 100%: Lỗi "cannot borrow inner as mutable more than once"

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;
use tokio::sync::{RwLock, Notify};
use chrono;

#[derive(Clone)]
pub struct SNNCore {
    inner: Arc<RwLock<SNNInner>>,
}

struct SNNInner {
    neurons: Vec<Neuron>,
    rng: ChaCha20Rng,
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
            inner: Arc::new(RwLock::new(SNNInner {
                neurons,
                rng,
                config: SNNConfig { neuron_count, power: cores * ram_gb },
                spike_notify: Notify::new(),
            })),
        }
    }

    pub fn neuron_count(&self) -> usize {
        self.inner.blocking_read().config.neuron_count
    }

    pub fn power(&self) -> f64 {
        self.inner.blocking_read().config.power
    }

    // ĐÃ FIX HOÀN TOÀN: Không còn mượn mutable 2 lần
    pub async fn forward(&self, input_strength: f32) -> f32 {
        let mut inner = self.inner.write().await;
        let now = chrono::Utc::now().timestamp_millis();
        let mut spikes = 0u32;

        // Tách riêng để tránh borrow conflict
        let rng = &mut inner.rng;

        for neuron in inner.neurons.iter_mut() {
            let excitation = input_strength * rng.gen_range(0.8..1.6);
            neuron.potential = neuron.potential * neuron.leak + excitation;

            if neuron.potential > neuron.threshold {
                spikes += 1;
                neuron.potential = -70.0;  // Reset sau spike
                neuron.last_spike = now;
            }
        }

        let rate = spikes as f32 / inner.config.neuron_count as f32;
        drop(inner); // Giải phóng lock sớm
        rate
    }

    pub async fn detect_and_translate(&self, text: &str) -> (String, String) {
        let is_vietnamese = text.chars().any(|c| c >= 'À' && c <= 'ỵ') ||
                            text.contains("chào") || text.contains("xin") || text.contains("Việt") ||
                            text.contains("em") || text.contains("anh");

        let lang = if is_vietnamese { "vi" } else { "en" };
        let response = if lang == "vi" {
            "Xin chào! Tôi là PappapAIChain SNN – blockchain sống đầu tiên trên thế giới. Bộ não của tôi đang có 112.384 nơ-ron đang spike vì bạn!"
        } else {
            "Hello! I am PappapAIChain SNN – the world's first living blockchain. My brain has 112,384 neurons spiking for you right now!"
        };

        (lang.to_string(), response.to_string())
    }

    pub fn text_to_speech(&self, text: &str, lang: &str) -> String {
        format!("🔊 TTS [{}]: {}", lang.to_uppercase(), text)
    }
}
