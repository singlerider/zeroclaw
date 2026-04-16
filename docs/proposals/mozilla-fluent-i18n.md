# RFC: Replace TOML i18n with Mozilla Fluent; remove in-repo translated docs

**Issue:** [#5787](https://github.com/zeroclaw-labs/zeroclaw/issues/5787)

**Status:** Proposed

**Date:** 2026-04-16

**Related RFCs:**
- #5576 §4 — Documentation Standards: recommends removing `docs/i18n/`, move translations to Wiki, optionally add `zeroclaw docs --translate`
- #5653 §4.6 — Zero Compromise in Practice: structured logging with stable keys
- #5679 — Translated setup guides reference removed flags

## Problem

ZeroClaw's i18n has three compounding problems:

1. **The TOML hack only covers tool descriptions.** `i18n.rs` loads locale-specific TOML files from `tool_descriptions/` for agent prompts. Every CLI string (~60+ in cron, memory, onboarding) is hardcoded English. The Tauri desktop GUI is in the same state.

2. **In-repo translated docs drift silently.** `docs/i18n/` contains 169 manually-maintained markdown files across 31 locales with no CI check and no staleness tracking. Issue #5679 documents 30+ guides referencing removed `install.sh` flags. This is the expected steady state, not an anomaly.

3. **No path for non-English users after doc removal.** Removing translated docs without replacement leaves non-English users with no recourse.

## Proposal

### Phase 1 — Delete the TOML system

Remove `tool_descriptions/` (32 `.toml` files). Rewrite `crates/zeroclaw-runtime/src/i18n.rs` with a Fluent-based loader. English descriptions embedded at compile time via `include_str!()`; non-English locales loaded from disk with silent English fallback.

### Phase 2 — Fluent for all user-facing CLI and runtime strings

New dependency: `fluent = "0.16"` (sole addition).

Locale file layout:
```
locales/
  en/
    tools.ftl    # migrated from TOML (57 keys)
    main.ftl     # cron + memory + onboarding CLI strings (~60 keys)
    errors.ftl   # error messages
  ja/
    tools.ftl
    main.ftl
    ...
```

Callsite pattern:
```rust
// User sees translated message
println!("{}", t!("cron-no-tasks"));

// Logs stay English with a stable key — never translated
tracing::error!(error_key = "cron-no-tasks", "No scheduled tasks yet");
```

Acceptance criteria additions (per review feedback):
- `LANG=ja_JP.UTF-8 zeroclaw cron list` renders Japanese when `locales/ja/` is present (encoding suffix normalization)
- Before/after cold build times reported alongside binary size in the PR

### Phase 3 — Extend Fluent to Tauri

The Tauri frontend is an independent adapter per the hexagonal architecture (FND-001 §4.2). The correct shape is a parallel `fluent-bundle` JS implementation reading the same `.ftl` files — not a Rust bridge that would couple the UI adapter to the Rust runtime adapter. The `.ftl` files are the port; two adapters consuming them is the expected pattern.

CI enforcement requirements:
- The Tauri build bundles or locates `locales/` from the same source as the Rust runtime
- CI treats `fluent-bundle` JS missing-key warnings as errors so coverage gaps are caught at build time, not at runtime in a user's UI

### Phase 4 — Remove in-repo translated docs (per RFC #5576 §4)

Delete `docs/i18n/` (169 files, ~2.2 MB), all `README.*.md` at root (31 files), and non-English hub files. Tag a release before deletion so content is archived at that SHA. Add `TRANSLATIONS.md` stub directing to `zeroclaw docs --translate`. Remove i18n parity requirement from `docs/contributing/docs-contract.md`.

### Phase 5 — Move Matrix E2EE guide to Wiki

Rename `docs/security/matrix-e2ee-guide.md` to "Matrix channel setup guide" on the Wiki (English only). Leave a forwarding stub for one release cycle. This is the first instance of the RFC #5576 §5 setup-guide-to-wiki pattern.

### Phase 6 — `zeroclaw docs --translate` CLI command

Add a subcommand: `zeroclaw docs --translate <path> --locale <lang>`. Output options: stdout, file, pager.

Required safeguards (per review feedback):

1. **Path validation is a trust boundary.** `<path>` must resolve within the docs directory. `zeroclaw docs --translate ../../../etc/passwd` must be rejected with a clear error. Arbitrary filesystem content must not be readable through the provider API.

2. **Code spans must not be translated.** Inline CLI flags, config keys, TOML examples, and fenced code blocks are not natural language. The implementation must use a structured system prompt instructing the model to preserve all code spans and fenced blocks, or a pre/post-processing step that extracts and re-inserts them. This is an acceptance criterion.

3. **Provider unavailability.** Graceful, specific error when no provider is configured or the provider is unreachable. Per FND-006 §4.1: configuration errors fail fast with an actionable message; operational errors surface with context.

### Phase 7 — Policy

Add `Localization` section to `AGENTS.md`:
- User-facing output uses Fluent
- Logs stay English with stable `error_key` field (RFC #5653 §4.6)
- Panics, tracing lines, debug output are never translated
- Wiki is English only
- Translated docs live on the Wiki, not in-repo (RFC #5576 §4)

## Log key design principle

User-facing and machine-facing outputs have different stability requirements and are separate surfaces:

```rust
// User surface — translated, may change between releases
println!("{}", t!("cron-job-added", id = job.id));

// Machine surface — English, stable across releases, greppable
tracing::info!(error_key = "cron-job-added", id = %job.id, "Cron job added");
```

## Non-goals

- `clap` `--help` text — clap's i18n story is separate
- po4a or PO-based docs pipelines — rejected; on-demand translation is the replacement
- Date/time/number locale formatting
- Log lines, panic messages, debug output — never translated
- Wiki translations — Wiki is English only
- Broader docs-to-Wiki migration beyond the Matrix guide

## Alternatives considered

| Alternative | Reason for rejection |
|---|---|
| Extend TOML system to CLI strings | No pluralization, no interpolation, no ecosystem, no upgrade path |
| Keep in-repo docs with staleness CI | Makes breakage visible but doesn't fix maintenance burden |
| Community Wiki translations | Wiki is English only; on-demand LLM translation is more accurate and always current |

## ADRs

This RFC produces two ADRs:
- [ADR-005: Adopt Mozilla Fluent as project i18n system](../architecture/adr-005-adopt-mozilla-fluent.md)
- [ADR-006: Remove in-repo translated docs](../architecture/adr-006-remove-in-repo-translated-docs.md)
