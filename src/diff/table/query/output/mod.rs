use colored::{ColoredString, Colorize};
use std::fmt::Display;

use crate::diff::diff_output::DiffOutput;
use crate::diff::types::DiffOutputMarker;
use std::time::Duration;

/// Represents the source of a table (either the first or the second).
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum TableSource {
    First,
    Second,
}

impl Display for TableSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Second => write!(f, "second"),
        }
    }
}

/// Represents the difference in table counts between two tables.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct TableCountDiff(i64, i64);

impl TableCountDiff {
    /// Creates a new `TableCountDiff` instance with the given counts.
    pub fn new(first: i64, second: i64) -> Self {
        Self(first, second)
    }

    pub fn first(&self) -> i64 {
        self.0
    }

    pub fn second(&self) -> i64 {
        self.1
    }
}

/// Represents the output of a table difference.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum TableDiffOutput {
    /// Indicates that there is no difference between the tables.
    NoCountDiff(String, i64),
    /// Indicates that there is no difference between the tables, along with the duration of the comparison.
    NoDiffWithDuration(String, Duration),
    /// Indicates that the table does not exist in a specific source.
    NotExists(String, TableSource),
    /// Indicates a difference in table counts.
    Diff(String, TableCountDiff),
    /// Indicates that no primary key was found in the table.
    NoPrimaryKeyFound(String),
    /// Indicates a difference in table data, along with the duration of the comparison.
    DataDiffWithDuration(String, i64, i64, Duration),
}

impl TableDiffOutput {
    /// Determines whether the table difference should be skipped.
    pub fn skip_table_diff(&self) -> bool {
        matches!(self, Self::Diff(_, _) | Self::NotExists(_, _))
    }

    /// Converts the table difference output to a colored string.
    pub fn to_string(&self) -> ColoredString {
        match self {
            Self::NoCountDiff(table, count) => {
                format!("{table} - No difference. Total rows: {count}")
                    .green()
                    .bold()
            }
            Self::NotExists(table, source) => format!("{table} - Does not exist in {source}")
                .red()
                .bold()
                .underline(),
            Self::Diff(table, diffs) => format!(
                "{} - First table rows: {}, Second table rows: {}",
                table,
                diffs.first(),
                diffs.second()
            )
            .red()
            .bold(),
            TableDiffOutput::NoPrimaryKeyFound(table) => {
                format!("{table} - No primary key found").red().bold()
            }
            TableDiffOutput::NoDiffWithDuration(table, duration) => {
                format!("{} - No difference in {}ms", table, duration.as_millis())
                    .green()
                    .bold()
            }
            TableDiffOutput::DataDiffWithDuration(table_name, position, offset, duration) => {
                format!(
                    "{} - Data diff between rows [{},{}] - in {}ms",
                    table_name,
                    position,
                    offset,
                    duration.as_millis()
                )
                .red()
                .bold()
            }
        }
    }
}

impl DiffOutputMarker for TableDiffOutput {
    fn convert(self) -> DiffOutput {
        DiffOutput::TableDiff(self.clone())
    }
}

impl From<TableDiffOutput> for DiffOutput {
    fn from(val: TableDiffOutput) -> Self {
        DiffOutput::TableDiff(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_table_when_needed() {
        let no_count_diff = TableDiffOutput::NoCountDiff("test".to_string(), 1000);
        let not_exists = TableDiffOutput::NotExists("test".to_string(), TableSource::First);
        let diff = TableDiffOutput::Diff("test".to_string(), TableCountDiff::new(1, 2));
        let no_primary_key = TableDiffOutput::NoPrimaryKeyFound("test".to_string());
        let no_diff_with_duration =
            TableDiffOutput::NoDiffWithDuration("test".to_string(), Duration::from_millis(1));
        let data_diff_with_duration = TableDiffOutput::DataDiffWithDuration(
            "test".to_string(),
            1,
            2,
            Duration::from_millis(1),
        );

        assert!(not_exists.skip_table_diff());
        assert!(diff.skip_table_diff());
        assert!(!no_count_diff.skip_table_diff());
        assert!(!no_primary_key.skip_table_diff());
        assert!(!no_diff_with_duration.skip_table_diff());
        assert!(!data_diff_with_duration.skip_table_diff());
    }
}
