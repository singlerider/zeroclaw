# Implementation Plan: Workspace Separation & Build Optimization

This document is an executable plan for splitting the ZeroClaw monolithic crate into workspace crates, feature-gating all channels, and applying build optimizations. It is written as input for an AI coding assistant operating on a future `upstream/master`.

The research branch `issue/compiled-size-improvements` on `singlerider/zeroclaw` contains a working reference implementation. The upstream tracker is zeroclaw-labs/zeroclaw#5272.

---

## Prerequisites

- Rust 1.87+ (edition 2024)
- `mold` linker installed (`sudo pacman -S mold` or equivalent)
- `objcopy` available (part of binutils)

## Step 1: Create workspace crates

Create these directories and `Cargo.toml` files. Each crate's dependencies are listed. Do not add code yet.

```
crates/
  zeroclaw-types/      # anyhow, async-trait, serde, serde_json, tokio, tokio-util
  zeroclaw-infra/      # zeroclaw-types, anyhow, chrono, parking_lot, rusqlite, serde, tokio, tracing
  zeroclaw-config/     # zeroclaw-types, anyhow, chacha20poly1305, chrono, chrono-tz, cron, directories, hostname, parking_lot, rand, regex, reqwest, rustls, schemars(optional), serde, shellexpand, thiserror, tokio, toml, tracing, url, uuid, webpki-roots
  zeroclaw-memory/     # zeroclaw-types, zeroclaw-config, anyhow, async-trait, chrono, parking_lot, regex, reqwest, rusqlite, serde, sha2, tokio, tracing, uuid
  zeroclaw-providers/  # zeroclaw-types, zeroclaw-config, anyhow, async-trait, base64, chrono, futures-util, hmac, parking_lot, rand, regex, reqwest, ring, serde, sha2, thiserror, tokio, tokio-stream, tracing, uuid
  zeroclaw-channels/   # zeroclaw-types, zeroclaw-infra, zeroclaw-config, zeroclaw-memory, zeroclaw-providers, plus channel-specific deps
  zeroclaw-tools/      # zeroclaw-types, zeroclaw-infra, zeroclaw-config, zeroclaw-memory, zeroclaw-providers, zeroclaw-channels, plus tool-specific deps
  zeroclaw-core/       # all of the above, plus console, cron, dialoguer, glob, indicatif, ratatui(optional), crossterm(optional), prometheus(optional), opentelemetry(optional), etc.
```

Add all crates to `[workspace] members` in the root `Cargo.toml`. Add each as a `[dependencies]` entry in the root crate.

## Step 2: Extract types (zeroclaw-types)

Move these type definitions from the root crate into `zeroclaw-types`:

| Source file | Types to move |
|---|---|
| `src/channels/traits.rs` | `Channel` trait, `ChannelMessage`, `SendMessage` |
| `src/channels/media_pipeline.rs` | `MediaAttachment`, `MediaKind` (struct + impl only, not `MediaPipeline`) |
| `src/tools/traits.rs` | `Tool` trait, `ToolResult`, `ToolSpec` |
| `src/providers/traits.rs` | `ChatMessage`, `ChatResponse`, `ToolCall`, `TokenUsage`, `StreamChunk`, `StreamEvent`, `StreamOptions`, `ProviderCapabilities`, `ToolsPayload`, `ConversationMessage`, `ToolResultMessage`, `ChatRequest` |

Also add:
- A `Summarizer` trait (single method: `async fn summarize(&self, system_prompt: Option<&str>, text: &str, model: &str, temperature: f64) -> Result<String>`) — this decouples memory consolidation from the full `Provider` trait.
- A `tokio::task_local! { pub static TOOL_LOOP_THREAD_ID: Option<String>; }` — this decouples SecurityPolicy from the agent loop.

Replace original files with `pub use zeroclaw_types::*;` re-exports. Keep tests in the original files.

**Do NOT move:** The `Provider` trait itself (it has streaming methods that depend on `reqwest` and `futures_util`). Keep it in the root crate. Only the data types move.

## Step 3: Extract infrastructure (zeroclaw-infra)

Copy these files from `src/channels/` into `zeroclaw-infra`:
- `debounce.rs`
- `stall_watchdog.rs`
- `session_backend.rs`
- `session_store.rs`
- `session_sqlite.rs`

