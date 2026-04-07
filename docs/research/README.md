# Binary Size & Compile Time Research

**Branch:** `issue/compiled-size-improvements`
**Upstream tracker:** [zeroclaw-labs/zeroclaw#5272](https://github.com/zeroclaw-labs/zeroclaw/issues/5272)
**Sub-issues:** [singlerider/zeroclaw#1-8](https://github.com/singlerider/zeroclaw/issues)

## Summary

Research and implementation to reduce the ZeroClaw compiled binary footprint and improve incremental compilation times.

**Results:**
- 8 workspace crates extracted (186,047 LOC, 63% of codebase)
- Root crate: 289K → 106K LOC
- Dev incremental rebuild: 20.8s → 10.0s (-52%)
- Release incremental: 338.8s → 234.4s (-31%)
- CI incremental: 188.3s → 105.6s (-44%)
- 28 channel feature gates, schemars optional, .eh_frame strip
- 3,990 tests pass, 0 failures

## Files in this directory

| File | Description |
|---|---|
| [`FINDINGS.md`](FINDINGS.md) | Human-readable summary and recommendations |
| [`PROGRESS.md`](PROGRESS.md) | Detailed log of all work done, findings, and decisions |
| [`TODO.md`](TODO.md) | Task tracking with completion status and future work |
| [`binary-size-tracking.csv`](binary-size-tracking.csv) | 54 measurement rows (27 v1 + 27 v2) |
| [`binary-size-tracking-pre-extraction.csv`](binary-size-tracking-pre-extraction.csv) | Archived original 19 rows |
| [`binary-size-tracking-schema.md`](binary-size-tracking-schema.md) | CSV column definitions, measurement commands |
| [`incremental-compile-times.csv`](incremental-compile-times.csv) | 54 timing rows (24 v1 + 30 v2) |
| [`cargo-bloat-crates.txt`](cargo-bloat-crates.txt) | Per-crate .text section breakdown |
| [`cargo-bloat-functions.txt`](cargo-bloat-functions.txt) | Per-function .text breakdown |
| [`dependency-isolation.txt`](dependency-isolation.txt) | Per-crate dep trees and duplicate count |
| [`feature-gate-validation.txt`](feature-gate-validation.txt) | Pass/fail for 28 feature configurations |

## Workspace crates

| Crate | LOC | Content |
|---|---|---|
| `zeroclaw-core` | 38,126 | Security, SOP, observability, identity, trust, etc. |
| `zeroclaw-tools` | 43,494 | 69 tool implementations |
| `zeroclaw-channels` | 38,397 | 34 channel implementations |
| `zeroclaw-providers` | 30,859 | All LLM providers, auth, multimodal |
| `zeroclaw-config` | 21,683 | Config schema, secrets, SecurityPolicy |
| `zeroclaw-memory` | 11,041 | SQLite, Qdrant, embeddings, consolidation |
| `zeroclaw-infra` | 1,847 | Session backends, debounce, watchdog |
| `zeroclaw-types` | 600 | Core traits and message types |

## Sub-issues

| # | Title |
|---|---|
| [#1](https://github.com/singlerider/zeroclaw/issues/1) | Feature-gate channel-email |
| [#2](https://github.com/singlerider/zeroclaw/issues/2) | Feature-gate channel-mqtt |
| [#3](https://github.com/singlerider/zeroclaw/issues/3) | Feature-gate tui-onboarding |
| [#4](https://github.com/singlerider/zeroclaw/issues/4) | Feature-gate channel-telegram |
| [#5](https://github.com/singlerider/zeroclaw/issues/5) | Eliminate .eh_frame bloat |
| [#6](https://github.com/singlerider/zeroclaw/issues/6) | Workspace restructuring |
| [#7](https://github.com/singlerider/zeroclaw/issues/7) | StageX + linker/allocator exploration |
| [#8](https://github.com/singlerider/zeroclaw/issues/8) | Make schemars optional |
