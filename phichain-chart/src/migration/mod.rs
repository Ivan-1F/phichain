use crate::migration::migration_0_1::Migration0To1;
use crate::migration::migration_1_2::Migration1To2;
use anyhow::{bail, Context};
use serde_json::{json, Value};

mod migration_0_1;
mod migration_1_2;

pub trait Migration {
    fn migrate(old: &Value) -> anyhow::Result<Value>;
}

pub const CURRENT_FORMAT: u64 = 2;

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

    if format == CURRENT_FORMAT {
        return Ok(chart.clone());
    }

    let new_chart = match format {
        0 => Migration0To1::migrate(chart)?,
        1 => Migration1To2::migrate(chart)?,
        _ => bail!("Unsupported chart format {}", format),
    };

    let new_format = get_format(&new_chart)?;

    if new_format < CURRENT_FORMAT {
        migrate(&new_chart)
    } else {
        Ok(new_chart)
    }
}
