use std::path::Path;

use anyhow::bail;
use plotly::Plot;
use polars::{
    io::{json::JsonReader, SerReader},
    lazy::frame::{IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame, LazyJsonLineReader},
};

pub fn read_df_file(
    path: impl AsRef<Path>,
    infer_schema_length: Option<usize>,
) -> anyhow::Result<LazyFrame> {
    let Some(extension) = path.as_ref().extension() else {
        bail!(
            "No extension at the name of the file `{}`",
            path.as_ref().to_string_lossy()
        );
    };
    Ok(match extension.to_string_lossy().as_ref() {
        "csv" => LazyCsvReader::new(&path)
            .has_header(true)
            .with_infer_schema_length(infer_schema_length)
            .finish()?,
        "json" => {
            let file = std::fs::File::options().read(true).open(&path)?;
            JsonReader::new(file).finish()?.lazy()
        }
        "ndjson" | "jsonl" => LazyJsonLineReader::new(&path)
            .with_infer_schema_length(infer_schema_length)
            .finish()?,
        _ => bail!(
            "Unknown extension `{}` at the name of the file `{}`",
            extension.to_string_lossy(),
            path.as_ref().to_string_lossy()
        ),
    })
}

pub fn output_plot(plot: Plot, output: Option<impl AsRef<Path>>) -> anyhow::Result<()> {
    match output {
        Some(output) => plot.write_html(output),
        None => plot.show(),
    }
    Ok(())
}
