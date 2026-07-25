use crate::diff::diff_payload::DiffPayload;
use crate::diff::table::query::input::{
    QueryHashDataInput, QueryPrimaryKeysInput, QueryTableCountInput, QueryTableNamesInput,
};
use crate::diff::table::query::output::{TableCountDiff, TableDiffOutput, TableSource};

use crate::diff::table::query::table_query_executor::{
    TableDualSourceQueryExecutor, TableSingleSourceQueryExecutor,
};
use crate::diff::table::query::table_types::{
    TableName, TableOffset, TablePosition, TablePrimaryKeys,
};
use anyhow::Result;
use colored::Colorize;
use futures::stream::{FuturesUnordered, StreamExt};
use tracing::{debug, info};

use crate::diff::diff_output::DiffOutput;
use crate::diff::types::SchemaName;
use std::time::Instant;

pub struct TableDiffer<TQE: TableSingleSourceQueryExecutor, DTQE: TableDualSourceQueryExecutor> {
    single_table_query_executor: TQE,
    dual_table_query_executor: DTQE,
}

impl<TQE: TableSingleSourceQueryExecutor, DTQE: TableDualSourceQueryExecutor>
    TableDiffer<TQE, DTQE>
{
    pub fn new(single_table_query_executor: TQE, dual_table_query_executor: DTQE) -> Self {
        Self {
            single_table_query_executor,
            dual_table_query_executor,
        }
    }

    pub async fn diff_all_table_data(&self, diff_payload: &DiffPayload) -> Result<Vec<DiffOutput>> {
        info!("{}", "Starting data analysis…".yellow().bold());

        let mut tables = self.get_all_tables(diff_payload).await?;

        tables.sort_by_key(|s| s.to_lowercase());

        info!(
            "{}",
            format!("Total tables for row count check: {}", tables.len())
                .bright_blue()
                .bold()
        );

        let sorted_tables = tables;

        let schema_name = diff_payload.schema_name().to_string();
        let only_count = diff_payload.only_count();
        let chunk_size = diff_payload.chunk_size();
        let start_position = diff_payload.start_position();

        // Use FuturesUnordered for bounded concurrent processing
        let mut futures: FuturesUnordered<_> = sorted_tables
            .iter()
            .map(|table_name| {
                let schema_name = schema_name.clone();
                let table_name = table_name.clone();
                async move {
                    let start = Instant::now();

                    // Start loading counts for table from both DBs
                    let query_count_input = QueryTableCountInput::new(
                        SchemaName::new(schema_name.clone()),
                        TableName::new(table_name.clone()),
                    );

                    let table_counts_start = Instant::now();
                    let (first_result, second_result) = self
                        .dual_table_query_executor
                        .query_table_count(query_count_input)
                        .await;

                    let table_counts_elapsed = table_counts_start.elapsed();
                    debug!(
                        "Table counts for {} loaded in: {}ms",
                        table_name,
                        table_counts_elapsed.as_millis()
                    );

                    info!(
                        "{}",
                        format!("Analyzing table: {}", table_name).yellow().bold()
                    );

                    // Start counts comparison
                    let table_diff_result =
                        Self::extract_result(&table_name, first_result, second_result);

                    let elapsed = start.elapsed();
                    debug!(
                        "{}",
                        format!("Table analysis completed in: {}ms", elapsed.as_millis())
                    );

                    debug!("##############################################");

                    // If we only care about counts, return the result
                    if only_count {
                        return table_diff_result;
                    }

                    // If the diff result permits us to skip data comparison, return the result
                    if table_diff_result.skip_table_diff() {
                        return table_diff_result;
                    }

                    let query_primary_keys_input = QueryPrimaryKeysInput::new(table_name.clone());

                    let primary_keys = self
                        .single_table_query_executor
                        .query_primary_keys(query_primary_keys_input)
                        .await;

                    // If no primary keys found, return the result
                    if primary_keys.is_empty() {
                        return TableDiffOutput::NoPrimaryKeyFound(table_name.clone());
                    }

                    // Prepare the primary keys for the table
                    // Will be used for query ordering when hashing data
                    let primary_keys = primary_keys.as_slice().join(",");

                    let total_rows = match &table_diff_result {
                        TableDiffOutput::NoCountDiff(_, rows) => *rows,
                        _ => {
                            panic!("Unexpected table diff result")
                        }
                    };

                    let schema_name_owned = SchemaName::new(schema_name);
                    let query_table_name = TableName::new(table_name.clone());
                    let table_offset = TableOffset::new(chunk_size);
                    let table_primary_keys = TablePrimaryKeys::new(primary_keys);

                    let hash_start = Instant::now();

                    // Start data comparison
                    let mut position = start_position;
                    while position <= total_rows {
                        let input = QueryHashDataInput::new(
                            schema_name_owned.clone(),
                            query_table_name.clone(),
                            table_primary_keys.clone(),
                            TablePosition::new(position),
                            table_offset.clone(),
                        );

                        let hash_fetch_start = Instant::now();
                        let (first_hash, second_hash) =
                            self.dual_table_query_executor.query_hash_data(input).await;
                        let hash_fetch_elapsed = hash_fetch_start.elapsed();
                        debug!(
                            "Hashes for {} loaded in: {}ms",
                            query_table_name.name(),
                            hash_fetch_elapsed.as_millis()
                        );

                        if first_hash != second_hash {
                            let elapsed = hash_start.elapsed();
                            return TableDiffOutput::DataDiffWithDuration(
                                query_table_name.name().to_string(),
                                position,
                                position + chunk_size,
                                elapsed,
                            );
                        }

                        position += chunk_size;
                    }

                    let elapsed = hash_start.elapsed();

                    TableDiffOutput::NoDiffWithDuration(table_name.clone(), elapsed)
                }
            })
            .collect();

        info!(
            "{}",
            "Waiting for table analysis to complete…".yellow().bold()
        );

        let start = Instant::now();
        let mut analysed_tables = Vec::with_capacity(futures.len());
        while let Some(result) = futures.next().await {
            analysed_tables.push(result);
        }
        let elapsed = start.elapsed();
        info!(
            "{}",
            format!(
                "Total table analysis completed in: {}ms",
                elapsed.as_millis()
            )
            .yellow()
            .bold(),
        );

        info!("##############################################");
        info!("{}", "Table analysis results 👇".bright_magenta().bold());

        for table_diff_result in &analysed_tables {
            info!("{}", table_diff_result.to_string());
        }

        info!("##############################################");

        Ok(analysed_tables
            .into_iter()
            .map(|diff| diff.into())
            .collect())
    }

    pub async fn get_all_tables(&self, diff_payload: &DiffPayload) -> Result<Vec<String>> {
        let input = QueryTableNamesInput::new(
            SchemaName::new(diff_payload.schema_name().to_string()),
            diff_payload.included_tables(),
            diff_payload.excluded_tables(),
        );

        let tables = self
            .single_table_query_executor
            .query_table_names(input)
            .await;

        Ok(tables)
    }

    fn extract_result(
        table_name: &str,
        first_result: Result<i64>,
        second_result: Result<i64>,
    ) -> TableDiffOutput {
        match (first_result, second_result) {
            (Ok(first_total_rows), Ok(second_total_rows)) => {
                if first_total_rows != second_total_rows {
                    TableDiffOutput::Diff(
                        table_name.to_owned(),
                        TableCountDiff::new(first_total_rows, second_total_rows),
                    )
                } else {
                    TableDiffOutput::NoCountDiff(table_name.to_owned(), first_total_rows)
                }
            }
            (Err(_e), _) => TableDiffOutput::NotExists(table_name.to_owned(), TableSource::First),
            (_, Err(_e)) => TableDiffOutput::NotExists(table_name.to_owned(), TableSource::Second),
        }
    }
}
