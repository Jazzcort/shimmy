use rmcp::service::{ClientInitializeError, ServerInitializeError, ServiceError};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum ShimmyError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Client Initialize Error: {0}")]
    ClientInitialize(#[from] ClientInitializeError),
    #[error("Server Initialize Error: {0}")]
    ServerInitialize(#[from] ServerInitializeError),
    #[error("Service Error: {0}")]
    Service(#[from] ServiceError),
    #[error("Tokio Task Join Error: {0}")]
    TokioJoin(#[from] JoinError),
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Middleman Error: {0}")]
    Middleman(String),
}
