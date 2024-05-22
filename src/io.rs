use std::path::Path;

use plotly::Plot;

pub fn output_plot(plot: Plot, output: Option<impl AsRef<Path>>) -> anyhow::Result<()> {
    match output {
        Some(output) => plot.write_html(output),
        None => plot.show(),
    }
    Ok(())
}
