// src/voice.rs – Voice AI trực tiếp trên chain
use tract_ndarray::Array;
use whisper_rs::{WhisperContext, FullParams};

pub struct VoiceEngine {
    whisper: WhisperContext,
    vits: VitsModel,
}

impl VoiceEngine {
    pub fn new() -> Self {
        let whisper = WhisperContext::new("models/whisper-tiny.bin").unwrap();
        let vits = VitsModel::from_ipfs("Qm...").unwrap();
        Self { whisper, vits }
    }

    pub fn speech_to_text(&self, audio: &[i16]) -> String {
        let mut state = self.whisper.create_state().unwrap();
        state.full(FullParams::default(), audio).unwrap();
        let mut result = String::new();
        while state.full_n_tokens() > 0 {
            state.full_get_token_text(0).unwrap();
        }
        result
    }

    pub fn text_to_speech(&self, text: &str) -> Vec<u8> {
        self.vits.synthesize(text, "vi_vn")
    }
}