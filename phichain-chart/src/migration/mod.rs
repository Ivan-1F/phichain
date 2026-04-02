use anyhow::{bail, Context};
use serde_json::{json, Value};

pub trait Migration {
    fn migrate(old: &Value) -> anyhow::Result<Value>;
}

#[cfg(test)]
pub(crate) mod test_utils;

/// Visit every line in a chart, including all nested child lines, in depth-first order.
///
/// This is intended for format migrations that need to apply the same transformation
/// to both root lines and their descendants.
pub(super) fn for_each_line_recursive(
    chart: &mut Value,
    f: &mut impl FnMut(&mut Value) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    fn visit_lines(
        lines: &mut [Value],
        f: &mut impl FnMut(&mut Value) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for line in lines {
            f(line)?;

            if let Some(children) = line.get_mut("children").and_then(|v| v.as_array_mut()) {
                visit_lines(children, f)?;
            }
        }

        Ok(())
    }

    let lines = chart
        .get_mut("lines")
        .context("Failed to get lines")?
        .as_array_mut()
        .context("`lines` is not an array")?;

    visit_lines(lines, f)
}

macro_rules! define_migrations {
    ($( $from:literal => $to:literal : $mod:ident :: $type:ident ),* $(,)?) => {
        $( mod $mod; )*
        $( use $mod::$type; )*

        // CURRENT_FORMAT is the largest target version
        pub const CURRENT_FORMAT: u64 = {
            let mut max = 0u64;
            $( if $to > max { max = $to; } )*
            max
        };

        /// Run the migration for a given format version, returns None if unsupported
        fn migrate_step(format: u64, chart: &Value) -> Option<anyhow::Result<Value>> {
            match format {
                $( $from => Some(<$type as Migration>::migrate(chart)), )*
                _ => None,
            }
        }
    };
}

define_migrations! {
    0 => 1: migration_0_1::Migration0To1,
    1 => 2: migration_1_2::Migration1To2,
    2 => 3: migration_2_3::Migration2To3,
    3 => 4: migration_3_4::Migration3To4,
    4 => 5: migration_4_5::Migration4To5,
    5 => 6: migration_5_6::Migration5To6,
}

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

    let new_chart = match migrate_step(format, chart) {
        Some(result) => result?,
        None => bail!("Unsupported chart format {}", format),
    };

    let new_format = get_format(&new_chart)?;

    if new_format < CURRENT_FORMAT {
        migrate(&new_chart)
    } else {
        Ok(new_chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::PhichainChart;

    /// Migrate a v0 chart all the way to the latest format and verify
    /// that the result can be deserialized into PhichainChart
    #[test]
    fn test_full_migration_chain_and_deserialize() {
        let v0 = json!({
          "offset": 0.0,
          "bpm_list": [
            { "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }
          ],
          "lines": [
            [
              [
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": {
                    "Hold": {
                      "hold_beat": [1, 0, 1]
                    }
                  },
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                }
              ],
              [
                {
                  "kind": "X",
                  "start": 0.0,
                  "end": 0.0,
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "easing": "Linear"
                },
                {
                  "kind": "Y",
                  "start": -300.0,
                  "end": -300.0,
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "easing": {
                    "Custom": [0.5, 0.0, 0.5, 1.0]
                  }
                }
              ]
            ]
          ]
        });

        let migrated = migrate(&v0).unwrap();

        assert_eq!(
            migrated.get("format").and_then(|v| v.as_u64()),
            Some(CURRENT_FORMAT),
        );

        let chart: PhichainChart = serde_json::from_value(migrated).unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].notes.len(), 2);
        assert_eq!(chart.lines[0].events.len(), 2);
    }
}
