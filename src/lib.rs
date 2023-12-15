use clap::Subcommand;
use scatter::ScatterArgs;

pub mod io;
pub mod scatter;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Scatter(ScatterArgs),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Scatter(args) => args.run(),
        }
    }
}
