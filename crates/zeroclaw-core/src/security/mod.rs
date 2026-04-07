pub mod audit;
pub mod bubblewrap;
pub mod detect;
pub mod docker;
pub mod estop;
pub mod firejail;
pub mod iam_policy;
pub mod landlock;
pub mod leak_detector;
pub mod nevis;
pub mod otp;
pub mod pairing;
pub mod playbook;
pub mod prompt_guard;
pub mod seatbelt;
pub mod secrets;
pub mod traits;
pub mod vulnerability;
pub mod webauthn;

pub use zeroclaw_config::security_policy::*;
pub use zeroclaw_config::domain_matcher::DomainMatcher;
pub use secrets::SecretStore;

pub fn redact(value: &str) -> String {
    let char_count = value.chars().count();
    if char_count <= 4 {
        "***".to_string()
    } else {
        let prefix: String = value.chars().take(4).collect();
        format!("{prefix}***")
    }
}

// Explicit re-exports for commonly used types
pub use detect::create_sandbox;
pub use leak_detector::{LeakDetector, LeakResult};
pub use traits::NoopSandbox;
pub use otp::OtpValidator;
pub use estop::EstopManager;
pub use estop::EstopLevel;
pub use estop::ResumeSelector;
pub use estop::EstopState;
