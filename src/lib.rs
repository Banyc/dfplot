use clap::Subcommand;
use histogram::HistogramArgs;
use r#box::BoxArgs;
use scatter::ScatterArgs;

pub mod r#box;
pub mod histogram;
pub mod io;
pub mod scatter;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Scatter(ScatterArgs),
    Histogram(HistogramArgs),
    Box(BoxArgs),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Scatter(args) => args.run(),
            Command::Histogram(args) => args.run(),
            Command::Box(args) => args.run(),
        }
    }
}
