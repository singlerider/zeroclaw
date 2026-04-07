//! Channel implementations for ZeroClaw messaging platform integrations.
//!
//! Each channel implements the `Channel` trait from `zeroclaw-types`.
//! The dispatch/runtime logic (process_channel_message, start_channels)
//! stays in the root crate.

// Infrastructure re-exports from zeroclaw-infra
pub mod debounce;
pub mod session_backend;
pub mod session_sqlite;
pub mod session_store;
pub mod stall_watchdog;

// Channel implementations
pub mod bluesky;
pub mod clawdtalk;
pub mod cli;
pub mod dingtalk;
pub mod discord_history;
pub mod discord;
#[cfg(feature = "channel-email")]
pub mod email_channel;
#[cfg(feature = "channel-email")]
pub mod gmail_push;
pub mod imessage;
pub mod irc;
#[cfg(feature = "channel-lark")]
pub mod lark;
pub mod link_enricher;
pub mod linq;
pub mod mattermost;
pub mod media_pipeline;
pub mod mochat;
pub mod nextcloud_talk;
#[cfg(feature = "channel-nostr")]
pub mod nostr;
pub mod notion;
pub mod qq;
pub mod reddit;
pub mod signal;
pub mod slack;
pub mod traits;
pub mod transcription;
pub mod tts;
pub mod twitter;
pub mod util;
pub mod voice_call;
#[cfg(feature = "voice-wake")]
pub mod voice_wake;
pub mod wati;
pub mod webhook;
pub mod wecom;
pub mod whatsapp;
#[cfg(feature = "whatsapp-web")]
pub mod whatsapp_storage;
#[cfg(feature = "whatsapp-web")]
pub mod whatsapp_web;

// Re-export key types
pub use traits::{Channel, SendMessage, ChannelMessage};
