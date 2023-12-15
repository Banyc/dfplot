use clap::Subcommand;
use histogram::HistogramArgs;
use scatter::ScatterArgs;

pub mod histogram;
pub mod io;
pub mod scatter;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Scatter(ScatterArgs),
    Histogram(HistogramArgs),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Scatter(args) => args.run(),
            Command::Histogram(args) => args.run(),
        }
    }
}
