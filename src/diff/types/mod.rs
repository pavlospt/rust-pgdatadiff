use crate::diff::diff_output::DiffOutput;

#[derive(Clone)]
pub struct SchemaName(String);

impl SchemaName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

pub trait DiffOutputMarker {
    fn convert(self) -> DiffOutput;
}
