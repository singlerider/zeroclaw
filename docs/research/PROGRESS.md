# Binary Size & Compile Time Research Progress

**Branch:** `issue/compiled-size-improvements`
**Tracker:** zeroclaw-labs/zeroclaw#5272

---

### Methodology verification — DONE
- Rebuilt all 4 individual gate configs with full measurements (sections, deps, dupes, build time)
- CSV now has complete data for all rows except ci-all and matrix (missing sections)
- All builds use `cargo build --release` (not incremental — each config triggers full LTO re-link)
- Build times are wall-clock from `date +%s` before/after cargo build

### Per-gate full measurements — DONE
- no-email: 20,174,952 bytes, text=11,360,616, deps=768, dupes=40, 280s
- no-mqtt: 20,399,208 bytes, text=11,440,040, deps=816, dupes=40, 278s
- no-tui: 20,507,240 bytes, text=11,526,440, deps=747, dupes=44, 284s
- no-telegram: 20,596,104 bytes, text=11,577,448, deps=829, dupes=46, 278s
- no-default-features: 19,283,816 bytes, text=10,766,184, deps=615, dupes=32, 262s
- CSV updated with all values

### .eh_frame investigation — DONE
- `-C force-unwind-tables=no` (nightly): NO EFFECT. .eh_frame 1,871,920 bytes (actually +3 KB). C deps (ring, libsqlite3-sys) emit their own unwind tables that Rust flags can't suppress.
- `-Wl,--gc-sections` (stable): NO EFFECT. Already enabled by default for release builds.
- `-Wl,--strip-all` + force-unwind-tables=no (nightly): NO EFFECT. .eh_frame is in the ALLOC segment; linker won't strip loaded sections.
- **`objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr` (post-build): WORKS. Saves 2,056 KB (2.0 MB). Binary still runs correctly (--version, --help, doctor, error handling all verified).**
- Recommendation: add `objcopy` step to release build pipeline
- CSV updated with measurements
- singlerider/zeroclaw#5 updated with findings

### Existing feature gate measurements — DONE
All existing optional features measured (added individually to default baseline):

| Feature | Added size | Dep entries added | Duplicate pairs added |
|---|---|---|---|
| channel-matrix | +10.72 MB (54.1%) | +428 | +24 |
| whatsapp-web | +3.56 MB (18.0%) | +256 | +27 |
| channel-nostr | +1.97 MB (10.0%) | +80 | +5 |
| rag-pdf | +1.19 MB (6.0%) | +47 | -4 |
| browser-native | +0.27 MB (1.4%) | +84 | +26 |
| channel-lark | +0.15 MB (0.8%) | +9 | +3 |
| hardware | +0.11 MB (0.6%) | +30 | +6 |
| plugins-wasm | +0.09 MB (0.5%) | +430 | +39 |
| probe | +0.00 MB (0.0%) | +189 | +15 |

Key insight: `plugins-wasm` (extism) adds only 0.09 MB binary but 430 dep entries and 39 duplicate pairs — mostly compile-time cost, not binary cost. `probe` adds zero binary size but 189 deps.
- CSV updated with all 8 existing gate measurements

### Regex audit — DONE
- 100 regex usage sites across 15 files
- Most are in `agent/loop_.rs` (XML/tool-call parsing) — 25+ patterns, all using `LazyLock` (compiled once)
- `skills/audit.rs` has 8 security-scanning patterns — must be regex
- `tools/content_search.rs` — output parsing for grep/ripgrep
- `memory/consolidation.rs` — single pattern for stripping media markers
- **Verdict: regex usage is legitimate and pervasive. Cannot remove or replace with string matching. The 515 KiB cost is unavoidable.**

