//! Config types that were originally defined in their consuming modules.
//!
//! Centralized here to break circular dependencies between config and
//! the subsystems that use these types.

use serde::{Deserialize, Serialize};

use crate::traits::ChannelConfig;

// ── Trust ────────────────────────────────────────────────────────

/// Trust scoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct TrustConfig {
    #[serde(default = "default_initial_score")]
    pub initial_score: f64,
    #[serde(default = "default_decay_half_life")]
    pub decay_half_life_days: f64,
    #[serde(default = "default_regression_threshold")]
    pub regression_threshold: f64,
    #[serde(default = "default_correction_penalty")]
    pub correction_penalty: f64,
    #[serde(default = "default_success_boost")]
    pub success_boost: f64,
}

fn default_initial_score() -> f64 { 0.8 }
fn default_decay_half_life() -> f64 { 30.0 }
fn default_regression_threshold() -> f64 { 0.5 }
fn default_correction_penalty() -> f64 { 0.05 }
fn default_success_boost() -> f64 { 0.01 }

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            initial_score: default_initial_score(),
            decay_half_life_days: default_decay_half_life(),
            regression_threshold: default_regression_threshold(),
            correction_penalty: default_correction_penalty(),
            success_boost: default_success_boost(),
        }
    }
}

// ── Browser Delegate ─────────────────────────────────────────────

/// Browser delegation tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct BrowserDelegateConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_browser_cli")]
    pub cli_binary: String,
    #[serde(default)]
    pub chrome_profile_dir: String,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    #[serde(default = "default_browser_task_timeout")]
    pub task_timeout_secs: u64,
}

fn default_browser_cli() -> String { "claude".into() }
fn default_browser_task_timeout() -> u64 { 120 }

impl Default for BrowserDelegateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cli_binary: default_browser_cli(),
            chrome_profile_dir: String::new(),
            allowed_domains: Vec::new(),
            blocked_domains: Vec::new(),
            task_timeout_secs: default_browser_task_timeout(),
        }
    }
}

// ── Thinking ─────────────────────────────────────────────────────

/// Thinking level for chain-of-thought reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    Off,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    Max,
}

impl ThinkingLevel {
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" | "none" => Some(Self::Off),
            "minimal" | "min" => Some(Self::Minimal),
            "low" => Some(Self::Low),
            "medium" | "med" | "default" => Some(Self::Medium),
            "high" => Some(Self::High),
            "max" | "maximum" => Some(Self::Max),
            _ => None,
        }
    }
}

/// Thinking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ThinkingConfig {
    #[serde(default)]
    pub default_level: ThinkingLevel,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self { default_level: ThinkingLevel::Medium }
    }
}

// ── History Pruner ───────────────────────────────────────────────

/// History pruning configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct HistoryPrunerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_pruner_max_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_pruner_keep_recent")]
    pub keep_recent: usize,
    #[serde(default = "default_pruner_collapse")]
    pub collapse_tool_results: bool,
}

fn default_pruner_max_tokens() -> usize { 8192 }
fn default_pruner_keep_recent() -> usize { 4 }
fn default_pruner_collapse() -> bool { true }

impl Default for HistoryPrunerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_tokens: default_pruner_max_tokens(),
            keep_recent: default_pruner_keep_recent(),
            collapse_tool_results: default_pruner_collapse(),
        }
    }
}

// ── Eval ─────────────────────────────────────────────────────────

/// Auto-classification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct AutoClassifyConfig {
    #[serde(default)]
    pub simple_hint: Option<String>,
    #[serde(default)]
    pub standard_hint: Option<String>,
    #[serde(default)]
    pub complex_hint: Option<String>,
    #[serde(default = "default_cost_optimized_hint")]
    pub cost_optimized_hint: String,
}

fn default_cost_optimized_hint() -> String { "cost-optimized".to_string() }

impl Default for AutoClassifyConfig {
    fn default() -> Self {
        Self {
            simple_hint: None,
            standard_hint: None,
            complex_hint: None,
            cost_optimized_hint: default_cost_optimized_hint(),
        }
    }
}

