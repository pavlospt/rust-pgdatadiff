use anyhow::Result;
use colored::Colorize;

use tracing::{debug, info};

use crate::diff::diff_output::DiffOutput;
use crate::diff::sequence::query::input::{QueryAllSequencesInput, QueryLastValuesInput};
use crate::diff::sequence::query::output::{SequenceCountDiff, SequenceDiffOutput, SequenceSource};
use tokio::time::Instant;

use crate::diff::sequence::query::sequence_query_executor::{
    SequenceDualSourceQueryExecutor, SequenceSingleSourceQueryExecutor,
};
use crate::diff::sequence::query::sequence_types::SequenceName;
use crate::diff::types::SchemaName;

pub struct SequenceDiffer<
    SQE: SequenceSingleSourceQueryExecutor,
    DSQE: SequenceDualSourceQueryExecutor,
> {
    single_sequence_query_executor: SQE,
    dual_sequence_query_executor: DSQE,
}

impl<SQE: SequenceSingleSourceQueryExecutor, DSQE: SequenceDualSourceQueryExecutor>
    SequenceDiffer<SQE, DSQE>
{
    pub fn new(single_sequence_query_executor: SQE, dual_sequence_query_executor: DSQE) -> Self {
        Self {
            single_sequence_query_executor,
            dual_sequence_query_executor,
        }
    }

    pub async fn diff_all_sequences(&self, schema_name: String) -> Result<Vec<DiffOutput>> {
        info!("{}", "Starting sequence analysis…".bold().yellow());
        let mut sequences = self.get_all_sequences(schema_name.to_owned()).await?;

        sequences.sort_by_key(|s| s.to_lowercase());

        let sorted_sequences = sequences.to_owned();

        let futures = sorted_sequences.iter().map(|sequence_name| async {
            let start = Instant::now();

            let schema_name = SchemaName::new(schema_name.to_owned());
            let sequence_name = SequenceName::new(sequence_name.to_owned());
            let input = QueryLastValuesInput::new(schema_name, sequence_name.to_owned());
            let (first_result, second_result) = self
                .dual_sequence_query_executor
                .query_sequence_last_values(input)
                .await;

            debug!(
                "{}",
                format!("Analyzing sequence: {}", &sequence_name.name())
                    .yellow()
                    .bold()
            );

            let sequence_diff_result =
                Self::extract_result(sequence_name.name(), first_result, second_result);

            let elapsed = start.elapsed();
            debug!(
                "{}",
                format!("Sequence analysis completed in: {}ms", elapsed.as_millis())
            );
            debug!("##############################################");

            sequence_diff_result
        });

        info!(
            "{}",
            "Waiting for total sequence analysis to complete…"
                .yellow()
                .bold()
        );
        let start = Instant::now();
        let sequences_analysed = futures::future::join_all(futures).await;
        let elapsed = start.elapsed();
        debug!(
            "{}",
            format!(
                "Total sequence analysis completed in: {}ms",
                elapsed.as_millis()
            )
            .yellow()
            .bold(),
        );

        for sequence_diff_result in &sequences_analysed {
            info!("{}", sequence_diff_result.to_string());
        }

        Ok(sequences_analysed
            .into_iter()
            .map(|diff| diff.into())
            .collect())
    }

    pub async fn get_all_sequences(&self, schema_name: String) -> Result<Vec<String>> {
        let input = QueryAllSequencesInput::new(SchemaName::new(schema_name));
        let query_result = self
            .single_sequence_query_executor
            .query_sequence_names(input)
            .await;
        Ok(query_result)
    }

    fn extract_result(
        sequence_name: String,
        first_result: Result<i64>,
        second_result: Result<i64>,
    ) -> SequenceDiffOutput {
        match (first_result, second_result) {
            (Ok(first_value), Ok(second_value)) => {
                if first_value != second_value {
                    SequenceDiffOutput::Diff(
                        sequence_name,
                        SequenceCountDiff::new(first_value, second_value),
                    )
                } else {
                    SequenceDiffOutput::NoDiff(sequence_name)
                }
            }
            (Err(_e), _) => SequenceDiffOutput::NotExists(sequence_name, SequenceSource::First),
            (_, Err(_e)) => SequenceDiffOutput::NotExists(sequence_name, SequenceSource::Second),
        }
    }
}
