use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Reconciliation failed: {0}")]
    ReconcileError(String),
}

/// Result type that uses our Error
pub type Result<T> = std::result::Result<T, Error>;
