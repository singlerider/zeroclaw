# Binary Size & Compile Time: What We Found

## The binary

Default release binary: **22.2 MB**. With all optional features (matrix, nostr, whatsapp-web, etc.): **45.4 MB**.

The single heaviest dependency is `matrix-sdk` — enabling it alone adds **9.3 MB**. The next heaviest are `whatsapp-web` (+3.4 MB) and `channel-nostr` (+2.0 MB). All are already feature-gated.

## What actually helps

| What | Savings | Effort |
|---|---|---|
| Strip `.eh_frame` after build (`objcopy`) | **2.2 MB** | One-line post-build script |
| Disable 4 optional channel deps | **1.3 MB** | Done (feature flags) |
| Make schemars optional | **2.1 MB** | Done (schema-export feature) |
| Use mold/lld for dev builds | **43% faster** compile | `.cargo/config.toml` change |
| **Best achievable binary** | **18.8 MB** | All of the above combined |

## Workspace splitting

**8 workspace crates extracted: 186,047 LOC (63% of codebase).** Root crate went from ~289K to ~106K LOC.

| Crate | LOC | Content |
|---|---|---|
| zeroclaw-core | 38,126 | Security, SOP, hardware, observability, identity, etc. |
| zeroclaw-tools | 43,494 | 69 tool implementations |
| zeroclaw-channels | 38,397 | 34 channel implementations |
| zeroclaw-providers | 30,859 | All LLM providers, auth, multimodal |
| zeroclaw-config | 21,683 | Config schema, secrets, SecurityPolicy |
| zeroclaw-memory | 11,041 | SQLite, Qdrant, embeddings, consolidation |
| zeroclaw-infra | 1,847 | Session backends, debounce, watchdog |
| zeroclaw-types | 600 | Core traits and message types |

### The impact on compile times

**Dev profile incremental rebuilds — before vs after:**

| What changed | Before | After | Improvement |
|---|---|---|---|
| `src/main.rs` (root only) | 20.8s | **10.0s** | **-52%** |
| `src/agent/loop_.rs` (root only) | 27.9s | **10.1s** | **-64%** |
| `providers/anthropic.rs` | 23.0s | **16.5s** | -28% |
| `memory/sqlite.rs` | 21.2s | **16.0s** | -25% |
| `channels/discord.rs` | — | **15.0s** | new |
| `tools/browser.rs` | — | **13.0s** | new |
| `core/security` | — | **11.7s** | new |

The root crate rebuild floor dropped from ~20s to ~10s — **halved** by extracting 63% of the code.

**Release profile (fat LTO):**

| What changed | Before | After | Improvement |
|---|---|---|---|
| `src/main.rs` | 338.8s | **234.4s** | **-31%** |
| `src/agent/loop_.rs` | 366.2s | **231.2s** | **-37%** |
| `providers/anthropic.rs` | 375.0s | **287.0s** | -23% |
| `tools/browser.rs` | — | **247.5s** | new |

**CI profile (thin LTO):**

| What changed | Before | After | Improvement |
|---|---|---|---|
| `src/main.rs` | 188.3s | **105.6s** | **-44%** |
| `src/agent/loop_.rs` | 188.9s | **109.4s** | **-42%** |

### What still can't be extracted

- **Agent loop** (18K LOC) — the core orchestration, everything depends on it
- **Gateway** (10K LOC) — `AppState` struct has 30+ fields from 8+ subsystems
- **Channels dispatch** (11K LOC) — calls `run_tool_call_loop` with 26 parameters
- **21 "dirty" tools** (10K LOC) — reference cron, SOP, skills, agent internals

These require architectural refactoring (dependency injection, callback-based dispatch) to extract further.

## What doesn't help

**Linker choice doesn't matter for release builds.** All three linkers (gnu-ld, mold, lld) produce identical release binaries. But for **dev builds**, mold and lld are 43% faster (120-123s vs 210s).

**Regex can't be removed.** 100 usage sites across 15 files. The 515 KB cost is unavoidable.

**`reqwest` blocking/socks features can't be gated.** Blocking is needed for the onboarding wizard. SOCKS is needed for proxy users.

## Recommendations

1. **Add `objcopy --remove-section=.eh_frame` to the release pipeline.** Free 2.2 MB, zero risk.

2. **Use mold or lld for dev builds.** Add to `.cargo/config.toml`:
   ```toml
   [target.x86_64-unknown-linux-gnu]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

3. **The workspace split delivers real value.** Dev incremental rebuilds are 25-64% faster depending on which file changed. Root-only changes (main.rs, agent loop) went from 20-28s to 10s. This compounds over every build a developer does.

4. **The feature gate system works.** All 28 channel features compile independently. Users who only need a subset can build a smaller, faster binary.

## What's not in the pipeline yet

- The `objcopy` strip step (recommendation #1)
- The mold/lld dev config (recommendation #2)
- StageX container builds (blocked — shipping Rust 1.82, we need 1.87+)
