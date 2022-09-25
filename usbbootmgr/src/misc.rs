use std::{error::Error, fmt::Display};

#[derive(thiserror::Error, Debug)]
pub struct AggregateError {
    pub errors: Vec<anyhow::Error>,
}
impl Display for AggregateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!()
    }
}
