# Linux

Install, update, run as a service, and uninstall — all Linux distributions.

## Install

### Option 1 — One-liner bootstrap (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/zeroclaw-labs/zeroclaw/master/install.sh | bash
```

This script:

1. Detects your distribution and architecture
2. Downloads a prebuilt binary if available for your target, falls back to `cargo install` otherwise
3. Places the binary at `~/.cargo/bin/zeroclaw` (or `$HOMEBREW_PREFIX/bin/zeroclaw` if Homebrew is present on Linux)
4. Runs `zeroclaw onboard` to complete first-time setup

Skip the onboard step with `--skip-onboard` if you want to hand-edit `~/.zeroclaw/config.toml` first.

### Option 2 — Homebrew

If you have Linuxbrew:

```bash
brew install zeroclaw
zeroclaw onboard
```

Homebrew-on-Linux installs follow Homebrew's service path convention — your workspace lives under `$HOMEBREW_PREFIX/var/zeroclaw/` instead of `~/.zeroclaw/`. See [Service management](./service.md) for why this matters.

### Option 3 — From source

```bash
git clone https://github.com/zeroclaw-labs/zeroclaw
cd zeroclaw
cargo install --locked --path .
zeroclaw onboard
```

Requires Rust stable (install via `rustup`). First build is slow; subsequent builds use the `cargo install` cache.

## System dependencies

The core binary is statically linked where possible. Some features require system libraries:

| Feature | Package (Debian/Ubuntu) | Package (Arch) | Package (Fedora) |
|---|---|---|---|
| Docs translation (`cargo mdbook sync`) | `gettext` | `gettext` | `gettext` |
| Hardware (GPIO / I2C / SPI) | `libgpiod-dev` | `libgpiod` | `libgpiod-devel` |
| Browser tool (playwright) | `libnss3`, `libatk1.0-0`, `libcups2` (see `playwright --help`) | `nss`, `atk`, `cups` | `nss`, `atk`, `cups` |
| Audio (TTS, voice channels) | `libasound2-dev` | `alsa-lib` | `alsa-lib-devel` |

Most deployments don't need any of these.

## Running as a service

Systemd is the default. OpenRC is detected and supported as a fallback.

```bash
zeroclaw service install
zeroclaw service start
zeroclaw service status
```

Logs go to the systemd journal by default:

```bash
journalctl --user -u zeroclaw -f
```

Full details: [Service management](./service.md).

### SBC / Raspberry Pi

On a Raspberry Pi or similar SBC, install with the hardware feature:

```bash
cargo install --locked --path . --features hardware
```

The stock systemd unit includes `SupplementaryGroups=gpio spi i2c` so the service user can access hardware without running as root. Verify your user is in those groups:

```bash
getent group gpio spi i2c
sudo usermod -aG gpio,spi,i2c $USER
# re-login for group changes to take effect
```

## Update

If installed via the bootstrap script:

```bash
curl -fsSL https://raw.githubusercontent.com/zeroclaw-labs/zeroclaw/master/install.sh | bash -s -- --skip-onboard
```

If installed via Homebrew:

```bash
brew update && brew upgrade zeroclaw
```

If installed via `cargo install`:

```bash
cd /path/to/zeroclaw
git pull
cargo install --locked --path . --force
```

After updating, restart the service:

```bash
zeroclaw service restart
```

## Uninstall

Stop and remove the service:

```bash
zeroclaw service stop
zeroclaw service uninstall
```

Remove the binary:

```bash
# cargo install / bootstrap
rm ~/.cargo/bin/zeroclaw

# Homebrew
brew uninstall zeroclaw
```

Remove config and workspace (optional — this deletes conversation history):

```bash
rm -rf ~/.zeroclaw ~/.config/zeroclaw
```

## Next

- [Service management](./service.md) — systemd unit details, logs, auto-start
- [Quick start](../getting-started/quick-start.md) — once installed, getting talking
- [Operations → Overview](../ops/overview.md) — running in production
