//! Onboard orchestrator.
//!
//! Thin dispatcher above the `OnboardUi` trait (defined in
//! `zeroclaw-config::traits`). Section-scoped entry points let callers run
//! just one slice (`zeroclaw onboard channels`) or the whole flow.
//!
//! Sections are stubs in this commit. Each fills in as it's implemented.
//! Everything writes through `Config::set_prop` (or its helpers); direct
//! struct-field assignment is off-limits per the DRY contract (#5951).

use anyhow::Result;
use zeroclaw_config::schema::Config;
use zeroclaw_config::traits::{OnboardUi, SelectItem};

pub mod ui;

/// Which slice of onboarding to run. `All` runs every section in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    All,
    Workspace,
    Providers,
    Channels,
    Memory,
    Hardware,
    Tunnel,
}

/// Runtime knobs sourced from CLI flags. `--quick`/`--tui` select the UI
/// backend at the binary edge and don't appear here — the orchestrator only
/// cares about per-section behavior.
#[derive(Debug, Default, Clone)]
pub struct Flags {
    /// Skip "keep existing value?" confirmations; always re-prompt.
    pub force: bool,
    /// Back up the current config dir and start from `Config::default()`.
    pub reinit: bool,
    pub api_key: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub memory: Option<String>,
}

/// Top-level onboard dispatcher.
pub async fn run(
    cfg: &mut Config,
    ui: &mut dyn OnboardUi,
    section: Section,
    flags: &Flags,
) -> Result<()> {
    match section {
        Section::All => {
            workspace(cfg, ui, flags).await?;
            providers(cfg, ui, flags).await?;
            channels(cfg, ui, flags).await?;
            memory(cfg, ui, flags).await?;
            hardware(cfg, ui, flags).await?;
            tunnel(cfg, ui, flags).await?;
        }
        Section::Workspace => workspace(cfg, ui, flags).await?,
        Section::Providers => providers(cfg, ui, flags).await?,
        Section::Channels => channels(cfg, ui, flags).await?,
        Section::Memory => memory(cfg, ui, flags).await?,
        Section::Hardware => hardware(cfg, ui, flags).await?,
        Section::Tunnel => tunnel(cfg, ui, flags).await?,
    }
    Ok(())
}

// ── Section stubs ────────────────────────────────────────────────────────
// Each lands in its own commit. Bodies stay in mod.rs until one grows past
// ~50 lines, at which point it earns its own file under `sections/`.

async fn workspace(cfg: &mut Config, ui: &mut dyn OnboardUi, _flags: &Flags) -> Result<()> {
    ui.status(&format!(
        "Workspace directory: {}",
        cfg.workspace_dir.display()
    ));

    let currently_enabled = cfg.workspace.enabled;
    let enable = ui
        .confirm(
            "Enable multi-workspace isolation (separate memory / secrets / audit per workspace)?",
            currently_enabled,
        )
        .await?;
    if enable != currently_enabled {
        cfg.set_prop("workspace.enabled", &enable.to_string())?;
    }

    if !enable {
        return Ok(());
    }

    let current_name = cfg.workspace.active_workspace.clone().unwrap_or_default();
    let name = ui
        .string("Active workspace name", Some(&current_name))
        .await?;
    if name != current_name && !name.trim().is_empty() {
        cfg.set_prop("workspace.active-workspace", name.trim())?;
    }

    Ok(())
}

async fn providers(_cfg: &mut Config, _ui: &mut dyn OnboardUi, _flags: &Flags) -> Result<()> {
    Ok(())
}

async fn channels(_cfg: &mut Config, _ui: &mut dyn OnboardUi, _flags: &Flags) -> Result<()> {
    Ok(())
}

async fn memory(cfg: &mut Config, ui: &mut dyn OnboardUi, flags: &Flags) -> Result<()> {
    const BACKEND_IDS: &[&str] = &["sqlite", "lucid", "markdown", "none"];
    let options = [
        SelectItem::new("sqlite   — BM25 + optional embeddings (recommended)"),
        SelectItem::new("lucid    — local vector store"),
        SelectItem::new("markdown — plain text files, no DB"),
        SelectItem::new("none     — disable memory"),
    ];

    let current_backend = cfg.memory.backend.clone();
    let current_idx = BACKEND_IDS.iter().position(|id| *id == current_backend);

    let new_backend = if let Some(forced) = &flags.memory {
        forced.clone()
    } else {
        let idx = ui.select("Memory backend", &options, current_idx).await?;
        BACKEND_IDS[idx].to_string()
    };
    if new_backend != current_backend {
        cfg.set_prop("memory.backend", &new_backend)?;
    }

    let current_auto_save = cfg.memory.auto_save;
    let auto_save = ui
        .confirm("Auto-save user messages to memory?", current_auto_save)
        .await?;
    if auto_save != current_auto_save {
        cfg.set_prop("memory.auto-save", &auto_save.to_string())?;
    }
    Ok(())
}

async fn hardware(_cfg: &mut Config, _ui: &mut dyn OnboardUi, _flags: &Flags) -> Result<()> {
    Ok(())
}

async fn tunnel(_cfg: &mut Config, _ui: &mut dyn OnboardUi, _flags: &Flags) -> Result<()> {
    Ok(())
}
