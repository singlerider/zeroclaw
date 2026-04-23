use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fluent", about = "ZeroClaw Fluent app UI translation")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan Rust source for user-facing strings and generate en.ftl
    Scan,
    /// AI-fill missing translations in non-English .ftl files
    Fill {
        #[arg(long)]
        locale: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show translation coverage per locale
    Stats,
    /// Validate .ftl syntax for all locales
    Check,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Scan => anyhow::bail!("not yet implemented"),
        Cmd::Fill { .. } => anyhow::bail!("not yet implemented"),
        Cmd::Stats => anyhow::bail!("not yet implemented"),
        Cmd::Check => anyhow::bail!("not yet implemented"),
    }
}
