use anyhow::Context;
use polars::frame::DataFrame;

pub fn category_names(df: &DataFrame, column_name: &str) -> anyhow::Result<Vec<String>> {
    let column = df.column(column_name)?;
    let categories = column.unique()?;
    let category_names = categories
        .utf8()?
        .into_iter()
        .map(|x| x.map(|s| s.to_string()).context("No string in group"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(category_names)
}