/// Eval quality gate configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct EvalConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_min_quality_score")]
    pub min_quality_score: f64,
    #[serde(default = "default_eval_max_retries")]
    pub max_retries: u32,
}

fn default_min_quality_score() -> f64 { 0.5 }
fn default_eval_max_retries() -> u32 { 1 }

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_quality_score: default_min_quality_score(),
            max_retries: default_eval_max_retries(),
        }
    }
}

// ── Context Compression ──────────────────────────────────────────

/// Context compression configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ContextCompressionConfig {
    #[serde(default = "default_compression_enabled")]
    pub enabled: bool,
    #[serde(default = "default_threshold_ratio")]
    pub threshold_ratio: f64,
    #[serde(default = "default_protect_first_n")]
    pub protect_first_n: usize,
    #[serde(default = "default_protect_last_n")]
    pub protect_last_n: usize,
    #[serde(default = "default_max_passes")]
    pub max_passes: u32,
    #[serde(default = "default_summary_max_chars")]
    pub summary_max_chars: usize,
    #[serde(default = "default_source_max_chars")]
    pub source_max_chars: usize,
    #[serde(default = "default_compression_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub summary_model: Option<String>,
    #[serde(default = "default_identifier_policy")]
    pub identifier_policy: String,
    #[serde(default = "default_tool_result_retrim_chars")]
    pub tool_result_retrim_chars: usize,
    #[serde(default)]
    pub tool_result_trim_exempt: Vec<String>,
}

fn default_compression_enabled() -> bool { true }
fn default_threshold_ratio() -> f64 { 0.50 }
fn default_protect_first_n() -> usize { 3 }
fn default_protect_last_n() -> usize { 4 }
fn default_max_passes() -> u32 { 3 }
fn default_summary_max_chars() -> usize { 4000 }
fn default_source_max_chars() -> usize { 50000 }
fn default_compression_timeout() -> u64 { 60 }
fn default_identifier_policy() -> String { "strict".to_string() }
fn default_tool_result_retrim_chars() -> usize { 2000 }

impl Default for ContextCompressionConfig {
    fn default() -> Self {
        Self {
            enabled: default_compression_enabled(),
            threshold_ratio: default_threshold_ratio(),
            protect_first_n: default_protect_first_n(),
            protect_last_n: default_protect_last_n(),
            max_passes: default_max_passes(),
            summary_max_chars: default_summary_max_chars(),
            source_max_chars: default_source_max_chars(),
            timeout_secs: default_compression_timeout(),
            summary_model: None,
            identifier_policy: default_identifier_policy(),
            tool_result_retrim_chars: default_tool_result_retrim_chars(),
            tool_result_trim_exempt: Vec::new(),
        }
    }
}

// ── ClawdTalk ────────────────────────────────────────────────────

/// ClawdTalk voice channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct ClawdTalkConfig {
    pub api_key: String,
    pub connection_id: String,
    pub from_number: String,
    #[serde(default)]
    pub allowed_destinations: Vec<String>,
    #[serde(default)]
    pub webhook_secret: Option<String>,
}

impl ChannelConfig for ClawdTalkConfig {
    fn name() -> &'static str { "ClawdTalk" }
    fn desc() -> &'static str { "ClawdTalk Channel" }
}

// ── Voice Call ───────────────────────────────────────────────────

/// Voice telephony provider.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum VoiceProvider {
    #[default]
    Twilio,
    Telnyx,
    Plivo,
}

impl std::fmt::Display for VoiceProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Twilio => write!(f, "twilio"),
            Self::Telnyx => write!(f, "telnyx"),
            Self::Plivo => write!(f, "plivo"),
        }
    }
}

/// Voice call channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct VoiceCallConfig {
    #[serde(default)]
    pub provider: VoiceProvider,
    pub account_id: String,
    pub auth_token: String,
    pub from_number: String,
    #[serde(default = "default_voice_webhook_port")]
    pub webhook_port: u16,
    #[serde(default = "default_voice_true")]
    pub require_outbound_approval: bool,
    #[serde(default = "default_voice_true")]
    pub transcription_logging: bool,
    #[serde(default)]
    pub tts_voice: Option<String>,
    #[serde(default = "default_max_call_duration")]
    pub max_call_duration_secs: u64,
    /// Webhook base URL override (e.g. ngrok/Tailscale tunnel URL).
    #[serde(default)]
    pub webhook_base_url: Option<String>,
}