Fix imports: `crate::providers::traits::ChatMessage` → `zeroclaw_types::provider::ChatMessage`. Replace originals with re-export stubs.

## Step 4: Extract config (zeroclaw-config)

This is the hardest step. The config schema (`src/config/schema.rs`, ~16K LOC) is the dependency hub — every module imports from it.

1. Copy `src/config/schema.rs` wholesale into the config crate. It contains both type definitions and runtime logic (proxy builders, `Config::load_or_init`, `SecretStore` integration). This is intentional — the config crate is self-contained.

2. Copy `src/security/secrets.rs` (SecretStore) into the config crate. It only depends on `chacha20poly1305`.

3. Move scattered config types from their home modules into the config crate:
   - `TrustConfig` from `src/trust/types.rs`
   - `BrowserDelegateConfig` from `src/tools/browser_delegate.rs`
   - `ThinkingConfig`, `ThinkingLevel` from `src/agent/thinking.rs`
   - `HistoryPrunerConfig` from `src/agent/history_pruner.rs`
   - `EvalConfig`, `AutoClassifyConfig` from `src/agent/eval.rs`
   - `ContextCompressionConfig` from `src/agent/context_compressor.rs`
   - `ClawdTalkConfig` from `src/channels/clawdtalk.rs`
   - `VoiceCallConfig`, `VoiceProvider` from `src/channels/voice_call.rs`
   - `AutonomyLevel` from `src/security/policy.rs`
   - `DomainMatcher` from `src/security/domain_matcher.rs`
   - `SopConfig` from `src/sop/types.rs`
   - `EmailConfig` from `src/channels/email_channel.rs`
   - `GmailPushConfig` from `src/channels/gmail_push.rs`

4. Inline provider alias functions (`is_glm_alias`, `is_zai_alias`, etc.) into the config crate to avoid a circular dependency with providers.

5. Move `SecurityPolicy` from `src/security/policy.rs` into the config crate. Change its `TOOL_LOOP_THREAD_ID` reference from `crate::agent::loop_::TOOL_LOOP_THREAD_ID` to `zeroclaw_types::TOOL_LOOP_THREAD_ID`.

6. Replace root `src/config/schema.rs` with re-exports from zeroclaw-config.

7. Replace original scattered type definitions with re-exports.

## Step 5: Extract memory (zeroclaw-memory)

Copy all files from `src/memory/` except `cli.rs` and `battle_tests.rs` (these depend on root crate CLI types).

Fix imports: `crate::config::*` → `zeroclaw_config::schema::*`, `crate::providers::traits::Provider` → `zeroclaw_types::summarizer::Summarizer`.

In the root crate's `src/memory/mod.rs`, add a `ProviderSummarizer` wrapper struct that adapts `&dyn Provider` to the `Summarizer` trait by delegating to `chat_with_system`. Update the two call sites (`channels/mod.rs` and `gateway/ws.rs`) that call `consolidate_turn` to wrap the provider.

## Step 6: Extract providers (zeroclaw-providers)

Copy all files from `src/providers/`, plus `src/auth/` and `src/multimodal.rs`.

Move the `TOOL_CHOICE_OVERRIDE` task-local from `src/agent/loop_.rs` to the providers crate's `lib.rs`. Update the agent loop to re-export it.

Make `pub(crate)` functions public where needed (provider alias functions, `is_context_window_exceeded`).

Replace root `src/providers/mod.rs` with `pub use zeroclaw_providers::*;`. Delete root's `src/auth/` and `src/multimodal.rs`, replace with re-export stubs.

## Step 7: Extract channels (zeroclaw-channels)

Copy all channel implementation files from `src/channels/` **except**:
- `mod.rs` (dispatch runtime — calls `run_tool_call_loop`, stays in root)
- `factory.rs` (channel construction — stays in root)
- `prompt.rs` (depends on `identity` and `skills` — stays in root)
- `acp_server.rs` (depends on `agent::Agent` — stays in root)
- `telegram.rs` (depends on `security::pairing::PairingGuard` — stays in root)
- `matrix.rs` (depends on `security::redact` — stays in root, or inline `redact`)
- `mqtt.rs` (depends on SOP engine — stays in root)

