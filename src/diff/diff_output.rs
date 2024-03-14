use crate::diff::sequence::query::output::SequenceDiffOutput;
use crate::diff::table::query::output::TableDiffOutput;

/// The output of a diff operation.
/// This is used in order to have a common format for
/// both table and sequence diff outputs.
pub enum DiffOutput {
    TableDiff(TableDiffOutput),
    SequenceDiff(SequenceDiffOutput),
}
