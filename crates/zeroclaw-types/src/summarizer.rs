//! Minimal summarizer trait for LLM-based text processing.
//!
//! This trait exists to decouple subsystems (like memory consolidation)
//! from the full `Provider` trait, which has streaming methods that
//! depend on `reqwest` and `futures_util`.

use async_trait::async_trait;

/// A minimal interface for LLM text processing.
///
/// Any `Provider` can be wrapped to implement this trait.
#[async_trait]
pub trait Summarizer: Send + Sync {
    /// Process text with an optional system prompt, returning the LLM's response.
    async fn summarize(
        &self,
        system_prompt: Option<&str>,
        text: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String>;
}