Add a `util.rs` to the channels crate with: `redact()`, `truncate_with_ellipsis()`, `strip_tool_call_tags()`, `BLOCK_KIT_PREFIX`, `MaybeSet<T>`, `is_serial_path_allowed()`.

Replace extracted channel files in root with `pub use zeroclaw_channels::<module>::*;` stubs.

## Step 8: Extract tools (zeroclaw-tools)

Copy tool files that have zero dependencies on `crate::agent`, `crate::cron`, `crate::gateway`, `crate::runtime`, `crate::skills`, `crate::sop`, `crate::verifiable_intent`, or `crate::hooks`. This is ~70 of ~90 tool files.

Tools that reference `Channel`/`SendMessage`/`SessionBackend` are fine — those are in `zeroclaw-types` and `zeroclaw-infra`.

Keep in root: `delegate.rs` (calls `run_tool_call_loop`), `model_switch.rs` (reads agent task-local), `file_read.rs` (test-only agent refs — keep tests in root), all `cron_*` tools, all `sop_*` tools, `skill_tool.rs`, `skill_http.rs`, `read_skill.rs`, `schedule.rs`, `shell.rs`, `node_tool.rs`, `verifiable_intent.rs`, `security_ops.rs`, `workspace_tool.rs`.

Replace extracted files with re-export stubs.

## Step 9: Extract core (zeroclaw-core)

Copy remaining modules with zero agent/channel dispatch dependencies:
- `security/` (all files except `policy.rs` which is in config, and `workspace_boundary.rs` which depends on `config/workspace.rs`)
- `trust/`
- `observability/`
- `identity/`
- `tunnel/`
- `heartbeat/`
- `routines/`
- `runtime/`
- `verifiable_intent/`
- `health/`
- `hooks/`
- `plugins/`
- `skillforge/`
- `hands/`
- `rag/`
- `cost/`
- `tui/` (behind `tui-onboarding` feature)
- `sop/` (remove CLI `handle_command` function from the copy — keep in root)
- `hardware/` (remove CLI handler, remove `uf2.rs`/`pico_flash.rs`/`pico_code.rs` which use `include_bytes!` with relative paths to firmware)
- `peripherals/` (remove CLI handler)
- `service/` (remove CLI handler)
- `integrations/` (remove CLI handler)

For modules with CLI handlers: the root keeps `mod.rs` with `pub use zeroclaw_core::<module>::*;` at the top followed by the `handle_command` function. Delete submodule files from root.

Add feature pass-throughs in root `Cargo.toml`: `hardware`, `tui-onboarding`, `observability-prometheus`, `observability-otel`, `peripheral-rpi` → corresponding features on `zeroclaw-core`.

Make `pub(crate)` items `pub` where needed for cross-crate visibility (especially test helpers like `set_sops_for_test` which must not be behind `#[cfg(test)]` in the defining crate).

## Step 10: Feature-gate all channels

Add a feature flag for every channel in root `Cargo.toml`. All go in `default` and `ci-all`.

```toml
channel-email = ["dep:lettre", "dep:mail-parser", "dep:async-imap", "zeroclaw-channels/channel-email"]
channel-mqtt = ["dep:rumqttc"]
channel-telegram = ["dep:image", "zeroclaw-channels/channel-telegram"]
channel-discord = []
channel-slack = []
# ... (28 total)
```

Gate each channel's `pub mod`, `pub use`, `collect_configured_channels` block, `build_channel_by_id` match arm, and cron scheduler delivery arm with `#[cfg(feature = "channel-X")]`. Add warn fallbacks for the `#[cfg(not(...))]` case.

Gate gateway webhook handlers (`handle_whatsapp_verify`, `handle_linq_webhook`, etc.) and their AppState fields.

## Step 11: Make schemars optional

Add `schema-export` feature to both `zeroclaw-config` and root crate. Convert all `#[derive(JsonSchema)]` to `#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]`. Gate the `zeroclaw config schema` CLI command.

## Step 12: Build optimizations

Add to `.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

Add to the release build script (CI, Makefile, or Dockerfile):
```bash
objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr target/release/zeroclaw
```

## Step 13: Verify

```bash
cargo check                              # default features
cargo check --no-default-features        # minimum
cargo check --features ci-all            # everything
cargo test                               # default features
cargo test --features ci-all             # all features
target/release/zeroclaw --version
target/release/zeroclaw doctor
```

All must pass with zero errors.
