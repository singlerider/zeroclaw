//! Core trait definitions and shared message types for ZeroClaw.
//!
//! This crate contains the fundamental abstractions that all ZeroClaw subsystems
//! depend on: the `Channel`, `Provider`, `Tool`, `Memory`, and `Observer` traits,
//! along with the message types they exchange (`ChannelMessage`, `SendMessage`,
//! `ChatMessage`, `ToolResult`, etc.).
//!
//! Extracting these into a leaf crate with minimal dependencies ensures that
//! changing a channel implementation, tool, or provider doesn't trigger
//! recompilation of the trait definitions themselves.

pub mod channel;
pub mod media;
pub mod provider;
pub mod summarizer;
pub mod tool;
