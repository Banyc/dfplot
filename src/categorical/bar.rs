use std::{borrow::Cow, collections::HashMap, path::PathBuf};

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
use polars::{
    frame::DataFrame,
    lazy::{dsl::col, frame::IntoLazy},
    series::Series,
};
use strict_num::PositiveF64;

use crate::{
    df::str_values,
    group::Groups,
    io::{output_plot, read_df_file},
};

#[derive(Debug, Clone, Args)]
pub struct BarArgs {
    pub input: PathBuf,
    #[clap(short, long, default_value = "x")]
    pub x: String,
    #[clap(short, long, default_value = "y")]
    pub y: Vec<String>,
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// `group` (default), `overlay`, `relative`, `stack`, `proportion`
    #[clap(short, long, default_value = "group")]
    pub barmode: String,
    #[clap(short, long)]
    pub group: Option<Vec<String>>,
}

impl BarArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input)?;
        let plot = plot(df.collect()?, &self.x, &self.y, self.group, &self.barmode)?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(
    df: DataFrame,
    x: &str,
    y: &[String],
    groups: Option<Vec<String>>,
    barmode: &str,
) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();
    let x_title = Title::new(x);

    let groups = match groups {
        Some(groups) => Some(Groups::from_df(&df, groups)?),
        None => None,
    };

    let mut scaler = None;
    let bar_mode = match barmode {
        "group" => BarMode::Group,
        "overlay" => BarMode::Overlay,
        "relative" => BarMode::Relative,
        "stack" => BarMode::Stack,
        "proportion" => {
            let mut df = df.clone().lazy();

            let y_columns = y.iter().map(|y| col(y).sum()).collect::<Vec<_>>();
            df = df.group_by([col(x)]).agg(y_columns);

            let df = df.collect()?;
            let x_names = str_values(df.column(x)?)?;
            let y_columns = df
                .columns(y)?
                .into_iter()
                .map(|c| {
                    let c = c.to_float().unwrap();
                    let c = c.f64().unwrap().cont_slice().unwrap();
                    let c = c
                        .iter()
                        .map(|c| {
                            PositiveF64::new(*c)
                                .ok_or_else(|| anyhow::anyhow!("negative number in y"))
                        })
                        .collect::<Result<Vec<_>, _>>();
                    c.map(|c| c.into_iter())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let rows = VecZip::new(y_columns);
            let est = ProportionScalingEstimator;
            let scalers = rows
                .map(|row| est.fit(row.into_iter()))
                .collect::<Result<Vec<_>, _>>()?;
            let scalers = x_names
                .into_iter()
                .zip(scalers)
                .collect::<HashMap<String, ProportionScaler>>();
            scaler = Some(scalers);
            BarMode::Stack
        }
        _ => anyhow::bail!("Unknown barmode `{barmode}`"),
    };

    match groups {
        Some(groups) => {
            groups.for_each_product(df.lazy(), |df, groups| {
                let groups = groups
                    .into_iter()
                    .map(|pair| pair.category)
                    .collect::<Vec<_>>();
                let df = df.collect()?;
                let x = df.column(x).ok();
                for y in y {
                    let y = df.column(y)?;
                    let trace = trace(x, y, Some(&groups), scaler.as_ref())?;
                    plot.add_trace(trace);
                }
                Ok(())
            })?;
        }
        None => {
            let x = df.column(x).ok();
            for y in y {
                let y = df.column(y)?;
                let trace = trace(x, y, None, scaler.as_ref())?;
                plot.add_trace(trace);
            }
        }
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
    groups: Option<&[&str]>,
    scaler: Option<&HashMap<String, ProportionScaler>>,
) -> anyhow::Result<Box<dyn Trace>> {
    let name: Cow<str> = match groups {
        Some(groups) => format!("{:?}:{}", groups, y.name()).into(),
        None => y.name().into(),
    };

    let x = match x {
        Some(x) => x
            .str()?
            .into_iter()
            .map(|x| x.map(|x| x.to_string()))
            .map(|x| x.ok_or_else(|| anyhow::anyhow!("One string in column `{name}` not exists")))
            .collect::<Result<Vec<_>, _>>()?,
        None => (0..y.len()).map(|x| (x + 1).to_string()).collect(),
    };
    let y = y.to_float()?;
    let y = y.f64()?.cont_slice()?;
    let y = match scaler {
        Some(scaler) => y
            .iter()
            .zip(x.iter())
            .map(|(y, x)| -> anyhow::Result<f64> {
                let Some(y) = PositiveF64::new(*y) else {
                    anyhow::bail!("negative number in y");
                };
                let proportion = scaler.get(x).unwrap().transform(y)?;
                Ok(proportion.get())
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => y.to_vec(),
    };
    let trace = Bar::new(x, y).name(name);
    Ok(trace)
}
