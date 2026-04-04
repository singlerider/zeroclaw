# Overnight Research Progress

## Session start: 2026-04-04

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

### Issue updates — DONE
- zeroclaw-labs/zeroclaw#5272 issue body: corrected baseline to 19.80 MB
- zeroclaw-labs/zeroclaw#5272 measurement comment: consolidated ALL data (new gates, existing gates, .eh_frame, cargo-bloat, best savings table)
- zeroclaw-labs/zeroclaw#5272 methodology comment: updated with full test matrix and data file links
- zeroclaw-labs/zeroclaw#5272 matrix comparison comment: removed (data merged into measurement comment)
- singlerider/zeroclaw#5: updated with .eh_frame test results
- singlerider/zeroclaw#8: created for schemars optimization
- CSV: 26 rows with complete data for all measured configurations

### Research complete for this session
All TODO items completed or conclusively investigated. No further builds needed.

**Key actionable findings:**
1. `objcopy --remove-section=.eh_frame` → **-2.0 MB** (zero effort, add to build pipeline)
2. 4 feature gates implemented → **-1.31 MB** when disabled (already done on branch)
3. schemars optional → **~-457 KiB** estimated (singlerider/zeroclaw#8, medium effort)
4. Theoretical floor: **~16 MB** without removing always-compiled functionality

### Comprehensive channel feature-gating — IN PROGRESS
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

**What remains for full compilation:**
- Gateway handler functions need `#[cfg]` gates (handle_whatsapp_verify, handle_whatsapp_message, handle_linq_webhook, handle_wati_verify, handle_wati_webhook, handle_nextcloud_talk_webhook and helpers)
- Tests that reference gated channel types need gating
- This is mechanical work — the pattern is established, just needs applying to ~6 more functions

**Value of this approach:**
With all channels disabled (`--no-default-features --features "observability-prometheus,skill-creation"`), the binary would contain only the core agent loop, CLI, provider system, and memory — a pure minimalist agent with no channel integrations. Users can opt-in to exactly the channels they need. The `default` feature set preserves full current behavior.
