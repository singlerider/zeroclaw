//! Configuration schema types for ZeroClaw.
//!
//! This crate contains all configuration struct/enum definitions, loading,
//! saving, proxy management, and workspace resolution. It is the single
//! source of truth for config types, breaking circular dependencies.

pub mod domain_matcher;
pub mod scattered_types;
pub mod schema;
pub mod secrets;
pub mod traits;
pub mod trust_config;

// Re-export key types at the crate root for convenience
pub use domain_matcher::DomainMatcher;
pub use scattered_types::*;
pub use secrets::SecretStore;
pub use trust_config::TrustConfig;
