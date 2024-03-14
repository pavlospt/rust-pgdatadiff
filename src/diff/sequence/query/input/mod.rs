use crate::diff::sequence::query::sequence_types::SequenceName;
use crate::diff::types::SchemaName;

/// Represents the input for querying the sequence names for a schema.
pub struct QueryAllSequencesInput(SchemaName);

impl QueryAllSequencesInput {
    /// Creates a new `QueryAllSequencesInput` with the given schema name.
    ///
    /// # Arguments
    ///
    /// * `schema_name` - The name of the schema to query.
    ///
    /// # Returns
    ///
    /// A new `QueryAllSequencesInput` instance.
    pub fn new(schema_name: SchemaName) -> Self {
        Self(schema_name)
    }

    /// Returns the schema name.
    ///
    /// # Returns
    ///
    /// A reference to the schema name.
    pub fn schema_name(self) -> SchemaName {
        self.0
    }
}

/// Represents the input for querying the last values of a sequence.
pub struct QueryLastValuesInput(SchemaName, SequenceName);

impl QueryLastValuesInput {
    /// Creates a new `QueryLastValuesInput` with the given sequence name.
    ///
    /// # Arguments
    ///
    /// * `sequence_name` - The name of the sequence to query.
    ///
    /// # Returns
    ///
    /// A new `QueryLastValuesInput` instance.
    pub fn new(schema_name: SchemaName, sequence_name: SequenceName) -> Self {
        Self(schema_name, sequence_name)
    }

    /// Returns the schema name.
    ///
    /// # Returns
    ///
    /// A reference to the schema name.
    pub fn schema_name(&self) -> &SchemaName {
        &self.0
    }

    /// Returns the sequence name.
    ///
    /// # Returns
    ///
    /// A reference to the sequence name.
    pub fn sequence_name(&self) -> &SequenceName {
        &self.1
    }
}
