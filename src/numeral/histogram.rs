use std::path::PathBuf;

use banyc_polars_util::read_df_file;
use clap::Args;
use plotly::{layout::Axis, Histogram, Layout, Plot, Trace};
use polars::{
    frame::DataFrame,
    prelude::{Column, DataType},
};
use primitive::iter::assertion::AssertIteratorItemExt;

use crate::io::output_plot;

#[derive(Debug, Clone, Args)]
pub struct HistogramArgs {
    pub input: PathBuf,
    #[clap(short, long, default_value = "x")]
    pub x: Vec<String>,
    #[clap(short, long)]
    pub output: Option<PathBuf>,
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

    let mut layout = Layout::default().y_axis(Axis::default().title("count"));
    if x.len() == 1 {
        layout = layout.x_axis(Axis::default().title(x.first().unwrap()));
    }
    plot.set_layout(layout);
    Ok(plot)
}

fn trace(x: &Column) -> anyhow::Result<Box<dyn Trace>> {
    let name = x.name();
    let Ok(str) = x.str() else {
        let x: Vec<Option<f64>> = x.cast(&DataType::Float64)?.f64()?.to_vec();
        let trace = Histogram::new(x).name(name);
        return Ok(trace);
    };

    let x: Vec<Option<String>> = str
        .into_iter()
        .assert_item::<Option<&str>>()
        .map(|x| x.map(|x| x.to_string()))
        .assert_item::<Option<String>>()
        .collect::<Vec<Option<String>>>();
    let trace = Histogram::new(x).name(name);
    Ok(trace)
}
