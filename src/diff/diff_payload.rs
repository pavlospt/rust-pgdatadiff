/// Represents a payload for performing database diffs.
pub struct DiffPayload {
    first_db: String,
    second_db: String,
    only_tables: bool,
    only_sequences: bool,
    only_count: bool,
    chunk_size: i64,
    start_position: i64,
    max_connections: i64,
    include_tables: Vec<String>,
    exclude_tables: Vec<String>,
    schema_name: String,
    accept_invalid_certs_first_db: bool,
    accept_invalid_certs_second_db: bool,
}

impl DiffPayload {
    /// Creates a new `DiffPayload` instance.
    ///
    /// # Arguments
    ///
    /// * `first_db` - The name of the first database.
    /// * `second_db` - The name of the second database.
    /// * `only_data` - A flag indicating whether to compare only data.
    /// * `only_sequences` - A flag indicating whether to compare only sequences.
    /// * `count_only` - A flag indicating whether to count differences only.
    /// * `chunk_size` - The chunk size for processing large tables.
    /// * `start_position` - The start position for the comparison.
    /// * `max_connections` - The maximum number of database connections to use.
    /// * `include_tables` - A list of tables to include in the comparison.
    /// * `exclude_tables` - A list of tables to exclude in the comparison.
    /// * `schema_name` - The name of the schema to compare.
    ///
    /// # Returns
    ///
    /// A new `DiffPayload` instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        first_db: impl Into<String>,
        second_db: impl Into<String>,
        only_tables: bool,
        only_sequences: bool,
        only_count: bool,
        chunk_size: i64,
        start_position: i64,
        max_connections: i64,
        include_tables: Vec<impl Into<String>>,
        exclude_tables: Vec<impl Into<String>>,
        schema_name: impl Into<String>,
        accept_invalid_certs_first_db: bool,
        accept_invalid_certs_second_db: bool,
    ) -> Self {
        let has_included_tables = !include_tables.is_empty();
        let has_excluded_tables = !exclude_tables.is_empty();

        if has_included_tables && has_excluded_tables {
            panic!("Cannot include and exclude tables at the same time");
        }

        Self {
            first_db: first_db.into(),
            second_db: second_db.into(),
            only_tables,
            only_sequences,
            only_count,
            chunk_size,
            start_position,
            max_connections,
            include_tables: include_tables.into_iter().map(|t| t.into()).collect(),
            exclude_tables: exclude_tables.into_iter().map(|t| t.into()).collect(),
            schema_name: schema_name.into(),
            accept_invalid_certs_first_db,
            accept_invalid_certs_second_db,
        }
    }

    pub fn first_db(&self) -> &str {
        &self.first_db
    }
    pub fn second_db(&self) -> &str {
        &self.second_db
    }
    pub fn only_tables(&self) -> bool {
        self.only_tables
    }
    pub fn only_sequences(&self) -> bool {
        self.only_sequences
    }
    pub fn only_count(&self) -> bool {
        self.only_count
    }
    pub fn chunk_size(&self) -> i64 {
        self.chunk_size
    }
    pub fn start_position(&self) -> i64 {
        self.start_position
    }
    pub fn max_connections(&self) -> u32 {
        self.max_connections as u32
    }
    pub fn included_tables(&self) -> &Vec<String> {
        &self.include_tables
    }
    pub fn excluded_tables(&self) -> &Vec<String> {
        &self.exclude_tables
    }
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }
    pub fn accept_invalid_certs_first_db(&self) -> bool {
        self.accept_invalid_certs_first_db
    }
    pub fn accept_invalid_certs_second_db(&self) -> bool {
        self.accept_invalid_certs_second_db
    }
    pub fn any_accept_invalid_certs(&self) -> bool {
        self.accept_invalid_certs_first_db || self.accept_invalid_certs_second_db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic = "Cannot include and exclude tables at the same time"]
    fn test_new_diff_payload() {
        _ = DiffPayload::new(
            "first_db",
            "second_db",
            false,
            false,
            false,
            10000,
            0,
            10,
            vec!["table1"],
            vec!["table2"],
            "schema_name",
            false,
            false,
        );
    }
}
