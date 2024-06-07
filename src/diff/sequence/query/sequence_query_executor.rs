/// This module contains the implementation of query executors for sequence-related operations.
/// It provides traits and structs for executing queries on a single data source and on dual data sources.
/// The single data source executor is responsible for querying sequence names.
/// The dual data source executor is responsible for querying sequence last values.
/// Both executors use the `sqlx` crate for interacting with the database.
///
/// # Examples
///
/// ```no_run
/// use rust_pgdatadiff::diff::sequence::query::sequence_query_executor::SequenceSingleSourceQueryExecutorImpl;
/// use rust_pgdatadiff::diff::sequence::query::sequence_query_executor::SequenceSingleSourceQueryExecutor;
/// use rust_pgdatadiff::diff::sequence::query::input::QueryAllSequencesInput;
/// use rust_pgdatadiff::diff::types::SchemaName;
/// use rust_pgdatadiff::diff::sequence::query::sequence_query_executor::SequenceDualSourceQueryExecutorImpl;
/// use rust_pgdatadiff::diff::sequence::query::sequence_query_executor::SequenceDualSourceQueryExecutor;
/// use rust_pgdatadiff::diff::sequence::query::sequence_types::SequenceName;
/// use rust_pgdatadiff::diff::sequence::query::input::QueryLastValuesInput;
///
/// #[tokio::main]
/// async fn main() {
///
///     let mut cfg = Config::new();
///     cfg.url = Some(String::from("postgres://user:password@localhost:5432/database"));
///
///     let db_pool: Pool = cfg
///         .create_pool(Some(Runtime::Tokio1), NoTls)
///         .unwrap();
///
///     // Create a single data source executor
///     let single_source_executor = SequenceSingleSourceQueryExecutorImpl::new(db_pool);
///
///     // Query sequence names
///     let schema_name = SchemaName::new("public".to_string());
///     let table_names = single_source_executor
///         .query_sequence_names(QueryAllSequencesInput::new(schema_name))
///         .await;
///
///     // Create a dual data source executor
///     let mut first_cfg = Config::new();
///     first_cfg.url = Some(String::from("postgres://user:password@localhost:5432/database"));
///
///     let mut second_cfg = Config::new();
///     second_cfg.url = Some(String::from("postgres://user:password@localhost:5432/database2"));
///
///     let first_db_pool: Pool = first_cfg
///         .create_pool(Some(Runtime::Tokio1), NoTls)
///         .unwrap();
///
///     let second_db_pool: Pool = second_cfg
///         .create_pool(Some(Runtime::Tokio1), NoTls)
///         .unwrap();
///
///     let dual_source_executor = SequenceDualSourceQueryExecutorImpl::new(first_db_pool, second_db_pool);
///
///     // Query sequence last values
///     let sequence_name = SequenceName::new("public".to_string());
///     let schema_name = SchemaName::new("public".to_string());
///     let (first_count, second_count) = dual_source_executor
///         .query_sequence_last_values(QueryLastValuesInput::new(schema_name, sequence_name))
///         .await;
/// }
/// ```
use crate::diff::sequence::query::input::{QueryAllSequencesInput, QueryLastValuesInput};
use crate::diff::sequence::query::sequence_query::SequenceQuery;

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SequenceSingleSourceQueryExecutor {
    /// Queries the sequence names from the database.
    ///
    /// # Returns
    ///
    /// A vector of sequence names.
    async fn query_sequence_names(&self, input: QueryAllSequencesInput) -> Vec<String>;
}

pub struct SequenceSingleSourceQueryExecutorImpl {
    db_pool: Pool,
}

impl SequenceSingleSourceQueryExecutorImpl {
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl SequenceSingleSourceQueryExecutor for SequenceSingleSourceQueryExecutorImpl {
    async fn query_sequence_names(&self, input: QueryAllSequencesInput) -> Vec<String> {
        // Clone the database client
        let client = self.db_pool.get().await.unwrap();

        let schema_name = input.schema_name();
        let sequence_query = SequenceQuery::AllSequences(schema_name);

        let query_binding = sequence_query.to_string();

        client
            .query(&query_binding, &[])
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get("sequence_name"))
            .collect::<Vec<String>>()
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SequenceDualSourceQueryExecutor {
    /// Executes a query to retrieve the last value of a sequence.
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters for the query.
    ///
    /// # Returns
    ///
    /// A tuple containing the result of the query as a `Result<i64>`.
    async fn query_sequence_last_values(
        &self,
        input: QueryLastValuesInput,
    ) -> (Result<i64>, Result<i64>);
}

pub struct SequenceDualSourceQueryExecutorImpl {
    first_db_pool: Pool,
    second_db_pool: Pool,
}

impl SequenceDualSourceQueryExecutorImpl {
    pub fn new(first_db_pool: Pool, second_db_pool: Pool) -> Self {
        Self {
            first_db_pool,
            second_db_pool,
        }
    }
}

#[async_trait]
impl SequenceDualSourceQueryExecutor for SequenceDualSourceQueryExecutorImpl {
    async fn query_sequence_last_values(
        &self,
        input: QueryLastValuesInput,
    ) -> (Result<i64>, Result<i64>) {
        // Clone the database clients
        let first_client = self.first_db_pool.get().await.unwrap();
        let second_client = self.second_db_pool.get().await.unwrap();

        let sequence_query = SequenceQuery::LastValue(
            input.schema_name().to_owned(),
            input.sequence_name().to_owned(),
        );

        let query_binding = sequence_query.to_string();

        let first_result = first_client.query_one(&query_binding, &[]);
        let second_result = second_client.query_one(&query_binding, &[]);

        let (first_result, second_result) =
            futures::future::join(first_result, second_result).await;

        let first_count: Result<i64> = match first_result {
            Ok(pg_row) => Ok(pg_row.try_get("last_value").unwrap()),
            Err(e) => {
                error!("Error while fetching first sequence: {}", e);
                Err(anyhow::anyhow!("Failed to fetch count for first sequence"))
            }
        };

        let second_count: Result<i64> = match second_result {
            Ok(pg_row) => Ok(pg_row.try_get("last_value").unwrap()),
            Err(e) => {
                error!("Error while fetching second sequence: {}", e);
                Err(anyhow::anyhow!("Failed to fetch count for second sequence"))
            }
        };

        (first_count, second_count)
    }
}
