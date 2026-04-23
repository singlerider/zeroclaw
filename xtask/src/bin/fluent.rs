use xtask::cmd;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fluent", about = "ZeroClaw Fluent app UI translation")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan Rust source for user-facing strings and report en.ftl coverage
    Scan,
    /// AI-fill missing translations in non-English .ftl files
    Fill {
        #[arg(long)]
        locale: Option<String>,
        /// Re-translate all entries (quality pass, costs more)
        #[arg(long)]
        force: bool,
        /// Provider name from [providers.models.<name>] in config.toml (e.g. my-ollama)
        #[arg(long)]
        provider: Option<String>,
    },
    /// Show translation coverage per locale
    Stats,
    /// Validate .ftl syntax for all locales
    Check,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Scan                   => cmd::fluent::scan::run(),
        Cmd::Fill { locale, force, provider } => cmd::fluent::fill::run(locale.as_deref(), force, provider.as_deref()),
        Cmd::Stats                  => cmd::fluent::stats::run(),
        Cmd::Check                  => cmd::fluent::check::run(),
    }
}
