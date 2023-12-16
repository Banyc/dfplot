use std::{borrow::Cow, collections::HashMap, path::PathBuf};

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
use polars::{
    frame::DataFrame,
    lazy::{
        dsl::{col, lit},
        frame::IntoLazy,
    },
    series::Series,
};

use crate::{
    df::{category_names, utf8_values},
    io::{output_plot, read_df_file},
};

#[derive(Debug, Clone, Args)]
pub struct BarArgs {
    input: PathBuf,
    #[clap(short, long, default_value = "x")]
    x: String,
    #[clap(short, long, default_value = "y")]
    y: Vec<String>,
    #[clap(short, long)]
    output: Option<PathBuf>,
    /// `group` (default), `overlay`, `relative`, `stack`, `proportion`
    #[clap(short, long, default_value = "group")]
    barmode: String,
    #[clap(short, long)]
    group: Option<String>,
}

impl BarArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input, None)?;
        let plot = plot(
            df.collect()?,
            &self.x,
            &self.y,
            self.group.as_deref(),
            &self.barmode,
        )?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(
    df: DataFrame,
    x: &str,
    y: &[String],
    group: Option<&str>,
    barmode: &str,
) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();
    let x_title = Title::new(x);

    let group_names = match group {
        Some(group) => {
            let group_names = category_names(&df, group)?;
            Some((group, group_names))
        }
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
            let x_names = utf8_values(df.column(x)?)?;
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
        _ => bail!("Unknown barmode `{barmode}`"),
    };

    match group_names {
        Some((group, group_names)) => {
            let lazy = df.lazy();
            for group_name in group_names {
                let df = lazy.clone();
                let df = df
                    .clone()
                    .filter(col(group).eq(lit(&*group_name)))
                    .collect()?;
                let x = df.column(x).ok();
                for y in y {
                    let y = df.column(y)?;
                    let trace = trace(x, y, Some(&group_name), scaler.as_ref())?;
                    plot.add_trace(trace);
                }
            }
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
    group: Option<&str>,
    scaler: Option<&HashMap<String, ProportionScaler>>,
) -> anyhow::Result<Box<dyn Trace>> {
    let name: Cow<str> = match group {
        Some(group) => format!("{}:{}", group, y.name()).into(),
        None => y.name().into(),
    };

    let x = match x {
        Some(x) => x
            .utf8()?
            .into_iter()
            .map(|x| x.map(|x| x.to_string()))
            .map(|x| match x {
                Some(x) => Ok(x),
                None => bail!("One string in column `{name}` not exists"),
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => (0..y.len()).map(|x| (x + 1).to_string()).collect(),
    };
    let y = y.to_float()?;
    let y = y.f64()?.cont_slice()?;
    let y = match scaler {
        Some(scaler) => y
            .iter()
            .zip(x.iter())
            .map(|(y, x)| scaler.get(x).unwrap().transform(*y))
            .collect(),
        None => y.to_vec(),
    };
    let trace = Bar::new(x, y).name(name);
    Ok(trace)
}
