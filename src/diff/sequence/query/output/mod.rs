use crate::diff::diff_output::DiffOutput;
use crate::diff::types::DiffOutputMarker;
use colored::{ColoredString, Colorize};
use std::fmt::Display;

/// Represents the source of a sequence.
#[derive(Clone)]
pub enum SequenceSource {
    First,
    Second,
}

impl Display for SequenceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::Second => write!(f, "second"),
        }
    }
}

/// Represents the difference in count between two sequences.
#[derive(Clone)]
pub struct SequenceCountDiff(i64, i64);

impl SequenceCountDiff {
    /// Creates a new `SequenceCountDiff` instance with the given counts.
    pub fn new(first: i64, second: i64) -> Self {
        Self(first, second)
    }

    /// Returns the count of the first sequence.
    pub fn first(&self) -> i64 {
        self.0
    }

    /// Returns the count of the second sequence.
    pub fn second(&self) -> i64 {
        self.1
    }
}

#[derive(Clone)]
/// Represents the output of a sequence difference.
pub enum SequenceDiffOutput {
    /// Indicates that there is no difference between the sequences.
    NoDiff(String),
    /// Indicates that a sequence does not exist in a specific source.
    NotExists(String, SequenceSource),
    /// Indicates a difference in count between the sequences.
    Diff(String, SequenceCountDiff),
}

impl SequenceDiffOutput {
    /// Converts the `SequenceDiffOutput` to a colored string representation.
    pub fn to_string(&self) -> ColoredString {
        match self {
            Self::NoDiff(sequence) => format!("{sequence} - No difference\n").green().bold(),
            Self::NotExists(sequence, source) => {
                format!("{sequence} - Does not exist in {source}\n")
                    .red()
                    .bold()
                    .underline()
            }
            Self::Diff(sequence, diffs) => format!(
                "Difference in sequence:{} - First: {}, Second: {}\n",
                sequence,
                diffs.first(),
                diffs.second()
            )
            .red()
            .bold()
            .underline(),
        }
    }
}

impl DiffOutputMarker for SequenceDiffOutput {
    fn convert(self) -> DiffOutput {
        DiffOutput::SequenceDiff(self.clone())
    }
}

impl From<SequenceDiffOutput> for DiffOutput {
    fn from(val: SequenceDiffOutput) -> Self {
        DiffOutput::SequenceDiff(val)
    }
}
