use clap::{Parser, Subcommand};
use inquire::Select;
use subxt::{
    ext::sp_core::{crypto::SecretString, sr25519::Pair, Pair as PairTrait},
    tx::PairSigner,
    PolkadotConfig,
};

pub mod claim;
pub mod consts;
pub mod insert_key;
pub mod youdle_staking_distribution;

pub use claim::claim_command;
pub use insert_key::insert_key_command;
pub use youdle_staking_distribution::youdle_staking_distribution_command;

use crate::{
    error::{CliError, KeystoreError},
    keystore::Keystore,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long)]
    pub endpoint: Option<String>,
}

pub struct ExtraArgs {
    pub endpoint: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    InsertKey {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        key: String,
    },

    #[command(subcommand)]
    Youdles(YoudlesCommands),

    #[command(subcommand)]
    Staking(StakingCommands),
}

#[derive(Subcommand, Debug)]
pub enum YoudlesCommands {
    DistributeRewards {
        #[arg(short, long)]
        account: Option<String>,
        #[arg(short, long)]
        csv: Option<Option<String>>,
    },
}

#[derive(Subcommand, Debug)]
pub enum StakingCommands {
    Claim {
        #[arg(short, long)]
        account: Option<String>,

        #[arg(long, conflicts_with = "staker", required_unless_present = "staker")]
        core: Option<u32>,

        #[arg(long, conflicts_with = "core", required_unless_present = "core")]
        staker: bool,

        #[arg(long, requires = "core")]
        as_staker: bool,

        #[arg(long, conflicts_with_all = ["start", "end"], required_unless_present_any = ["start", "end"])]
        all: bool,

        #[arg(short, long, conflicts_with = "all", required_unless_present_any = ["all", "end"])]
        start: Option<u32>,

        #[arg(short, long, conflicts_with = "all", required_unless_present_any = ["start", "all"])]
        end: Option<u32>,
    },
}

pub fn input_keystore_password() -> Result<SecretString, CliError> {
    Ok(SecretString::new(
        rpassword::prompt_password("Keystore password: ").map_err(|_| CliError::Unknown)?,
    ))
}

pub fn get_signer_interactive(
    keystore: &Keystore,
    maybe_name: Option<String>,
) -> Result<PairSigner<PolkadotConfig, Pair>, CliError> {
    let name = if let Some(n) = maybe_name {
        n
    } else {
        let account_list = keystore.account_list();

        let selection = Select::new("Select an account from the keystore:", account_list).prompt();

        selection.map_err(|_| CliError::Unknown)?
    };

    let key = keystore.get(name).ok_or(KeystoreError::KeyNotFound)?;

    let keypair =
        Pair::from_string(key.as_str(), None).map_err(|_| KeystoreError::InvalidKeyRetrieved)?;

    Ok(PairSigner::new(keypair))
}
