use std::path::PathBuf;

use clap::Args;
use plotly::{common::Title, layout::Axis, BoxPlot, Layout, Plot, Trace};
use polars::{frame::DataFrame, series::Series};

use crate::io::{output_plot, read_df_file};

#[derive(Debug, Clone, Args)]
pub struct BoxArgs {
    pub input: PathBuf,
    #[clap(short, long, default_value = "y")]
    pub y: Vec<String>,
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

impl BoxArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input)?;
        let plot = plot(df.collect()?, &self.y)?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(df: DataFrame, y: &[String]) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();

    for y in y {
        let y = df.column(y)?;
        let trace = trace(y)?;
        plot.add_trace(trace);
    }

    let mut layout = Layout::default();
    if y.len() == 1 {
        layout = layout.x_axis(Axis::default().title(Title::new(y.first().unwrap())));
    }
    plot.set_layout(layout);
    Ok(plot)
}

fn trace(y: &Series) -> anyhow::Result<Box<dyn Trace>> {
    let name = y.name();
    let y = y.to_float()?.f64()?.to_vec();
    let trace = BoxPlot::new(y).name(name);
    Ok(trace)
}
