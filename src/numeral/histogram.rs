use std::path::PathBuf;

use anyhow::bail;
use clap::Args;
use plotly::{common::Title, layout::Axis, Histogram, Layout, Plot, Trace};
use polars::{frame::DataFrame, series::Series};

use crate::io::{output_plot, read_df_file};

#[derive(Debug, Clone, Args)]
pub struct HistogramArgs {
    input: PathBuf,
    #[clap(short, long, default_value = "x")]
    x: Vec<String>,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl HistogramArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input)?;
        let plot = plot(df.collect()?, &self.x)?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(df: DataFrame, x: &[String]) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();

    for x in x {
        let x = df.column(x)?;
        let trace = trace(x)?;
        plot.add_trace(trace);
    }

    let mut layout = Layout::default().y_axis(Axis::default().title(Title::new("count")));
    if x.len() == 1 {
        layout = layout.x_axis(Axis::default().title(Title::new(x.first().unwrap())));
    }
    plot.set_layout(layout);
    Ok(plot)
}

fn trace(x: &Series) -> anyhow::Result<Box<dyn Trace>> {
    let name = x.name();
    let Ok(str) = x.str() else {
        let x = x.to_float()?.f64()?.cont_slice()?.to_vec();
        let trace = Histogram::new(x).name(name);
        return Ok(trace);
    };

    let x = str
        .into_iter()
        .map(|x| x.map(|x| x.to_string()))
        .map(|x| match x {
            Some(x) => Ok(x),
            None => bail!("One string in column `{name}` not exists"),
        })
        .collect::<Result<_, _>>()?;
    let trace = Histogram::new(x).name(name);
    Ok(trace)
}
