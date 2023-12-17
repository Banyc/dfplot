use itertools::Itertools;
use polars::{
    frame::DataFrame,
    lazy::{
        dsl::{col, lit},
        frame::LazyFrame,
    },
};

use crate::df::category_names;

pub struct Group {
    column_name: String,
    categories: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct SelectedGroupCategoryPair<'a> {
    pub column_name: &'a str,
    pub category: &'a str,
}

pub struct Groups {
    groups: Vec<Group>,
}
impl Groups {
    pub fn from_df(df: &DataFrame, group_column_names: Vec<String>) -> anyhow::Result<Self> {
        let mut groups = vec![];
        for column_name in group_column_names {
            let categories = category_names(df, &column_name)?;
            let g = Group {
                column_name,
                categories,
            };
            groups.push(g);
        }
        Ok(Self { groups })
    }

    pub fn for_each_product(
        &self,
        df: LazyFrame,
        mut f: impl FnMut(LazyFrame, Vec<SelectedGroupCategoryPair>) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let groups = self.groups.iter().map(|g| {
            g.categories
                .iter()
                .map(|category| SelectedGroupCategoryPair {
                    column_name: &g.column_name,
                    category,
                })
        });
        let product = groups.multi_cartesian_product();
        for groups in product {
            let mut df = df.clone();
            for pair in &groups {
                df = df.filter(col(pair.column_name).eq(lit(pair.category)));
            }
            f(df, groups)?;
        }
        Ok(())
    }
}
