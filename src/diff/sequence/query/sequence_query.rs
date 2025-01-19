use crate::diff::sequence::query::sequence_types::SequenceName;
use crate::diff::types::SchemaName;
use std::fmt::{Display, Formatter};

/// Represents a query for retrieving information about sequences.
pub enum SequenceQuery {
    /// Retrieves the last value of a specific sequence.
    LastValue(SchemaName, SequenceName),
    /// Retrieves all sequences in the database.
    AllSequences(SchemaName),
}

impl Display for SequenceQuery {
    /// Formats the `SequenceQuery` as a string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LastValue(schema_name, sequence_name) => {
                write!(
                    f,
                    "SELECT last_value FROM {}.\"{}\";",
                    schema_name.name(),
                    sequence_name.name()
                )
            }
            SequenceQuery::AllSequences(schema_name) => {
                write!(
                    f,
                    r#"
                    SELECT sequence_name
                    FROM information_schema.sequences
                    WHERE sequence_schema = '{}';
                    "#,
                    schema_name.name()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::internal::tests::sanitize_raw_string;

    impl From<SequenceQuery> for String {
        fn from(value: SequenceQuery) -> Self {
            value.to_string()
        }
    }

    #[test]
    fn test_sequence_last_value_query() {
        let schema_name = SchemaName::new("test_schema");
        let sequence_name = SequenceName::new("test_sequence");
        let last_value_query = SequenceQuery::LastValue(schema_name, sequence_name);

        assert_eq!(
            last_value_query.to_string(),
            "SELECT last_value FROM test_schema.test_sequence;"
        );
    }

    #[test]
    fn test_all_sequences_query() {
        let schema_name = SchemaName::new("test_schema");
        let all_sequences_query = SequenceQuery::AllSequences(schema_name);

        assert_eq!(
            sanitize_raw_string(all_sequences_query),
            "SELECT sequence_name FROM information_schema.sequences WHERE sequence_schema = 'test_schema';"
        );
    }
}
