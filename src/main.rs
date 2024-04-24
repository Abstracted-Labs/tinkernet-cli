use clap::Parser;

mod commands;
mod error;
mod keystore;
use anyhow::Result;
use commands::{
    claim_command, insert_key_command, youdle_staking_distribution_command, Args, Commands,
    ExtraArgs, StakingCommands, YoudlesCommands,
};

#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod tinkernet {}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let extra = ExtraArgs {
        endpoint: args.endpoint,
    };

    match args.command {
        Commands::InsertKey { name, key } => insert_key_command(name, key)?,
        Commands::Youdles(youdles_command) => match youdles_command {
            YoudlesCommands::DistributeRewards { account, csv } => {
                youdle_staking_distribution_command(account, csv, extra).await
            }
        },
        Commands::Staking(staking_command) => match staking_command {
            StakingCommands::Claim {
                account,
                core,
                staker,
                as_staker,
                all,
                start,
                end,
            } => claim_command(account, core, staker, as_staker, all, start, end, extra).await?,
        },
    };

    Ok(())
}
