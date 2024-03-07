use crate::diff::db_clients::DBClients;
use anyhow::Result;
use colored::Colorize;

use crate::diff::diff_output::DiffOutput;
use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;

use crate::diff::diff_payload::DiffPayload;
use crate::diff::sequence::query::sequence_query_executor::{
    SequenceDualSourceQueryExecutorImpl, SequenceSingleSourceQueryExecutorImpl,
};

use crate::diff::sequence::sequence_differ::SequenceDiffer;
use crate::diff::table::query::table_query_executor::{
    TableDualSourceQueryExecutorImpl, TableSingleSourceQueryExecutorImpl,
};

use crate::diff::table::table_differ::TableDiffer;

/// The `Differ` struct represents a database differ.
///
/// It provides a method `diff_dbs` that performs the diffing operation between two databases.
pub struct Differ;

impl Differ {
    pub async fn diff_dbs(diff_payload: DiffPayload) -> Result<Vec<DiffOutput>> {
        info!("{}", "Initiating DB diffing…".bold().blue());

        let first_db_pool = PgPoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    conn.execute("SET application_name = 'rust-pgdatadiff';")
                        .await?;
                    Ok(())
                })
            })
            .max_connections(diff_payload.max_connections())
            .connect(diff_payload.first_db())
            .await
            .expect("Failed to connect to first DB");

        info!("{}", "Connected to first DB".magenta().bold());

        let second_db_pool = PgPoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    conn.execute("SET application_name = 'rust-pgdatadiff';")
                        .await?;
                    Ok(())
                })
            })
            .max_connections(diff_payload.max_connections())
            .connect(diff_payload.second_db())
            .await
            .expect("Failed to connect to second DB");

        info!("{}", "Connected to second DB".magenta().bold());

        let db_clients = DBClients::new(first_db_pool, second_db_pool);

        info!("{}", "Going for diff…".green().bold());

        // Create a single source query executor for tables
        let single_table_query_executor =
            TableSingleSourceQueryExecutorImpl::new(db_clients.first_db_pool());

        // Create a dual source query executor for tables
        let dual_source_table_query_executor = TableDualSourceQueryExecutorImpl::new(
            db_clients.first_db_pool(),
            db_clients.second_db_pool(),
        );

        // Create a table differ
        let table_differ = TableDiffer::new(
            single_table_query_executor,
            dual_source_table_query_executor,
        );

        // Create a single source query executor for sequences
        let single_sequence_query_executor =
            SequenceSingleSourceQueryExecutorImpl::new(db_clients.first_db_pool());

        // Create a dual source query executor for sequences
        let dual_source_sequence_query_executor = SequenceDualSourceQueryExecutorImpl::new(
            db_clients.first_db_pool(),
            db_clients.second_db_pool(),
        );

        // Create a sequence differ
        let sequence_differ = SequenceDiffer::new(
            single_sequence_query_executor,
            dual_source_sequence_query_executor,
        );

        // Prepare diff output
        let diff_output = if diff_payload.only_tables() {
            // Load only tables diff
            let original_table_diff = table_differ.diff_all_table_data(&diff_payload).await?;
            original_table_diff.into_iter().collect::<Vec<DiffOutput>>()
        } else if diff_payload.only_sequences() {
            // Load only sequences diff
            let original_sequence_diff = sequence_differ
                .diff_all_sequences(diff_payload.schema_name().into())
                .await?;
            original_sequence_diff
                .into_iter()
                .collect::<Vec<DiffOutput>>()
        } else {
            // Load both tables and sequences diff
            let original_sequence_diff =
                sequence_differ.diff_all_sequences(diff_payload.schema_name().into());

            let original_table_diff = table_differ.diff_all_table_data(&diff_payload);

            let (table_diff, sequence_diff) =
                futures::future::join(original_table_diff, original_sequence_diff).await;

            let table_diff: Vec<DiffOutput> = table_diff.unwrap();
            let sequence_diff: Vec<DiffOutput> = sequence_diff.unwrap();

            table_diff
                .into_iter()
                .chain(sequence_diff.into_iter())
                .collect::<Vec<DiffOutput>>()
        };

        Ok(diff_output)
    }
}
