#[cfg(test)]
mod tests {
    use crate::diff::diff_output::DiffOutput;
    use crate::diff::diff_payload::DiffPayload;
    use crate::diff::table::query::output::TableDiffOutput;
    use crate::diff::table::query::table_query_executor::{
        MockTableDualSourceQueryExecutor, MockTableSingleSourceQueryExecutor,
    };
    use crate::diff::table::table_differ::TableDiffer;

    const EMPTY_STRING_VEC: Vec<String> = Vec::new();

    #[tokio::test]
    async fn test_get_all_tables_from_table_differ() {
        let mut single_source_query_executor = MockTableSingleSourceQueryExecutor::new();
        let dual_source_query_executor = MockTableDualSourceQueryExecutor::new();

        single_source_query_executor
            .expect_query_table_names()
            .times(1)
            .returning(|_| vec!["table1".to_string(), "table2".to_string()]);

        let table_differ =
            TableDiffer::new(single_source_query_executor, dual_source_query_executor);

        let diff_payload = DiffPayload::new(
            "first_db",
            "second_db",
            false,
            false,
            false,
            10000,
            0,
            10,
            vec!["table1", "table2"],
            EMPTY_STRING_VEC,
            "schema_name",
            false,
            false,
        );

        let tables = table_differ.get_all_tables(&diff_payload).await.unwrap();

        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0], "table1");
        assert_eq!(tables[1], "table2");
    }

    #[tokio::test]
    async fn test_not_diff_table_data_from_table_differ_when_different_counts() {
        let mut single_source_query_executor = MockTableSingleSourceQueryExecutor::new();
        let mut dual_source_query_executor = MockTableDualSourceQueryExecutor::new();

        single_source_query_executor
            .expect_query_table_names()
            .times(1)
            .returning(|_| vec!["table1".to_string()]);

        dual_source_query_executor
            .expect_query_table_count()
            .times(1)
            .returning(|_| (Ok(2), Ok(1)));

        single_source_query_executor
            .expect_query_primary_keys()
            .times(0);

        dual_source_query_executor.expect_query_hash_data().times(0);

        let table_differ =
            TableDiffer::new(single_source_query_executor, dual_source_query_executor);

        let diff_payload = DiffPayload::new(
            "first_db",
            "second_db",
            false,
            false,
            false,
            10000,
            0,
            10,
            vec!["table1", "table2"],
            EMPTY_STRING_VEC,
            "schema_name",
            false,
            false,
        );

        let diff_output = table_differ
            .diff_all_table_data(&diff_payload)
            .await
            .unwrap();

        assert_eq!(diff_output.len(), 1);

        let actual = diff_output.first().unwrap();

        assert!(matches!(actual, DiffOutput::TableDiff(_)));
        match actual {
            DiffOutput::TableDiff(table_diff_output) => match table_diff_output {
                TableDiffOutput::Diff(table_name, table_count_diff) => {
                    assert_eq!("table1", table_name);
                    assert_eq!(2, table_count_diff.first());
                    assert_eq!(1, table_count_diff.second());
                }
                _ => panic!("Expected TableDiffOutput::Diff"),
            },
            _ => panic!("Expected DiffOutput::TableDiff"),
        }
    }

    #[tokio::test]
    async fn test_diff_all_table_data_from_table_differ_when_same_counts() {
        let mut single_source_query_executor = MockTableSingleSourceQueryExecutor::new();
        let mut dual_source_query_executor = MockTableDualSourceQueryExecutor::new();

        single_source_query_executor
            .expect_query_table_names()
            .times(1)
            .returning(|_| vec!["table1".to_string()]);

        dual_source_query_executor
            .expect_query_table_count()
            .times(1)
            .returning(|_| (Ok(1), Ok(1)));

        single_source_query_executor
            .expect_query_primary_keys()
            .times(1)
            .returning(|_| vec!["id".to_string()]);

        dual_source_query_executor
            .expect_query_hash_data()
            .times(1)
            .returning(|_| ("hash1".to_string(), "hash2".to_string()));

        let table_differ =
            TableDiffer::new(single_source_query_executor, dual_source_query_executor);

        let diff_payload = DiffPayload::new(
            "first_db",
            "second_db",
            false,
            false,
            false,
            10000,
            0,
            10,
            vec!["table1", "table2"],
            EMPTY_STRING_VEC,
            "schema_name",
            false,
            false,
        );

        let diff_output = table_differ
            .diff_all_table_data(&diff_payload)
            .await
            .unwrap();

        assert_eq!(diff_output.len(), 1);

        let actual = diff_output.first().unwrap();

        assert!(matches!(actual, DiffOutput::TableDiff(_)));
        match actual {
            DiffOutput::TableDiff(diff_output) => match diff_output {
                TableDiffOutput::DataDiffWithDuration(table_name, position, offset, _) => {
                    assert_eq!("table1", table_name);
                    assert_eq!(0, *position);
                    assert_eq!(10000, *offset);
                }
                _ => panic!("Expected TableDiffOutput::DataDiffWithDuration"),
            },
            _ => panic!("Expected DiffOutput::TableDiff"),
        }
    }
}
