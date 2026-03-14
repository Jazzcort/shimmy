use std::borrow::Cow;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShimmyError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Shimmy Error: {0}")]
    Shimmy(String),
}

impl Serialize for ShimmyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
