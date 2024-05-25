use crate::migration::migration_0_1::Migration0To1;
use anyhow::Context;
use serde_json::{json, Value};

mod migration_0_1;

pub trait Migration {
    fn migrate(old: &Value) -> anyhow::Result<Value>;
}

pub const CURRENT_FORMAT: u64 = 1;

fn get_format(chart: &Value) -> anyhow::Result<u64> {
    let version = chart
        .get("format")
        .unwrap_or(&json!(0))
        .as_u64()
        .context("format field is not a number")?;

    Ok(version)
}

/// Migrate a chart to the latest format
pub fn migrate(chart: &Value) -> anyhow::Result<Value> {
    let format = get_format(chart)?;

    let new_chart = match format {
        0 => Migration0To1::migrate(chart)?,
        _ => unreachable!(),
    };

    let new_format = get_format(&new_chart)?;

    if new_format < CURRENT_FORMAT {
        migrate(&new_chart)
    } else {
        Ok(new_chart)
    }
}
