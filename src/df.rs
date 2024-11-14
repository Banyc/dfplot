use anyhow::Context;
use polars::{frame::DataFrame, prelude::Column};

pub fn category_names(df: &DataFrame, column_name: &str) -> anyhow::Result<Vec<String>> {
    let column = df.column(column_name)?;
    let categories = column.unique_stable()?;
    let category_names: Vec<String> = filter_str_values(&categories)?;
    Ok(category_names)
}

pub fn cont_str_values(column: &Column) -> anyhow::Result<Vec<String>> {
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

pub fn filter_str_values(column: &Column) -> anyhow::Result<Vec<String>> {
    let values = column
        .str()?
        .into_iter()
        .filter_map(|x| x.map(|s| s.to_string()))
        .collect::<Vec<String>>();
    Ok(values)
}
