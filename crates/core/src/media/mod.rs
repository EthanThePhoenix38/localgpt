//! Media processing modules for LocalGPT.
//!
//! This module provides document loading and media processing capabilities.

pub mod audio;
pub mod document;

pub use audio::{SttConfig, SttProvider, SttRegistry};
pub use document::DocumentLoaders;
