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

/// Builder for `DiffPayload`.
///
/// All fields are required. Call `.build()` after setting all fields.
pub struct DiffPayloadBuilder {
    first_db: Option<String>,
    second_db: Option<String>,
    only_tables: Option<bool>,
    only_sequences: Option<bool>,
    only_count: Option<bool>,
    chunk_size: Option<i64>,
    start_position: Option<i64>,
    max_connections: Option<i64>,
    include_tables: Option<Vec<String>>,
    exclude_tables: Option<Vec<String>>,
    schema_name: Option<String>,
    accept_invalid_certs_first_db: Option<bool>,
    accept_invalid_certs_second_db: Option<bool>,
}

impl DiffPayloadBuilder {
    pub fn first_db(mut self, value: impl Into<String>) -> Self {
        self.first_db = Some(value.into());
        self
    }
    pub fn second_db(mut self, value: impl Into<String>) -> Self {
        self.second_db = Some(value.into());
        self
    }
    pub fn only_tables(mut self, value: bool) -> Self {
        self.only_tables = Some(value);
        self
    }
    pub fn only_sequences(mut self, value: bool) -> Self {
        self.only_sequences = Some(value);
        self
    }
    pub fn only_count(mut self, value: bool) -> Self {
        self.only_count = Some(value);
        self
    }
    pub fn chunk_size(mut self, value: i64) -> Self {
        self.chunk_size = Some(value);
        self
    }
    pub fn start_position(mut self, value: i64) -> Self {
        self.start_position = Some(value);
        self
    }
    pub fn max_connections(mut self, value: i64) -> Self {
        self.max_connections = Some(value);
        self
    }
    pub fn include_tables(mut self, value: Vec<impl Into<String>>) -> Self {
        self.include_tables = Some(value.into_iter().map(|t| t.into()).collect());
        self
    }
    pub fn exclude_tables(mut self, value: Vec<impl Into<String>>) -> Self {
        self.exclude_tables = Some(value.into_iter().map(|t| t.into()).collect());
        self
    }
    pub fn schema_name(mut self, value: impl Into<String>) -> Self {
        self.schema_name = Some(value.into());
        self
    }
    pub fn accept_invalid_certs_first_db(mut self, value: bool) -> Self {
        self.accept_invalid_certs_first_db = Some(value);
        self
    }
    pub fn accept_invalid_certs_second_db(mut self, value: bool) -> Self {
        self.accept_invalid_certs_second_db = Some(value);
        self
    }

    pub fn build(self) -> DiffPayload {
        let first_db = self.first_db.expect("first_db is required");
        let second_db = self.second_db.expect("second_db is required");
        let only_tables = self.only_tables.expect("only_tables is required");
        let only_sequences = self.only_sequences.expect("only_sequences is required");
        let only_count = self.only_count.expect("only_count is required");
        let chunk_size = self.chunk_size.expect("chunk_size is required");
        let start_position = self.start_position.expect("start_position is required");
        let max_connections = self.max_connections.expect("max_connections is required");
        let include_tables: Vec<String> = self
            .include_tables
            .expect("include_tables is required");
        let exclude_tables: Vec<String> = self
            .exclude_tables
            .expect("exclude_tables is required");
        let schema_name = self.schema_name.expect("schema_name is required");
        let accept_invalid_certs_first_db = self
            .accept_invalid_certs_first_db
            .expect("accept_invalid_certs_first_db is required");
        let accept_invalid_certs_second_db = self
            .accept_invalid_certs_second_db
            .expect("accept_invalid_certs_second_db is required");

        if !include_tables.is_empty() && !exclude_tables.is_empty() {
            panic!("Cannot include and exclude tables at the same time");
        }

        DiffPayload {
            first_db,
            second_db,
            only_tables,
            only_sequences,
            only_count,
            chunk_size,
            start_position,
            max_connections,
            include_tables,
            exclude_tables,
            schema_name,
            accept_invalid_certs_first_db,
            accept_invalid_certs_second_db,
        }
    }
}

impl DiffPayload {
    pub fn builder() -> DiffPayloadBuilder {
        DiffPayloadBuilder {
            first_db: None,
            second_db: None,
            only_tables: None,
            only_sequences: None,
            only_count: None,
            chunk_size: None,
            start_position: None,
            max_connections: None,
            include_tables: None,
            exclude_tables: None,
            schema_name: None,
            accept_invalid_certs_first_db: None,
            accept_invalid_certs_second_db: None,
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
    pub fn included_tables(&self) -> &[String] {
        &self.include_tables
    }
    pub fn excluded_tables(&self) -> &[String] {
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
        _ = DiffPayload::builder()
            .first_db("first_db")
            .second_db("second_db")
            .only_tables(false)
            .only_sequences(false)
            .only_count(false)
            .chunk_size(10000)
            .start_position(0)
            .max_connections(10)
            .include_tables(vec!["table1"])
            .exclude_tables(vec!["table2"])
            .schema_name("schema_name")
            .accept_invalid_certs_first_db(false)
            .accept_invalid_certs_second_db(false)
            .build();
    }
}
