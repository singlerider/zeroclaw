//! Configuration schema — re-exported from `zeroclaw-config`.
//!
//! All config types, loading, saving, proxy management, and validation logic
//! now live in the `zeroclaw-config` workspace crate. This module re-exports
//! everything for backwards compatibility.

pub use zeroclaw_config::domain_matcher::DomainMatcher;
pub use zeroclaw_config::schema::*;
pub use zeroclaw_config::scattered_types::*;
pub use zeroclaw_config::secrets::SecretStore;
pub use zeroclaw_config::traits::*;
pub use zeroclaw_config::trust_config::TrustConfig;
