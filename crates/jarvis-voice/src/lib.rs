//! Voice input (speech-to-text) and output (text-to-speech) abstraction.
//!
//! Not implemented in Phase 1. This crate exists so the workspace layout
//! and crate boundaries are correct from the start; it exposes only the
//! trait shape that wake-word detection, STT and TTS backends will
//! eventually implement.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VoiceError {
    #[error("voice support is not implemented yet")]
    NotImplemented,
}

pub type VoiceResult<T> = Result<T, VoiceError>;

/// Converts spoken audio to text. No implementation exists yet.
pub trait SpeechToText: Send + Sync {
    fn transcribe(&self, _audio: &[u8]) -> VoiceResult<String> {
        Err(VoiceError::NotImplemented)
    }
}

/// Converts text to spoken audio. No implementation exists yet.
pub trait TextToSpeech: Send + Sync {
    fn synthesize(&self, _text: &str) -> VoiceResult<Vec<u8>> {
        Err(VoiceError::NotImplemented)
    }
}
