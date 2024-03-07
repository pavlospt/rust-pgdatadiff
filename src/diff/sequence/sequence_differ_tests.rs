#[cfg(test)]
mod tests {
    use crate::diff::diff_output::DiffOutput;
    use crate::diff::sequence::query::output::SequenceDiffOutput;
    use crate::diff::sequence::query::sequence_query_executor::{
        MockSequenceDualSourceQueryExecutor, MockSequenceSingleSourceQueryExecutor,
    };
    use crate::diff::sequence::sequence_differ::SequenceDiffer;

    #[tokio::test]
    async fn test_get_all_sequences() {
        let mut single_source_query_executor = MockSequenceSingleSourceQueryExecutor::new();
        let dual_source_query_executor = MockSequenceDualSourceQueryExecutor::new();

        single_source_query_executor
            .expect_query_sequence_names()
            .times(1)
            .returning(|_| vec!["sequence1".to_string(), "sequence2".to_string()]);

        let sequence_differ =
            SequenceDiffer::new(single_source_query_executor, dual_source_query_executor);

        let sequences = sequence_differ
            .get_all_sequences("public".to_string())
            .await
            .unwrap();

        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0], "sequence1");
        assert_eq!(sequences[1], "sequence2");
    }

    #[tokio::test]
    async fn test_diff_all_sequences() {
        let mut single_source_query_executor = MockSequenceSingleSourceQueryExecutor::new();
        let mut dual_source_query_executor = MockSequenceDualSourceQueryExecutor::new();

        single_source_query_executor
            .expect_query_sequence_names()
            .times(1)
            .returning(|_| vec!["sequence1".to_string()]);

        dual_source_query_executor
            .expect_query_sequence_last_values()
            .times(1)
            .returning(|_| (Ok(2), Ok(1)));

        let sequence_differ =
            SequenceDiffer::new(single_source_query_executor, dual_source_query_executor);

        let sequences = sequence_differ
            .diff_all_sequences("public".to_string())
            .await
            .unwrap();
        let actual = sequences.first().unwrap();

        assert_eq!(sequences.len(), 1);
        assert!(matches!(actual, DiffOutput::SequenceDiff(_)));
        match actual {
            DiffOutput::SequenceDiff(sequence_diff_output) => match sequence_diff_output {
                SequenceDiffOutput::Diff(sequence_name, sequence_count_diff) => {
                    assert_eq!("sequence1", sequence_name);
                    assert_eq!(1, sequence_count_diff.second());
                    assert_eq!(2, sequence_count_diff.first());
                }
                _ => panic!("Expected Diff"),
            },
            _ => panic!("Expected SequenceDiff"),
        }
    }
}
