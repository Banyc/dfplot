use clap::Parser;
use dfplot::Command;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run()?;
    Ok(())
}
