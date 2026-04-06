//! Channel infrastructure utilities for ZeroClaw.
//!
//! Contains reusable components used by channel implementations:
//! message debouncing, stall detection, and session persistence.

pub mod debounce;
pub mod session_backend;
pub mod session_sqlite;
pub mod session_store;
pub mod stall_watchdog;
