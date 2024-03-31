/// This module contains the implementation of query executors for table-related operations.
/// It provides traits and structs for executing queries on a single data source and on dual data sources.
/// The single data source executor is responsible for querying table names and primary keys.
/// The dual data source executor is responsible for querying table counts and hash data.
/// Both executors use the `sqlx` crate for interacting with the database.
///
/// # Examples
///
/// ```no_run
/// use sqlx::postgres::PgPool;
/// use rust_pgdatadiff::diff::table::query::table_query_executor::{
///     TableSingleSourceQueryExecutor, TableSingleSourceQueryExecutorImpl,
///     TableDualSourceQueryExecutor, TableDualSourceQueryExecutorImpl,
/// };
/// use rust_pgdatadiff::diff::table::query::input::{QueryHashDataInput, QueryPrimaryKeysInput, QueryTableCountInput, QueryTableNamesInput};///
/// use rust_pgdatadiff::diff::table::query::table_types::{TableName, TableOffset, TablePosition, TablePrimaryKeys};
/// use rust_pgdatadiff::diff::types::SchemaName;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a single data source executor
///     let db_client: PgPool = PgPool::connect("postgres://user:password@localhost:5432/database")
///         .await
///         .unwrap();
///     let single_source_executor = TableSingleSourceQueryExecutorImpl::new(db_client);
///
///     // Query table names
///     let schema_name = SchemaName::new("public".to_string());
///     let included_tables = vec!["table1", "table2"];
///     let excluded_tables: Vec<String> = vec![];
///     let table_names = single_source_executor
///         .query_table_names(QueryTableNamesInput::new(schema_name, included_tables, excluded_tables))
///         .await;
///
///     // Query primary keys
///     let primary_keys = single_source_executor
///         .query_primary_keys(QueryPrimaryKeysInput::new("table1".to_string()))
///         .await;
///
///     // Create a dual data source executor
///     let first_db_client: PgPool = PgPool::connect("postgres://user:password@localhost:5432/database1")
///         .await
///         .unwrap();
///     let second_db_client: PgPool = PgPool::connect("postgres://user:password@localhost:5432/database2")
///         .await
///         .unwrap();
///     let dual_source_executor = TableDualSourceQueryExecutorImpl::new(first_db_client, second_db_client);
///
///     // Query table counts
///     let schema_name = SchemaName::new("public");
///     let table_name = TableName::new("table1");
///     let (first_count, second_count) = dual_source_executor
///         .query_table_count(QueryTableCountInput::new(schema_name, table_name))
///         .await;
///
///     // Query hash data
///     let schema_name = SchemaName::new("public");
///     let table_name = TableName::new("table1");
///     let primary_keys = TablePrimaryKeys::new("id");
///     let table_position = TablePosition::new(0);
///     let table_offset = TableOffset::new(100);
///     let (first_hash, second_hash) = dual_source_executor
///         .query_hash_data(QueryHashDataInput::new(schema_name, table_name, primary_keys, table_position, table_offset))
///         .await;
/// }
/// ```
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};

use crate::diff::table::query::input::{
    QueryHashDataInput, QueryPrimaryKeysInput, QueryTableCountInput, QueryTableNamesInput,
};
use crate::diff::table::query::table_query::TableQuery;
use crate::diff::table::query::table_types::{IncludedExcludedTables, TableName};

#[cfg(test)]
use mockall::automock;
use rayon::iter::ParallelIterator;
use rayon::prelude::{IntoParallelIterator};

#[cfg_attr(test, automock)]
#[async_trait]
/// This trait represents a query executor for a single source table.
pub trait TableSingleSourceQueryExecutor {
    /// Queries the table names from the database.
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters for the query.
    ///
    /// # Returns
    ///
    /// A vector of table names.
    async fn query_table_names(&self, input: QueryTableNamesInput) -> Vec<String>;

    /// Queries the primary keys of a table from the database.
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters for the query.
    ///
    /// # Returns
    ///
    /// A vector of primary key column names.
    async fn query_primary_keys(&self, input: QueryPrimaryKeysInput) -> Vec<String>;
}

pub struct TableSingleSourceQueryExecutorImpl {
    db_client: Pool<Postgres>,
}

impl TableSingleSourceQueryExecutorImpl {
    pub fn new(db_client: Pool<Postgres>) -> Self {
        Self { db_client }
    }
}

#[async_trait]
impl TableSingleSourceQueryExecutor for TableSingleSourceQueryExecutorImpl {
    async fn query_table_names(&self, input: QueryTableNamesInput) -> Vec<String> {
        // Clone the database client
        let pool = self.db_client.clone();

        // Prepare the query for fetching table names
        let all_tables_query = TableQuery::AllTablesForSchema(
            input.schema_name().to_owned(),
            IncludedExcludedTables::new(input.included_tables(), input.excluded_tables()),
        );

        // Fetch table names
        let query_result = sqlx::query(all_tables_query.to_string().as_str())
            .bind(input.schema_name().name())
            .fetch_all(&pool)
            .await
            .unwrap_or(vec![]);

        // Map query results to [Vec<String>]
        query_result
            .into_par_iter()
            .map(|row| row.get("table_name"))
            .collect::<Vec<String>>()
    }

