use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Error inserting key.")]
    InsertKey(#[from] InsertKeyError),

    #[error("Error claiming staking rewards.")]
    Claim(#[from] ClaimError),

    #[error("Error distributing YoudleDAO staking rewards.")]
    YoudleDist(#[from] YoudleDistError),

    #[error("Keystore error.")]
    Keystore(#[from] KeystoreError),

    #[error("Api Error.")]
    Api(#[from] ApiError),

    #[error("Unknown error encountered.")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum YoudleDistError {
    #[error("Failed to decode an account during reward generation.")]
    FailedDecodingAccount,
}

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("Invalid key retrieved from keystore.")]
    InvalidKeyRetrieved,

    #[error("Could not find key with the provided name in the keystore.")]
    KeyNotFound,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to connect to the provided endpoint.")]
    EndpointConnectionFailed,

    #[error("Failed to get chain storage data.")]
    StorageFailed,

    #[error("Failed to decode storage data.")]
    DecodeFailed,

    #[error("Failed to sign transaction payload.")]
    SigningFailed,

    #[error("Failed submitting transaction to RPC.")]
    SubmissionFailed,

    #[error("Transaction was not successful: {0}")]
    TransactionNotSuccessful(#[from] subxt::Error),

    #[error("Could not find the relevant events resulting from the transaction")]
    EventNotFound,
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
}
