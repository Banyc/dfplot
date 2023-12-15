use std::path::PathBuf;

use clap::Args;
use plotly::{Plot, Scatter, Trace};
use polars::{frame::DataFrame, series::Series};

use crate::io::read_df_file;

#[derive(Debug, Clone, Args)]
pub struct ScatterArgs {
    input: PathBuf,
    #[clap(short, long, default_value = "x")]
    x: String,
    #[clap(short, long, default_value = "y")]
    y: Vec<String>,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl ScatterArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input, None)?;
        let plot = plot(df.collect()?, &self.x, &self.y)?;
        match self.output {
            Some(output) => plot.write_html(output),
            None => plot.show(),
        }
        Ok(())
    }
}

fn plot(df: DataFrame, x: &str, y: &[String]) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();

    let x = df.column(x).ok();
    for y in y {
        let y = df.column(y)?;
        let trace = trace(x, y)?;
        plot.add_trace(trace);
    }

    Ok(plot)
}

fn trace(x: Option<&Series>, y: &Series) -> anyhow::Result<Box<dyn Trace>> {
    let name = y.name();
    let x = match x {
        Some(x) => x.to_float()?.f64()?.cont_slice()?.to_vec(),
        None => (0..y.len()).map(|x| (x + 1) as f64).collect(),
    };
    let y = y.to_float()?.f64()?.cont_slice()?.to_vec();
    let trace = Scatter::new(x, y).name(name);
    Ok(trace)
}
