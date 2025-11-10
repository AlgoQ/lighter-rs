use thiserror::Error;

#[derive(Error, Debug)]
pub enum FFIError {
    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Invalid nonce: {0}")]
    Nonce(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, FFIError>;
