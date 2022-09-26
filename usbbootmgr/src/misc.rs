use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct AggregateError {
    errors: Vec<anyhow::Error>,
}
impl AggregateError {
    /// Returns None if the provided vector is empty.
    pub fn new(errors: Vec<anyhow::Error>) -> Option<AggregateError> {
        if errors.len() == 0 {
            return None;
        }
        Some(AggregateError {
            errors,
        })
    }
    pub fn get(&self) -> &[anyhow::Error] {
        self.errors.as_slice()
    }
}
impl Display for AggregateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Multiple errors:")?;
        for error in &self.errors {
            writeln!(f, "    - {}", error)?;
        }
        Ok(())
    }
}
impl Error for AggregateError {
}
