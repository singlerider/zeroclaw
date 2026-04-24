# CI & Actions

Every workflow lives in `.github/workflows/`. The sections below group them by trigger — automatic on git events, or manual via `workflow_dispatch`.

## Automatic workflows

### Quality Gate (`ci.yml`)

Fires on every PR targeting `master`. Composite job with multiple matrix legs:

- **lint** — `cargo fmt --check`, `cargo clippy -D warnings`, `cargo check --features ci-all`, strict delta lint on changed lines
- **build** — matrix: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
- **check** — all features + no-default-features
- **check-32bit** — `i686-unknown-linux-gnu` with no default features
- **bench** — benchmarks compile check
- **test** — `cargo nextest run --locked` on Linux
- **security** — `cargo deny check`

`CI Required Gate` is the composite job branch protection pins. A PR cannot merge until this is green.

### Daily Advisory Scan (`daily-audit.yml`)

Runs `cargo audit` nightly against the dependency tree. Opens an issue on findings. No action unless a vulnerability is reported.

### PR Path Labeler (`pr-path-labeler.yml`)

Auto-applies scope and risk labels based on changed file paths. Runs silently on every PR — if a PR is missing labels, check whether the paths in `.github/labeler.yml` cover the changes.

### Discord Release (`discord-release.yml`)

Fires after a successful stable release. Posts the release notes to the community Discord.

### Tweet Release (`tweet-release.yml`)

Fires after a successful stable release. Posts an announcement tweet.

### Sync Marketplace Templates (`sync-marketplace-templates.yml`)

Fires after every stable release. Auto-opens PRs to update version numbers in the downstream marketplace template repos (docker, k8s, compose).

Docs are built and published as part of the release pipeline rather than on every `master` push. Translation is a local-only workflow — run `cargo mdbook sync --provider <name>` before PRing. See [Docs & Translations](./docs-and-translations.md) for details.

## Manual workflows

### Cross-Platform Build (`cross-platform-build-manual.yml`)

Manual trigger for building release binaries across the full target matrix (Linux GNU/MUSL, macOS Intel/ARM, Windows, additional ARM Linux targets). Use this to verify a branch compiles cleanly on non-Linux targets before tagging.

### Release Stable (`release-stable-manual.yml`)

Manual trigger for the full release pipeline. Builds all targets, creates the GitHub Release, publishes to crates.io, pushes Docker images, and invokes downstream workflows. Three environment gates require maintainer approval mid-run: `github-releases`, `crates-io`, `docker`.

See the release runbook in the repo's `docs/maintainers/` directory for the full procedure (not yet migrated into this mdBook).

### Package Publishers

Each fires on `workflow_dispatch` with a version input. They are also invoked from the release workflow after a successful publish.

| Workflow | What it does |
|---|---|
| `pub-aur.yml` | Updates the Arch User Repository `PKGBUILD` and pushes to the AUR |
| `pub-homebrew-core.yml` | Opens a PR against `homebrew/homebrew-core` with the new version |
| `pub-scoop.yml` | Updates the Scoop manifest for Windows |

## Required secrets

| Secret | Used by |
|---|---|
| `AUR_SSH_KEY` | `pub-aur.yml` |
| `DISCORD_WEBHOOK_URL` | `discord-release.yml` |
| `TWITTER_*` tokens | `tweet-release.yml` |
| `HOMEBREW_CORE_TOKEN` | `pub-homebrew-core.yml` |
| `CARGO_REGISTRY_TOKEN` | `release-stable-manual.yml` |
| `DOCKER_HUB_TOKEN` | `release-stable-manual.yml` |
| `GITHUB_TOKEN` (automatic) | All workflows that push commits or open PRs |