fn default_voice_webhook_port() -> u16 { 8090 }
fn default_voice_true() -> bool { true }
fn default_max_call_duration() -> u64 { 3600 }

impl ChannelConfig for VoiceCallConfig {
    fn name() -> &'static str { "Voice Call" }
    fn desc() -> &'static str { "Voice Call Channel" }
}

// ── Autonomy ─────────────────────────────────────────────────────

/// Runtime autonomy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum AutonomyLevel {
    ReadOnly,
    #[default]
    Supervised,
    Full,
}

// ── Domain Matcher ───────────────────────────────────────────────

/// Pattern matcher for domain-based security gating.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct DomainMatcher {
    patterns: Vec<String>,
}

impl DomainMatcher {
    /// Create a new matcher from explicit domains and category names.
    pub fn new(gated_domains: &[String], categories: &[String]) -> anyhow::Result<Self> {
        use std::collections::BTreeSet;
        let mut set = BTreeSet::new();
        for domain in gated_domains {
            set.insert(domain.trim().to_lowercase());
        }
        for cat in categories {
            for domain in Self::expand_category(cat)? {
                set.insert(domain);
            }
        }
        Ok(Self { patterns: set.into_iter().collect() })
    }

    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    pub fn matches(&self, domain: &str) -> bool {
        let lower = domain.to_lowercase();
        self.patterns.iter().any(|p| {
            if let Some(suffix) = p.strip_prefix("*.") {
                lower == suffix || lower.ends_with(&format!(".{suffix}"))
            } else {
                lower == *p
            }
        })
    }

    fn expand_category(category: &str) -> anyhow::Result<Vec<String>> {
        // Simplified — full implementation stays in root crate
        match category.to_lowercase().as_str() {
            "banking" | "finance" => Ok(vec![
                "*.chase.com", "*.bankofamerica.com", "*.wellsfargo.com",
            ].into_iter().map(String::from).collect()),
            _ => Ok(Vec::new()),
        }
    }
}

// ── SOP ──────────────────────────────────────────────────────────

/// Standard Operating Procedure engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub struct SopConfig {
    #[serde(default)]
    pub sops_dir: Option<String>,
    #[serde(default = "default_sop_execution_mode")]
    pub default_execution_mode: String,
    #[serde(default = "default_sop_max_concurrent_total")]
    pub max_concurrent_total: usize,
    #[serde(default = "default_sop_approval_timeout_secs")]
    pub approval_timeout_secs: u64,
    #[serde(default = "default_sop_max_finished_runs")]
    pub max_finished_runs: usize,
}

fn default_sop_execution_mode() -> String { "supervised".to_string() }
fn default_sop_max_concurrent_total() -> usize { 4 }
fn default_sop_approval_timeout_secs() -> u64 { 300 }
fn default_sop_max_finished_runs() -> usize { 100 }

impl Default for SopConfig {
    fn default() -> Self {
        Self {
            sops_dir: None,
            default_execution_mode: default_sop_execution_mode(),
            max_concurrent_total: default_sop_max_concurrent_total(),
            approval_timeout_secs: default_sop_approval_timeout_secs(),
            max_finished_runs: default_sop_max_finished_runs(),
        }
    }
}

// ── Eval complexity ──────────────────────────────────────────────

/// Complexity tier for auto-classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
pub enum ComplexityTier {
    Simple,
    Standard,
    Complex,
}

impl AutoClassifyConfig {
    /// Map a complexity tier to the configured hint, if any.
    pub fn hint_for(&self, tier: ComplexityTier) -> Option<&str> {
        match tier {
            ComplexityTier::Simple => self.simple_hint.as_deref(),
            ComplexityTier::Standard => self.standard_hint.as_deref(),
            ComplexityTier::Complex => self.complex_hint.as_deref(),
        }
    }
}
