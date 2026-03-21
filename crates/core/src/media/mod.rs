//! Media processing modules for LocalGPT.
//!
//! This module provides document loading, audio transcription, and text-to-speech capabilities.

pub mod audio;
pub mod cache;
pub mod document;
#[cfg(feature = "image-resize")]
pub mod image_optimize;
pub mod tts;

pub use audio::{SttConfig, SttProvider, SttRegistry};
pub use document::DocumentLoaders;
pub use tts::{TtsConfig, TtsProvider, TtsRegistry};