    async fn query_primary_keys(&self, input: QueryPrimaryKeysInput) -> Vec<String> {
        // Clone the database client
        let pool = self.db_client.clone();

        // Prepare the query for primary keys fetching
        let find_primary_key_query =
            TableQuery::FindPrimaryKeyForTable(TableName::new(input.table_name()));

        // Fetch primary keys for the table
        let query_result = sqlx::query(find_primary_key_query.to_string().as_str())
            .fetch_all(&pool)
            .await
            .unwrap_or(vec![]);

        // Map query results to [Vec<String>]
        query_result
            .into_par_iter()
            .map(|row| row.get("attname"))
            .collect::<Vec<String>>()
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
/// This trait defines the methods for executing queries on a dual source table.
pub trait TableDualSourceQueryExecutor {
    /// Executes a query to retrieve the count of rows in a table.
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters for the query.
    ///
    /// # Returns
    ///
    /// A tuple containing the result of the query as a `Result<i64>`.
    async fn query_table_count(&self, input: QueryTableCountInput) -> (Result<i64>, Result<i64>);

    /// Executes a query to retrieve the hash data of a table.
    ///
    /// # Arguments
    ///
    /// * `input` - The input parameters for the query.
    ///
    /// # Returns
    ///
    /// A tuple containing the hash data as two `String` values.
    async fn query_hash_data(&self, input: QueryHashDataInput) -> (String, String);
}

pub struct TableDualSourceQueryExecutorImpl {
    first_db_client: Pool<Postgres>,
    second_db_client: Pool<Postgres>,
}

impl TableDualSourceQueryExecutorImpl {
    pub fn new(first_db_client: Pool<Postgres>, second_db_client: Pool<Postgres>) -> Self {
        Self {
            first_db_client,
            second_db_client,
        }
    }
}

#[async_trait]
impl TableDualSourceQueryExecutor for TableDualSourceQueryExecutorImpl {
    async fn query_table_count(&self, input: QueryTableCountInput) -> (Result<i64>, Result<i64>) {
        // Clone the database clients
        let first_pool = self.first_db_client.clone();
        let second_pool = self.second_db_client.clone();

        // Prepare the query for counting rows
        let count_rows_query = TableQuery::CountRowsForTable(
            input.schema_name().to_owned(),
            input.table_name().to_owned(),
        );

        let count_query_binding = count_rows_query.to_string();

        // Prepare count queries for both databases
        let first_count = sqlx::query(count_query_binding.as_str()).fetch_one(&first_pool);
        let second_count = sqlx::query(count_query_binding.as_str()).fetch_one(&second_pool);

        // Fetch counts for both databases
        let count_fetch_futures = futures::future::join_all(vec![first_count, second_count]).await;

        let first_count = count_fetch_futures.first().unwrap();
        let second_count = count_fetch_futures.get(1).unwrap();

        // Map count results to [anyhow::Result<i64>]
        let first_count: Result<i64> = match first_count {
            Ok(pg_row) => Ok(pg_row.try_get::<i64, _>("count").unwrap()),
            Err(_e) => Err(anyhow::anyhow!("Failed to fetch count for first table")),
        };

        let second_count: Result<i64> = match second_count {
            Ok(pg_row) => Ok(pg_row.try_get::<i64, _>("count").unwrap()),
            Err(_e) => Err(anyhow::anyhow!("Failed to fetch count for second table")),
        };

        (first_count, second_count)
    }

    async fn query_hash_data(&self, input: QueryHashDataInput) -> (String, String) {
        // Clone the database clients
        let first_pool = self.first_db_client.clone();
        let second_pool = self.second_db_client.clone();

        // Prepare the query for fetching data hashes
        let hash_query = TableQuery::HashQuery(
            input.schema_name(),
            input.table_name(),
            input.primary_keys(),
            input.position(),
            input.offset(),
        );

        let hash_query_binding = hash_query.to_string();

        // Prepare hash queries for both databases
        let first_hash = sqlx::query(hash_query_binding.as_str()).fetch_one(&first_pool);
        let second_hash = sqlx::query(hash_query_binding.as_str()).fetch_one(&second_pool);

        // Fetch hashes for both databases
        let hash_fetch_futures = futures::future::join_all(vec![first_hash, second_hash]).await;

        let first_hash = hash_fetch_futures.first().unwrap();
        let second_hash = hash_fetch_futures.get(1).unwrap();

        // Map hash results to [String]
        let first_hash = match first_hash {
            Ok(pg_row) => pg_row
                .try_get::<String, _>("md5")
                .unwrap_or("not_available".to_string()),
            Err(e) => e.to_string(),
        };
        let second_hash = match second_hash {
            Ok(pg_row) => pg_row
                .try_get::<String, _>("md5")
                .unwrap_or("not_available".to_string()),
            Err(e) => e.to_string(),
        };

        (first_hash, second_hash)
    }
}