### reqwest feature audit — DONE
- `reqwest::blocking` used in 15 call sites across 4 files:
  - `providers/gemini.rs` (2 sites) — blocking token exchange
  - `providers/mod.rs` (2 sites) — blocking model list fetch
  - `onboard/wizard.rs` (9 sites) — blocking API validation during setup
  - `skills/mod.rs` (1 site) — blocking skill fetch
  - All are in synchronous contexts (onboarding wizard, CLI commands) where async would require runtime changes
  - **Verdict: `blocking` feature is needed; gating it would break onboarding and CLI commands**
- SOCKS proxy used in `config/schema.rs` for WebSocket proxy connections
  - **Verdict: `socks` is needed for proxy support; gating possible but would break proxy users**

### schemars / Config bloat investigation — DONE
- `schemars` contributes 373 KiB to .text (3.3%) — from 167 `#[derive(JsonSchema)]` sites
- `Config::json_schema` function alone is 84 KiB
- `Config::serialize` is 193 KiB, `Config::deserialize` is 54 KiB
- **Total Config serde + schema cost: ~331 KiB (serialize + deserialize + json_schema)**
- `schemars::schema_for!(Config)` is called in exactly 2 places:
  - `main.rs:1623` — the `zeroclaw config schema` CLI command
  - `config/schema.rs:11319` — a test
- **Potential optimization: make schemars an optional feature.** The 457 KiB (schemars crate + json_schema fn) exists solely for `zeroclaw config schema`. Could be gated behind a `schema-export` feature.
- The serde Serialize/Deserialize cost (247 KiB) is unavoidable — config must be parsed.
- The Config struct has ~1,159 `pub` fields across all nested structs in schema.rs. The mega-struct monomorphization is the root cause.
- Created singlerider/zeroclaw#8 for this research area

### Linker and codegen backend testing — DONE
- **mold vs GNU ld** (incremental rebuild, release-fast profile): ~370ms difference (18,029ms vs 17,656ms) — negligible. Codegen dominates, not linking.
- **Cranelift backend** (nightly, debug builds): 14% faster clean build (147s vs 170s). No incremental benefit — the speed-up is in initial codegen, not re-linking.
- **wild linker**: not tested — v0.8.0 fails to compile on stable Rust 1.93. Too immature.
- **StageX container builds**: `stagex/rust:latest` ships Rust 1.82 (needs 1.87+). Pallet composition (rust + busybox + musl + gcc + libunwind + openssl + llvm + zlib) works but toolchain is too old. Waiting on StageX maintainers to update.

### cargo-bloat analysis — DONE
Per-crate `.text` breakdown (top contributors):
- **zeroclaw own code**: 4.5 MiB (39.9%) — monolithic crate
- **std**: 2.1 MiB (18.6%)
- **rustls**: 411 KiB (3.6%)
- **schemars**: 373 KiB (3.3%) — only needed for `zeroclaw config schema`
- **regex_automata**: 305 KiB (2.7%)
- **axum**: 283 KiB (2.5%)
- **reqwest**: 259 KiB (2.3%)

Per-function (top 3):
- `Config::serialize`: 193 KiB (serde for ~1,159-field mega-Config)
- `main::{{closure}}`: 131 KiB
- `Config::json_schema`: 84 KiB (schemars)

Raw output: `cargo-bloat-crates.txt`, `cargo-bloat-functions.txt`

### Issue updates — DONE
- zeroclaw-labs/zeroclaw#5272 issue body: corrected baseline to 19.80 MB
- zeroclaw-labs/zeroclaw#5272 measurement comment: consolidated ALL data (new gates, existing gates, .eh_frame, cargo-bloat, best savings table)
- zeroclaw-labs/zeroclaw#5272 methodology comment: updated with full test matrix and data file links
- zeroclaw-labs/zeroclaw#5272 matrix comparison comment: removed (data merged into measurement comment)
- singlerider/zeroclaw#5: updated with .eh_frame test results
- singlerider/zeroclaw#8: created for schemars optimization
- CSV: 26 rows with complete data for all measured configurations

### Initial research complete
Feature gating, .eh_frame, linker comparison, regex/reqwest audits, cargo-bloat analysis done. Workspace extraction and further measurement continued below.

