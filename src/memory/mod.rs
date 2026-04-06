//! Memory subsystem — re-exported from `zeroclaw-memory`.
//!
//! Module definitions and implementations live in the `zeroclaw-memory` workspace crate.
//! This module re-exports everything and adds the CLI handler + factory functions
//! that depend on root crate types.

// Re-export all public types from zeroclaw-memory
pub use zeroclaw_memory::*;

// CLI handler stays in root crate (depends on crate::MemoryCommands)
pub mod cli;

// Battle tests stay in root crate
#[cfg(test)]
mod battle_tests;

// Implement Summarizer for any Provider (bridges zeroclaw-types and root crate)
use async_trait::async_trait;
use zeroclaw_types::summarizer::Summarizer;

/// Wrapper that adapts a `Provider` to the `Summarizer` trait for memory consolidation.
pub struct ProviderSummarizer<'a>(pub &'a dyn crate::providers::traits::Provider);

#[async_trait]
impl Summarizer for ProviderSummarizer<'_> {
    async fn summarize(
        &self,
        system_prompt: Option<&str>,
        text: &str,
        model: &str,
        temperature: f64,
    ) -> anyhow::Result<String> {
        self.0.chat_with_system(system_prompt, text, model, temperature).await
    }
}
