use crate::{
    error::{CliError, InsertKeyError},
    keystore::Keystore,
};
use subxt::ext::sp_core::{crypto::Pair as PairTrait, sr25519::Pair};

pub fn insert_key_command(name: String, key: String) -> Result<(), CliError> {
    Pair::from_string(&key, None).map_err(|_| InsertKeyError::InvalidKey)?;

    Keystore::open().insert_and_save(name, key).unwrap();

    Ok(())
}
