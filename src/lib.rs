use categorical::bar::BarArgs;
use clap::Subcommand;
use numeral::{histogram::HistogramArgs, r#box::BoxArgs, scatter::ScatterArgs};

pub mod categorical;
pub mod io;
pub mod numeral;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Scatter(ScatterArgs),
    Histogram(HistogramArgs),
    Box(BoxArgs),
    Bar(BarArgs),
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Scatter(args) => args.run(),
            Command::Histogram(args) => args.run(),
            Command::Box(args) => args.run(),
            Command::Bar(args) => args.run(),
        }
    }
}
