use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Error inserting key.")]
    InsertKey(#[from] InsertKeyError),

    #[error("Error claiming staking rewards.")]
    ClaimError(#[from] ClaimError),

    #[error("Could not find key with the provided name in the keystore.")]
    KeyNotFound,

    #[error("Unknown error encountered.")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum InsertKeyError {
    #[error("The provided key is invalid.")]
    InvalidKey,
}

#[derive(Error, Debug)]
pub enum ClaimError {
    #[error("Confirmation rejected.")]
    Rejected,

    #[error("Unknown error encountered.")]
    Unknown,
}
