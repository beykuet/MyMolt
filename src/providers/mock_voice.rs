// use anyhow::Result;
use base64;

/// A mock TTS provider that returns a hardcoded "Hello" audio sample.
/// In a real app, this would call une ElevenLabs, OpenAI TTS, etc.
pub struct MockVoiceProvider;

impl MockVoiceProvider {
    pub fn get_response_audio() -> String {
        // This is a tiny 1-second silent WAV or similar placeholder base64
        // For testing, even a small valid base64 string works if the frontend isn't 
        // strictly validating the codec headers yet, but let's use a "valid-ish" one.
        // Below is a base64 for a very short beep or silent chunk.
        "UklGRigAAABXQVZFZm10IBAAAAABAAEARKwAAIhYAQACABAAZGF0YQQAAAAAAA==".to_string()
    }
}
