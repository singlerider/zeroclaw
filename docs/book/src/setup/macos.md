# macOS

Install, update, run as a LaunchAgent, and uninstall on macOS (Intel or Apple Silicon).

## Install

### Option 1 — Homebrew (recommended)

```bash
brew install zeroclaw
zeroclaw onboard
```

Homebrew is the best-supported path on macOS. Updates are one command, the `brew services` integration Just Works, and the install is in `$HOMEBREW_PREFIX/bin/zeroclaw` where `PATH` already looks.

**Workspace location:** with Homebrew, the service user and the CLI user may be different, so the workspace lives at `$HOMEBREW_PREFIX/var/zeroclaw/` rather than `~/.zeroclaw/`. Point CLI invocations at the same workspace:

```bash
export ZEROCLAW_WORKSPACE="$HOMEBREW_PREFIX/var/zeroclaw"
```

Add that to your shell profile if you want it permanent.

### Option 2 — One-liner bootstrap

```bash
curl -fsSL https://raw.githubusercontent.com/zeroclaw-labs/zeroclaw/master/install.sh | bash
```

Installs to `~/.cargo/bin/zeroclaw`. Workspace at `~/.zeroclaw/`. No Homebrew required. Uses prebuilt binaries when available for your architecture.

### Option 3 — From source

```bash
git clone https://github.com/zeroclaw-labs/zeroclaw
cd zeroclaw
cargo install --locked --path .
zeroclaw onboard
```

Requires Rust stable (`rustup`). First build is slow.

## System dependencies

Most features work with a stock macOS install. Optional extras:

| Feature | Install |
|---|---|
| Docs translation | `brew install gettext` |
| Browser tool | Playwright pulls Chromium automatically on first use |
| Hardware | No native GPIO on macOS; use a USB peripheral like Aardvark. See [Hardware → Aardvark](../hardware/aardvark.md) |
| iMessage channel | Requires macOS 11+. See [Channels → Other chat platforms](../channels/chat-others.md) |

## Running as a service

```bash
zeroclaw service install   # writes ~/Library/LaunchAgents/com.zeroclaw.daemon.plist
zeroclaw service start
zeroclaw service status
```

Logs go to `~/Library/Logs/ZeroClaw/`:

```bash
tail -f ~/Library/Logs/ZeroClaw/zeroclaw.log
```

For Homebrew installs, prefer:

```bash
brew services start zeroclaw
brew services info zeroclaw
```

Both methods produce the same end state — a loaded LaunchAgent that starts on login. Pick one and stick with it.

Full details: [Service management](./service.md).

## Update

Homebrew:

```bash
brew update && brew upgrade zeroclaw
brew services restart zeroclaw
```

Bootstrap install:

```bash
curl -fsSL https://raw.githubusercontent.com/zeroclaw-labs/zeroclaw/master/install.sh | bash -s -- --skip-onboard
zeroclaw service restart
```

Source install:

```bash
cd /path/to/zeroclaw
git pull
cargo install --locked --path . --force
zeroclaw service restart
```

## Uninstall

```bash
# stop and unregister the service
zeroclaw service stop
zeroclaw service uninstall

# Homebrew
brew uninstall zeroclaw

# bootstrap / cargo
rm ~/.cargo/bin/zeroclaw
```

Remove config and workspace (optional — this deletes conversation history):

```bash
# Homebrew workspace
rm -rf "$HOMEBREW_PREFIX/var/zeroclaw"

# Default workspace
rm -rf ~/.zeroclaw ~/.config/zeroclaw

# Logs
rm -rf ~/Library/Logs/ZeroClaw
```

## Gotchas

- **Homebrew config path mismatch.** The wizard warns if it detects Homebrew — the `brew services` daemon reads `$HOMEBREW_PREFIX/var/zeroclaw/config.toml`, not `~/.zeroclaw/config.toml`. If your service is reading stale config, check which one the daemon sees.
- **First launch of the browser tool** downloads Chromium (~150 MB) via Playwright.
- **Apple Silicon** and **Intel** builds are both released. The bootstrap script auto-detects. Homebrew auto-selects.

## Next

- [Service management](./service.md)
- [Quick start](../getting-started/quick-start.md)
- [Operations → Overview](../ops/overview.md)
