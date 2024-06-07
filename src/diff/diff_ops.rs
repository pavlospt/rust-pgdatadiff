use crate::diff::db_clients::DBClients;
use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::tokio_postgres::NoTls;
use deadpool_postgres::{Config, ManagerConfig, PoolConfig, RecyclingMethod, Runtime};

use crate::diff::diff_output::DiffOutput;
use tracing::info;

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

        let tls_connector = if diff_payload.any_accept_invalid_certs() {
            use native_tls::TlsConnector;
            use postgres_native_tls::MakeTlsConnector;

            let tls_connector = TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();

            Some(MakeTlsConnector::new(tls_connector))
        } else {
            None
        };

        let mut first_cfg = Config::new();
        first_cfg.url = Some(diff_payload.first_db().to_string());
        first_cfg.application_name = Some(String::from("rust-pgdatadiff"));
        first_cfg.pool = Some(PoolConfig::new(diff_payload.max_connections() as usize));
        first_cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let mut second_cfg = Config::new();
        second_cfg.url = Some(diff_payload.second_db().to_string());
        second_cfg.application_name = Some(String::from("rust-pgdatadiff"));
        second_cfg.pool = Some(PoolConfig::new(diff_payload.max_connections() as usize));
        second_cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        info!("{}", "Connected to first DB".magenta().bold());
        let first_db_pool = if diff_payload.accept_invalid_certs_first_db() {
            first_cfg
                .create_pool(Some(Runtime::Tokio1), tls_connector.clone().unwrap())
                .unwrap()
        } else {
            first_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
        };

        info!("{}", "Connected to second DB".magenta().bold());
        let second_db_pool = if diff_payload.accept_invalid_certs_second_db() {
            second_cfg
                .create_pool(Some(Runtime::Tokio1), tls_connector.unwrap())
                .unwrap()
        } else {
            second_cfg
                .create_pool(Some(Runtime::Tokio1), NoTls)
                .unwrap()
        };

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
