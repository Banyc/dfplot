use std::path::PathBuf;

use anyhow::bail;
use clap::Args;
use math::{
    transformer::{
        proportion_scaler::{ProportionScaler, ProportionScalingEstimator},
        Estimate, Transform,
    },
    two_dim::VecZip,
};
use plotly::{
    common::Title,
    layout::{Axis, BarMode},
    Bar, Layout, Plot, Trace,
};
use polars::{frame::DataFrame, series::Series};

use crate::io::{output_plot, read_df_file};

#[derive(Debug, Clone, Args)]
pub struct BarArgs {
    input: PathBuf,
    #[clap(short, long, default_value = "x")]
    x: String,
    #[clap(short, long, default_value = "y")]
    y: Vec<String>,
    #[clap(short, long)]
    output: Option<PathBuf>,
    #[clap(short, long, default_value = "group")]
    barmode: String,
}

impl BarArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input, None)?;
        let plot = plot(df.collect()?, &self.x, &self.y, &self.barmode)?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(df: DataFrame, x: &str, y: &[String], barmode: &str) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();
    let x_title = Title::new(x);
    let mut scaler = None;
    let bar_mode = match barmode {
        "group" => BarMode::Group,
        "overlay" => BarMode::Overlay,
        "relative" => BarMode::Relative,
        "stack" => BarMode::Stack,
        "proportion" => {
            let y_columns = df
                .columns(y)?
                .into_iter()
                .map(|c| {
                    let c = c.to_float().unwrap();
                    let c = c.f64().unwrap().cont_slice().unwrap();
                    let c = c.to_vec();
                    c.into_iter()
                })
                .collect::<Vec<_>>();
            let rows = VecZip::new(y_columns);
            let est = ProportionScalingEstimator;
            scaler = Some(
                rows.map(|row| est.fit(row.into_iter()))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            dbg!(&scaler);
            BarMode::Stack
        }
        _ => bail!("Unknown barmode `{barmode}`"),
    };

    let x = df.column(x).ok();
    for y in y {
        let y = df.column(y)?;
        let trace = trace(x, y, scaler.as_deref())?;
        plot.add_trace(trace);
    }

    let mut layout = Layout::default()
        .x_axis(Axis::default().title(x_title))
        .bar_mode(bar_mode);
    if y.len() == 1 {
        layout = layout.y_axis(Axis::default().title(Title::new(y.first().unwrap())));
    }
    plot.set_layout(layout);
    Ok(plot)
}

fn trace(
    x: Option<&Series>,
    y: &Series,
    scaler: Option<&[ProportionScaler]>,
) -> anyhow::Result<Box<dyn Trace>> {
    let name = y.name();

    let x = match x {
        Some(x) => x
            .utf8()?
            .into_iter()
            .map(|x| x.map(|x| x.to_string()))
            .map(|x| match x {
                Some(x) => Ok(x),
                None => bail!("One string in column `{name}` not exists"),
            })
            .collect::<Result<_, _>>()?,
        None => (0..y.len()).map(|x| (x + 1).to_string()).collect(),
    };
    let y = y.to_float()?;
    let y = y.f64()?.cont_slice()?;
    let y = match scaler {
        Some(scaler) => y
            .iter()
            .zip(scaler.iter())
            .map(|(y, scaler)| {
                dbg!(y);
                dbg!(scaler.transform(*y))
            })
            .collect(),
        None => y.to_vec(),
    };
    let trace = Bar::new(x, y).name(name);
    Ok(trace)
}
