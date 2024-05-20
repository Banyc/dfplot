use std::{borrow::Cow, path::PathBuf};

use anyhow::bail;
use clap::Args;
use plotly::{
    common::{Mode, Title},
    layout::Axis,
    Layout, Plot, Scatter, Trace,
};
use polars::{frame::DataFrame, lazy::frame::IntoLazy, series::Series};

use crate::{
    group::Groups,
    io::{output_plot, read_df_file},
};

#[derive(Debug, Clone, Args)]
pub struct ScatterArgs {
    pub input: PathBuf,
    #[clap(short, long, default_value = "x")]
    pub x: String,
    #[clap(short, long, default_value = "y")]
    pub y: Vec<String>,
    #[clap(short, long)]
    pub group: Option<Vec<String>>,
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// Options: `lines`, `markers`, `text`, or combinations like `lines+markers` or `lines,markers`
    #[clap(short, long)]
    pub mode: Option<String>,
}

impl ScatterArgs {
    pub fn run(self) -> anyhow::Result<()> {
        let df = read_df_file(self.input)?;
        let plot = plot(
            df.collect()?,
            &self.x,
            &self.y,
            self.group,
            self.mode.as_deref(),
        )?;
        output_plot(plot, self.output.as_deref())?;
        Ok(())
    }
}

fn plot(
    df: DataFrame,
    x: &str,
    y: &[String],
    groups: Option<Vec<String>>,
    mode: Option<&str>,
) -> anyhow::Result<Plot> {
    let mut plot = Plot::new();
    let x_title = Title::new(x);

    let groups = match groups {
        Some(groups) => Some(Groups::from_df(&df, groups)?),
        None => None,
    };

    let mode = match mode {
        Some(mode) => Some(parse_mode(mode)?),
        None => None,
    };

    match groups {
        Some(groups) => {
            groups.for_each_product(df.lazy(), |df, groups| {
                let groups = groups
                    .into_iter()
                    .map(|pair| pair.category)
                    .collect::<Vec<_>>();
                let df = df.collect()?;
                let x: Option<&Series> = df.column(x).ok();
                for y in y {
                    let y: &Series = df.column(y)?;
                    let trace = trace(x, y, Some(&groups), mode.clone())?;
                    plot.add_trace(trace);
                }
                Ok(())
            })?;
        }
        None => {
            let x = df.column(x).ok();
            for y in y {
                let y = df.column(y)?;
                let trace = trace(x, y, None, mode.clone())?;
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

fn trace(
    x: Option<&Series>,
    y: &Series,
    groups: Option<&[&str]>,
    mode: Option<Mode>,
) -> anyhow::Result<Box<dyn Trace>> {
    let name: Cow<str> = match groups {
        Some(groups) => format!("{:?}:{}", groups, y.name()).into(),
        None => y.name().into(),
    };

    let x: Vec<Option<f64>> = match x {
        Some(x) => x.to_float()?.f64()?.to_vec(),
        None => (0..y.len()).map(|x| (x + 1) as f64).map(Some).collect(),
    };
    let y: Vec<Option<f64>> = y.to_float()?.f64()?.to_vec();
    let mut trace = Scatter::new(x, y).name(name);
    if let Some(mode) = mode {
        trace = trace.mode(mode);
    }
    Ok(trace)
}

fn parse_mode(src: &str) -> anyhow::Result<Mode> {
    #[derive(Debug)]
    struct CheckList {
        lines: bool,
        markers: bool,
        text: bool,
    }
    let mut check_list = CheckList {
        lines: false,
        markers: false,
        text: false,
    };
    let options = src.split([',', '+']);
    for option in options {
        match option.trim() {
            "lines" => check_list.lines = true,
            "markers" => check_list.markers = true,
            "text" => check_list.text = true,
            "none" => return Ok(Mode::None),
            _ => bail!("Unknown mode `{option}`"),
        }
    }
    Ok(match check_list {
        CheckList {
            lines: true,
            markers: false,
            text: false,
        } => Mode::Lines,
        CheckList {
            lines: true,
            markers: true,
            text: false,
        } => Mode::LinesMarkers,
        CheckList {
            lines: true,
            markers: true,
            text: true,
        } => Mode::LinesMarkersText,
        CheckList {
            lines: true,
            markers: false,
            text: true,
        } => Mode::LinesText,
        CheckList {
            lines: false,
            markers: true,
            text: false,
        } => Mode::Markers,
        CheckList {
            lines: false,
            markers: true,
            text: true,
        } => Mode::MarkersText,
        CheckList {
            lines: false,
            markers: false,
            text: true,
        } => Mode::Text,
        CheckList {
            lines: false,
            markers: false,
            text: false,
        } => Mode::None,
    })
}
