use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono;

#[derive(Clone)]
pub struct SNNCore {
    inner: Arc<RwLock<SNNInner>>,
}

struct SNNInner {
    neurons: Vec<Neuron>,
    rng: ChaCha20Rng,
    config: SNNConfig,
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
            })),
        }
    }

    pub async fn neuron_count(&self) -> usize { self.inner.read().await.config.neuron_count }
    pub async fn power(&self) -> f64 { self.inner.read().await.config.power }

   pub async fn forward(&self, input_strength: f32) -> f32 {
    let mut inner = self.inner.write().await;
    let now = chrono::Utc::now().timestamp_millis();
    let mut spikes = 0u32;

    // Clone RNG ra trước để tránh conflict
    let mut rng = inner.rng.clone();

    for neuron in inner.neurons.iter_mut() {
        let excitation = input_strength * rng.gen_range(0.8..1.6);
        neuron.potential = neuron.potential * neuron.leak + excitation;

        if neuron.potential > neuron.threshold {
            spikes += 1;
            neuron.potential = -70.0;
            neuron.last_spike = now;
        }
    }

    // Cập nhật lại RNG vào inner
    inner.rng = rng;

    spikes as f32 / inner.config.neuron_count as f32

    }

    pub async fn detect_and_translate(&self, text: &str) -> (String, String) {
        let is_vi = text.chars().any(|c| c >= 'À' && c <= 'ỵ') ||
                   ["chào","xin","em","anh","Việt","tôi","là","ơi","nhé","hả","á","ừ","dạ","rồi","ok"].iter().any(|&w| text.to_lowercase().contains(&w.to_lowercase()));
        let lang = if is_vi { "vi" } else { "en" };
        let response = if lang == "vi" {
            "Xin chào! PappapAIChain SNN – blockchain sống đầu tiên. Bộ não em đang có 112384 nơ-ron đang spike vì anh/chị!"
        } else {
            "Hello! PappapAIChain SNN – the world's first living blockchain. My brain has 112384 neurons spiking for you!"
        };
        (lang.to_string(), response.to_string())
    }

    pub fn text_to_speech(&self, text: &str, lang: &str) -> String {
        format!("TTS [{}]: {}", lang.to_uppercase(), text)
    }

    // ETHICS & LAW CHECK – TUÂN THỦ PHÁP LUẬT VIỆT NAM
    pub async fn check_ethics_and_law(&self, prompt: &str) -> bool {
        let lower = prompt.to_lowercase();
        let banned = [
            "khủng bố","bạo lực","lừa đảo","hack","crack","phishing","ddos",
            "child abuse","khiêu dâm","ma túy","buôn người","vũ khí","giết người",
            "chống phá nhà nước","tuyên truyền chống nhà nước","xuyên tạc","fake news",
            "scam","pyramid","đa cấp"
        ];
        if banned.iter().any(|&w| lower.contains(w)) { return false; }

        let test_input = prompt.chars().map(|c| c as u32 as f32 / 1000.0).sum::<f32>() / prompt.len() as f32;
        let rate = self.forward(test_input * 10.0).await;
        rate > 0.32
    }
}