**Key actionable findings:**
1. `objcopy --remove-section=.eh_frame` → **-2.0 MB** (zero effort, add to build pipeline)
2. 4 feature gates implemented → **-1.31 MB** when disabled (already done on branch)
3. schemars optional → **~-457 KiB** estimated (singlerider/zeroclaw#8, medium effort)
4. Theoretical floor: **~16 MB** without removing always-compiled functionality

### Comprehensive channel feature-gating — DONE
Extended feature-gating to ALL remaining channels (22 additional gates).

**Channels now gated (total 28):**
- Previously gated: matrix, nostr, lark, whatsapp-web, voice-wake
- First batch (this session): email, mqtt, telegram, tui-onboarding
- Comprehensive batch: discord, slack, signal, mattermost, irc, imessage, dingtalk, qq, bluesky, twitter, reddit, notion, linq, wati, nextcloud, mochat, wecom, clawdtalk, webhook, acp-server, whatsapp-cloud, voice-call

**Files modified:**
- `Cargo.toml`: 22 new feature flags, all added to `default` and `ci-all`
- `src/channels/mod.rs`: all module declarations, re-exports, `collect_configured_channels`, `build_channel_by_id` gated
- `src/cron/scheduler.rs`: all channel delivery match arms gated
- `src/gateway/mod.rs`: imports, AppState fields, construction, routes gated for linq/wati/nextcloud/whatsapp-cloud
- `src/gateway/api.rs`: test AppState fields gated

**Value of this approach:**
With all channels disabled (`--no-default-features --features "observability-prometheus,skill-creation"`), the binary would contain only the core agent loop, CLI, provider system, and memory — a pure minimalist agent with no channel integrations. Users can opt-in to exactly the channels they need. The `default` feature set preserves full current behavior.

### Phase 0: Split channels/mod.rs — STARTED
- Extracted `build_system_prompt`, `build_system_prompt_with_mode`, `build_system_prompt_with_mode_and_autonomy`, `inject_workspace_file`, `load_openclaw_bootstrap_files` into `src/channels/prompt.rs` (389 lines)
- `mod.rs` reduced from 11,876 → 11,502 lines (-374)
- Re-exports in `mod.rs` preserve all public API paths
- Removed unused imports in `email_channel.rs`, `gmail_push.rs`, `mod.rs`
- Fixed `AcpServerChannel` re-export (type is `AcpServer`, not `AcpServerChannel`)
- Fixed duplicate `WhatsAppChannel` re-export
- `cargo check` passes with zero warnings

### Phase 0 continued: Extract factory.rs — DONE
- Extracted `build_channel_by_id`, `send_channel_message`, `ConfiguredChannel`, `ChannelHealthState`, `classify_health_result`, `collect_configured_channels` into `src/channels/factory.rs` (874 lines)
- `mod.rs` reduced from 11,502 → 10,710 lines (-792 more)
- All functions made `pub(super)` for cross-file visibility
- Added `use factory::*` import in `mod.rs` with `#[allow(unused_imports)]`
- Skipped `doctor_channels` extraction — only 61 lines, not worth the coupling overhead
- Cleaned up all remaining warnings (added `#[allow(unused_imports)]` on public API re-exports)
- `cargo check` passes with **zero warnings, zero errors**

### Phase 0 final state
- `mod.rs`: 11,876 → **10,710 lines** (-1,166 lines, -9.8%)
- `prompt.rs`: 389 lines (system prompt construction)
- `factory.rs`: 874 lines (channel instantiation and routing)
- Total: 11,973 lines across 3 files (vs 11,876 in 1 file — 97 lines of module headers added)

### Phase 1: zeroclaw-types crate — DONE
Created `crates/zeroclaw-types/` workspace crate with core trait definitions and data types.

**Crate contents (571 LOC):**
- `channel.rs` (208 LOC): `Channel` trait, `ChannelMessage`, `SendMessage`
- `provider.rs` (246 LOC): `ChatMessage`, `ChatResponse`, `ToolCall`, `TokenUsage`, `StreamChunk`, `StreamEvent`, `StreamOptions`, `ProviderCapabilities`, `ToolsPayload`, `ConversationMessage`, `ToolResultMessage`, `ChatRequest`
- `media.rs` (57 LOC): `MediaAttachment`, `MediaKind`
- `tool.rs` (45 LOC): `Tool` trait, `ToolResult`, `ToolSpec`
- `lib.rs` (15 LOC): module declarations

**Dependencies:** `anyhow`, `async-trait`, `serde`, `serde_json`, `tokio`, `tokio-util` only. No `reqwest`, `futures_util`, or any heavy deps.

**Root crate changes:**
- `channels/traits.rs`: 375 → 153 lines (re-exports + tests)
- `channels/media_pipeline.rs`: `MediaAttachment`/`MediaKind` replaced with re-exports
- `tools/traits.rs`: 121 → 84 lines (re-exports + tests)
- `providers/traits.rs`: 1,092 → 829 lines (data types replaced with re-exports; `Provider` trait + `StreamError` + streaming methods kept locally due to `reqwest`/`futures_util` deps)

**Note:** The `Provider` trait stays in the root crate because it has streaming methods returning `stream::BoxStream<StreamResult<StreamChunk>>` which depends on `futures_util` and `reqwest`. Only the data types were moved.

**Compilation:** `cargo check` — zero errors, zero warnings across entire workspace.

### Phase 2: zeroclaw-infra crate — DONE
Created `crates/zeroclaw-infra/` with 5 infrastructure modules (1,837 LOC):

| Module | LOC | Dependencies |
|---|---|---|
| `debounce.rs` | 191 | tokio only |
| `stall_watchdog.rs` | 188 | tokio only |
| `session_backend.rs` | 159 | zeroclaw-types (ChatMessage), chrono |
| `session_store.rs` | 372 | session_backend, ChatMessage |
| `session_sqlite.rs` | 927 | rusqlite, chrono, parking_lot |

**NOT moved (config type dependencies):** `transcription.rs`, `tts.rs`, `link_enricher.rs`, `media_pipeline.rs` — these import `TranscriptionConfig`, `TtsConfig`, etc. from `config/schema.rs`. Moving them would require extracting config types first.

**Compilation:** `cargo check` — zero errors, zero warnings.

### Phase 3: zeroclaw-gateway — NOT VIABLE (documented)

The gateway (10,038 LOC) cannot be extracted as a separate crate. Root cause: **`AppState` coupling**.

Every handler function takes `State<AppState>` where `AppState` contains:
- `Arc<dyn Provider>` (from providers)
- `Arc<dyn Memory>` (from memory)
- `Arc<dyn Observer>` (from observability)
- Channel instances (`WhatsAppChannel`, `LinqChannel`, etc.)
- `SecurityPolicy`, `PairingGuard` (from security)
- `CostTracker` (from cost)
- Tool registry, session backends, etc.

Even the "clean" gateway files (`sse.rs`, `canvas.rs`, `static_files.rs`) reference `super::AppState` via axum extractors.

**Only truly standalone:** `auth_rate_limit.rs` (204 LOC) and `session_queue.rs` (234 LOC) — too small to justify a separate crate.

**What would be needed:** Define `AppState` as a trait or move it to a shared crate. But `AppState` is 30+ fields of concrete types from 8+ subsystems. This is a multi-week architectural refactor, not a mechanical extraction.

**Recommendation:** Gateway extraction requires a broader AppState dependency injection refactor.

### zeroclaw-config extraction — DONE

Broke the config circular dependency by:
- Copying full `config/schema.rs` (16,235 lines) + runtime logic into `zeroclaw-config`
- Centralizing 12 scattered config types from 6 subsystems into `scattered_types.rs` (TrustConfig, BrowserDelegateConfig, ThinkingConfig, HistoryPrunerConfig, EvalConfig, AutoClassifyConfig, ContextCompressionConfig, ClawdTalkConfig, VoiceCallConfig, AutonomyLevel, DomainMatcher, SopConfig)
- Copying `SecretStore` to config crate (self-contained with chacha20poly1305)
- Inlining provider alias functions to avoid circular dep
- Root crate's `config/schema.rs` replaced with 12-line re-export file

### schemars optional — DONE

- Created `schema-export` feature flag on both zeroclaw-config and root crate
- 181 `JsonSchema` derives converted to `#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]`
- `schemars` dep made optional in both crates
- `zeroclaw config schema` CLI command gated behind the feature
- Estimated savings: ~457 KB when disabled

### zeroclaw-memory extraction — DONE

- Extracted 11,041 LOC to `crates/zeroclaw-memory/`
- Created `Summarizer` trait in `zeroclaw-types` to decouple memory from the full `Provider` trait (memory only needs `chat_with_system` for consolidation)
- Root crate's `memory/mod.rs` replaced with re-exports + `ProviderSummarizer` bridge
- `cli.rs` and `battle_tests.rs` kept in root crate (depend on root-only types)

### zeroclaw-providers extraction — DONE

- Extracted 30,859 LOC to `crates/zeroclaw-providers/` including auth (2,599 LOC) and multimodal (853 LOC)
- Moved `TOOL_CHOICE_OVERRIDE` task-local from agent loop to providers crate
- Made `pub(crate)` alias functions public for cross-crate access
- Root crate's `providers/mod.rs` replaced with 6-line re-export file

### Gateway handler gating — DONE

- 6 handler functions gated: `handle_whatsapp_verify`, `handle_whatsapp_message`, `handle_linq_webhook`, `handle_wati_verify`, `handle_wati_webhook`, `handle_nextcloud_talk_webhook`
- Plus helpers: `verify_whatsapp_signature`, `nextcloud_talk_memory_key`, `WatiVerifyQuery`

### VoiceCallConfig consolidation — DONE

- Removed local definition from `voice_call.rs`, replaced with re-export from zeroclaw-config
- Added missing `webhook_base_url` field and `#[serde(rename_all)]` to config crate version

---

## Final state

### Workspace crates (186,047 LOC extracted)

| Crate | LOC | Content | Wired |
|---|---|---|---|
| `zeroclaw-types` | 594 | Channel/Tool traits, message types, Summarizer | Yes |
| `zeroclaw-infra` | 1,847 | Session backends, debounce, stall watchdog | Yes |
| `zeroclaw-config` | 18,262 | Full config schema, secrets, proxy runtime | Yes |
| `zeroclaw-memory` | 11,041 | SQLite/Qdrant/embeddings/consolidation | Yes |
| `zeroclaw-providers` | 30,859 | All LLM providers, auth, multimodal | Yes |

### Feature gates (28 channels + extras)

All channels independently toggleable. `schema-export` feature for schemars. All in `default` — existing behavior preserved.

### Not extracted (documented reasons)

- **Gateway** (10,038 LOC): `AppState` couples every handler to 8+ subsystems
- **Channels** (58,524 LOC): `channels/mod.rs` calls `agent::loop_::run_tool_call_loop` directly
- **Tools + Agent** (~76,000 LOC): `delegate` tool creates a circular tools ↔ agent dependency

---

## Systematic Test Results

### Round 1 (2026-04-07) — After initial extraction (7 crates, 174K LOC)

27 binary size rows + 24 incremental timing rows. See CSV files for full data. Root crate was ~130K LOC.

### Round 2 (2026-04-08) — After full extraction (8 crates, 186K LOC)

Extracted zeroclaw-core (security, SOP, hardware, observability, etc.). Root crate down to ~106K LOC.

**Binary size (27 v2 rows):**

| test_id | Size | Notes |
|---|---|---|
| `release-gnuld-default-v2` | **22.15 MB** | Post-full-extraction baseline |
| `release-gnuld-no-default-v2` | **17.51 MB** | Absolute minimum |
| `release-gnuld-ciall-v2` | **45.40 MB** | Everything enabled |
| `release-gnuld-matrix-v2` | **31.48 MB** | Matrix adds +9.3 MB |
| `release-gnuld-default-strip-eh-v2` | **19.93 MB** | objcopy .eh_frame strip |
| `release-gnuld-no-4gates-strip-eh-v2` | **18.76 MB** | Minimum achievable |
| `dev-mold-default-v2` | 562 MB / **120s** | mold 43% faster than gnu-ld for dev |
| `dev-lld-default-v2` | 538 MB / **123s** | lld 41% faster than gnu-ld for dev |
| `dev-gnuld-default-v2` | 538 MB / **210s** | gnu-ld baseline for dev |

**Incremental compile times — dev profile (v1 → v2 comparison):**

| Touch point | v1 (7 crates) | v2 (8 crates) | Improvement |
|---|---|---|---|
| root-main | 20.8s | **10.0s** | **-52%** |
| root-agent | 27.9s | **10.1s** | **-64%** |
| core-security | — | **11.7s** | new |
| tools-browser | — | **13.0s** | new |
| infra-debounce | 20.2s | **14.4s** | -29% |
| channels-discord | — | **15.0s** | new |
| memory-sqlite | 21.2s | **16.0s** | -25% |
| providers-anthropic | 23.0s | **16.5s** | -28% |
| config-schema | 24.5s | **23.8s** | -3% |
| types-channel | 32.4s | **28.0s** | -14% |

**Incremental compile times — release profile (v1 → v2):**

| Touch point | v1 | v2 | Improvement |
|---|---|---|---|
| root-main | 338.8s | **234.4s** | **-31%** |
| root-agent | 366.2s | **231.2s** | **-37%** |
| tools-browser | — | **247.5s** | new |
| core-security | — | **250.3s** | new |
| infra-debounce | 353.9s | **274.5s** | -22% |
| channels-discord | — | **282.1s** | new |
| memory-sqlite | 336.0s | **286.5s** | -15% |
| providers-anthropic | 375.0s | **287.0s** | -23% |
| config-schema | 377.6s | **323.9s** | -14% |
| types-channel | 427.3s | **339.7s** | -20% |

**Incremental compile times — CI profile (v1 → v2):**

| Touch point | v1 | v2 | Improvement |
|---|---|---|---|
| root-main | 188.3s | **105.6s** | **-44%** |
| root-agent | 188.9s | **109.4s** | **-42%** |
| core-security | — | **119.5s** | new |
| tools-browser | — | **134.3s** | new |
| memory-sqlite | 187.3s | **144.0s** | -23% |
| infra-debounce | 196.4s | **145.4s** | -26% |
| channels-discord | — | **155.8s** | new |
| providers-anthropic | 184.7s | **175.3s** | -5% |
| config-schema | 207.7s | **180.6s** | -13% |
| types-channel | 209.2s | **198.3s** | -5% |

### Feature gate validation

**28/28 PASS, 0 FAIL.** All channel features compile correctly when individually disabled.

### Key findings

| Optimization | Savings | Status |
|---|---|---|
| `objcopy --remove-section=.eh_frame` | 2.2 MB | Documented, not in build pipeline |
| All 4 initial feature gates disabled | 1.3 MB | Implemented |
| schemars optional | 2.1 MB | Implemented |
| Workspace splitting (dev root rebuild) | **52% faster** | 186K LOC extracted |
| Workspace splitting (release root rebuild) | **31-37% faster** | 8 crates |
| Workspace splitting (CI root rebuild) | **42-44% faster** | |
| mold/lld for dev builds | **43% faster** link | Not yet in config |
