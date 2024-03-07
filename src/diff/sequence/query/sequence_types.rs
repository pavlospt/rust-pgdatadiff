#[derive(Clone)]
pub struct SequenceName(String);

impl SequenceName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> String {
        self.0.to_string()
    }
}
