use std::{borrow::Cow, path::PathBuf};

use clap::Args;
use plotly::{common::Title, layout::Axis, Layout, Plot, Scatter, Trace};
use polars::{
    frame::DataFrame,
    lazy::{
        dsl::{col, lit},
        frame::IntoLazy,
    },
    series::Series,
};

use crate::{
    df::category_names,
    io::{output_plot, read_df_file},
};

#[derive(Debug, Clone, Args)]
pub struct ScatterArgs {
    input: PathBuf,
    #[clap(short, long, default_value = "x")]
    x: String,
    #[clap(short, long, default_value = "y")]
    y: Vec<String>,
    #[clap(short, long)]
    group: Option<String>,
    #[clap(short, long)]
    output: Option<PathBuf>,
}

impl ScatterArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input, None)?;
        let plot = plot(df.collect()?, &self.x, &self.y, self.group.as_deref())?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(df: DataFrame, x: &str, y: &[String], group: Option<&str>) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();
    let x_title = Title::new(x);

    let group_names = match group {
        Some(group) => {
            let group_names = category_names(&df, group)?;
            Some((group, group_names))
        }
        None => None,
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
                    let trace = trace(x, y, Some(&group_name))?;
                    plot.add_trace(trace);
                }
            }
        }
        None => {
            let x = df.column(x).ok();
            for y in y {
                let y = df.column(y)?;
                let trace = trace(x, y, None)?;
                plot.add_trace(trace);
            }
        }
    }

    let mut layout = Layout::default().x_axis(Axis::default().title(x_title));
    if y.len() == 1 {
        layout = layout.y_axis(Axis::default().title(Title::new(y.first().unwrap())));
    }
    plot.set_layout(layout);
    Ok(plot)
}

fn trace(x: Option<&Series>, y: &Series, group: Option<&str>) -> anyhow::Result<Box<dyn Trace>> {
    let name: Cow<str> = match group {
        Some(group) => format!("{}:{}", group, y.name()).into(),
        None => y.name().into(),
    };
    let x = match x {
        Some(x) => x.to_float()?.f64()?.cont_slice()?.to_vec(),
        None => (0..y.len()).map(|x| (x + 1) as f64).collect(),
    };
    let y = y.to_float()?.f64()?.cont_slice()?.to_vec();
    let trace = Scatter::new(x, y).name(name);
    Ok(trace)
}
