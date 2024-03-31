use super::table_types::{TableName, TableOffset, TablePosition, TablePrimaryKeys};
use crate::diff::types::SchemaName;


/// Represents the input for querying the count of a table.
pub struct QueryTableCountInput {
    schema_name: SchemaName,
    table_name: TableName,
}

impl QueryTableCountInput {
    /// Creates a new `QueryTableCountInput` instance.
    pub fn new(schema_name: SchemaName, table_name: TableName) -> Self {
        Self {
            schema_name,
            table_name,
        }
    }

    pub fn schema_name(&self) -> &SchemaName {
        &self.schema_name
    }

    pub fn table_name(&self) -> &TableName {
        &self.table_name
    }
}

/// Represents the input for querying table names.
pub struct QueryTableNamesInput {
    schema_name: SchemaName,
    included_tables: Vec<String>,
    excluded_tables: Vec<String>,
}

impl QueryTableNamesInput {
    /// Creates a new `QueryTableNamesInput` instance.
    pub fn new(
        schema_name: SchemaName,
        included_tables: Vec<impl Into<String>>,
        excluded_tables: Vec<impl Into<String>>,
    ) -> Self {
        Self {
            schema_name,
            included_tables: included_tables.into_iter().map(|t| t.into()).collect(),
            excluded_tables: excluded_tables.into_iter().map(|t| t.into()).collect(),
        }
    }

    pub fn schema_name(&self) -> &SchemaName {
        &self.schema_name
    }

    pub fn included_tables(&self) -> Vec<String> {
        self.included_tables.to_vec()
    }

    pub fn excluded_tables(&self) -> Vec<String> {
        self.excluded_tables.to_vec()
    }
}

/// Represents the input for querying hash data.
pub struct QueryHashDataInput {
    schema_name: SchemaName,
    table_name: TableName,
    primary_keys: TablePrimaryKeys,
    position: TablePosition,
    offset: TableOffset,
}

impl QueryHashDataInput {
    /// Creates a new `QueryHashDataInput` instance.
    pub fn new(
        schema_name: SchemaName,
        table_name: TableName,
        primary_keys: TablePrimaryKeys,
        position: TablePosition,
        offset: TableOffset,
    ) -> Self {
        Self {
            schema_name,
            table_name,
            primary_keys,
            position,
            offset,
        }
    }

    pub fn schema_name(&self) -> SchemaName {
        self.schema_name.clone()
    }

    pub fn table_name(&self) -> TableName {
        self.table_name.clone()
    }

    pub fn primary_keys(&self) -> TablePrimaryKeys {
        self.primary_keys.clone()
    }

    pub fn position(&self) -> TablePosition {
        self.position.clone()
    }

    pub fn offset(&self) -> TableOffset {
        self.offset.clone()
    }
}

/// Represents the input for querying primary keys.
pub struct QueryPrimaryKeysInput {
    table_name: String,
}

impl QueryPrimaryKeysInput {
    pub fn new(table_name: String) -> Self {
        Self { table_name }
    }

    pub fn table_name(&self) -> String {
        self.table_name.to_string()
    }
}
