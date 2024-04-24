use anyhow::Context;
use polars::{frame::DataFrame, series::Series};

pub fn category_names(df: &DataFrame, column_name: &str) -> anyhow::Result<Vec<String>> {
    let column = df.column(column_name)?;
    let categories = column.unique_stable()?;
    let category_names = str_values(&categories)?;
    Ok(category_names)
}

pub fn str_values(column: &Series) -> anyhow::Result<Vec<String>> {
    let values = column
        .str()?
        .into_iter()
        .map(|x| {
            x.map(|s| s.to_string())
                .with_context(|| format!("No string in the column `{}`", column.name()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values)
}
