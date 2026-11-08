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

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopVoice;
    impl SpeechToText for NoopVoice {}
    impl TextToSpeech for NoopVoice {}

    #[test]
    fn default_speech_to_text_is_not_implemented() {
        let voice = NoopVoice;
        assert!(matches!(
            voice.transcribe(&[]),
            Err(VoiceError::NotImplemented)
        ));
    }

    #[test]
    fn default_text_to_speech_is_not_implemented() {
        let voice = NoopVoice;
        assert!(matches!(
            voice.synthesize("hello"),
            Err(VoiceError::NotImplemented)
        ));
    }
}
