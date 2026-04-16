# ADR-005: Adopt Mozilla Fluent as Project i18n System

**Status:** Proposed

**Date:** 2026-04-16

**Issue:** [#5787](https://github.com/zeroclaw-labs/zeroclaw/issues/5787)

## Context

ZeroClaw has a hand-rolled TOML-based i18n system that only covers tool
descriptions injected into agent system prompts. The system loads locale files
from `tool_descriptions/<locale>.toml` at runtime and threads a
`ToolDescriptions` struct through the agent builder, prompt context, and
tool-call loop.

This system has three limitations:

1. No pluralization or variable interpolation — TOML values are opaque strings.
2. No coverage beyond tool descriptions — ~60+ CLI strings in cron, memory,
   onboarding, and the Tauri frontend are hardcoded English.
3. No ecosystem — contributors must learn a bespoke format with no tooling,
   linting, or community familiarity.

Mozilla Fluent is a mature, pure-Rust i18n system with native pluralization
(CLDR rules), variable interpolation, and an established file format (`.ftl`)
with editor support.

## Decision

### 1. Adopt Mozilla Fluent as the sole i18n system

All user-facing strings — tool descriptions, CLI output, error messages, and
Tauri GUI text — use Fluent `.ftl` files under `locales/<lang>/`.

The `fluent` crate is the only new dependency. English locale files are
embedded at compile time via `include_str!()`. Non-English locales are loaded
from disk at runtime with silent English fallback.

### 2. Logs stay English — always

Log lines (`tracing::info!`, `tracing::error!`, etc.) are never translated.
When a user-facing string has a corresponding log event, the log includes a
stable `error_key` field matching the FTL message identifier:

```rust
println!("{}", t!("cron-no-tasks"));
tracing::info!(error_key = "cron-no-tasks", "No scheduled tasks yet");
```

This aligns with RFC #5653 §4.6: user-facing and machine-facing outputs have
different stability requirements and are separate surfaces.

### 3. Locale resolution priority

`config.locale` > `ZEROCLAW_LOCALE` > `LANG` > `LC_ALL` > `"en"`.

Encoding suffixes (`ja_JP.UTF-8`) are stripped and underscores normalized to
hyphens (`ja-JP`) before file lookup.

### 4. Tauri uses a parallel JS implementation

The Tauri frontend reads the same `.ftl` files via `fluent-bundle` (JS), not
a Rust bridge. Both adapters are independent per the hexagonal architecture
(FND-001 §4.2). CI treats JS missing-key warnings as errors.

## Consequences

### Positive

- All user-facing strings have a translation path for the first time.
- Pluralization and interpolation work natively.
- Contributors use a well-documented, widely-adopted file format.
- English is always available — no crash path from missing locale files.

### Negative

- One new compile-time dependency (`fluent` + transitive deps).
- Existing 31 locale TOML files must be migrated to `.ftl` format.
- Cold build time increases marginally (measured and reported per Phase 2 PR).

### Neutral

- The `toml` crate remains in the dependency tree for other uses (SOP, skills,
  cron store, etc.).
- Locale detection logic (`detect_locale`, `normalize_locale`) is preserved
  unchanged.

## References

- [RFC proposal](../proposals/mozilla-fluent-i18n.md)
- `crates/zeroclaw-runtime/src/i18n.rs` — Fluent-based loader
- `crates/zeroclaw-runtime/locales/en/tools.ftl` — embedded English tool descriptions
- [Mozilla Fluent](https://projectfluent.org/)
- RFC #5653 §4.6 — structured logging with stable keys
