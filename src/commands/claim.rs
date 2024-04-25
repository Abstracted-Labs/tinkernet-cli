use crate::{
    commands::{consts::TINKERNET_WEBSOCKET, get_signer_interactive, ExtraArgs},
    error::{ApiError, ClaimError, CliError},
    keystore::Keystore,
    tinkernet::{
        self,
        runtime_types::{
            pallet_ocif_staking::{pallet::Call as OcifStakingCall, primitives::CoreStakeInfo},
            tinkernet_runtime::RuntimeCall,
        },
    },
    util::planck_to_unit,
};
use inquire::Confirm;
use subxt::{OnlineClient, PolkadotConfig};

pub enum EraRange {
    All,
    EraToEnd(u32),
    StartToEra(u32),
    EraToEra(u32, u32),
}

pub async fn claim_command(
    account: Option<String>,
    core: Option<u32>,
    staker: bool,
    as_staker: bool,
    all: bool,
    start: Option<u32>,
    end: Option<u32>,
    extra: ExtraArgs,
) -> Result<(), CliError> {
    let keystore = Keystore::open();
    let signer = get_signer_interactive(&keystore, account)?;

    let api = OnlineClient::<PolkadotConfig>::from_url(
        extra.endpoint.unwrap_or(TINKERNET_WEBSOCKET.to_string()),
    )
    .await
    .map_err(|_| ApiError::EndpointConnectionFailed)?;

    let era_range = match (all, start, end) {
        (true, None, None) => EraRange::All,
        (false, Some(start_era), None) => EraRange::EraToEnd(start_era),
        (false, None, Some(end_era)) => EraRange::StartToEra(end_era),
        (false, Some(start_era), Some(end_era)) => EraRange::EraToEra(start_era, end_era),
        _ => return Err(CliError::Unknown),
    };

    let (min, max) = if let Some(core_id) = core {
        match era_range {
            EraRange::All => {
                let core_storage_query =
                    subxt::dynamic::storage("OcifStaking", "CoreEraStake", vec![core_id.into()]);

                let mut results = api
                    .storage()
                    .at_latest()
                    .await
                    .map_err(|_| ApiError::StorageFailed)?
                    .iter(core_storage_query)
                    .await
                    .map_err(|_| ApiError::StorageFailed)?;

                let (mut min, mut max) = (0, 0);

                while let Some(Ok(kv)) = results.next().await {
                    if !kv
                        .value
                        .as_type::<CoreStakeInfo<u128>>()
                        .map_err(|_| ApiError::DecodeFailed)?
                        .reward_claimed
                    {
                        if let Some(era) = kv.keys[1].as_u128().map(|k| k as u32) {
                            if era < min || min == 0 {
                                min = era
                            }
                            if era > max {
                                max = era
                            }
                        }
                    }
                }

                (min, max)
            }

            _ => unimplemented!(),
        }
    } else {
        unimplemented!()
    };

    match (core, as_staker, staker) {
        (Some(core_id), false, false) => {
            let mut claim_calls: Vec<RuntimeCall> = Vec::new();

            for era in min..max {
                claim_calls.push(RuntimeCall::OcifStaking(
                    OcifStakingCall::core_claim_rewards { core_id, era },
                ))
            }

            let proposal_tx = tinkernet::tx().utility().batch(claim_calls);

            let tx = api
                .tx()
                .create_signed(&proposal_tx, &signer, Default::default())
                .await
                .map_err(|_| ApiError::SigningFailed)?;

            let maybe_fee = tx.partial_fee_estimate().await;

            Confirm::new(
                format!(
                    "Confirm transaction to claim rewards for {} eras for core #{}?",
                    (min..max).len(),
                    core_id
                )
                .as_str(),
            )
            .with_default(false)
            .with_help_message(
                maybe_fee
                    .map(|fee| {
                        format!(
                            "This transaction will cost approximately {} TNKR in fees.",
                            planck_to_unit(fee)
                        )
                    })
                    .unwrap_or(String::from("Could not calculate transaction fees."))
                    .as_str(),
            )
            .prompt()
            .map_err(|_| ClaimError::Rejected)?;

            let events = tx
                .submit_and_watch()
                .await
                .map_err(|_| ApiError::SubmissionFailed)?
                .wait_for_finalized_success()
                .await
                .map_err(ApiError::TransactionNotSuccessful)?;

            let total_claimed: u128 = events
                .find::<tinkernet::ocif_staking::events::CoreClaimed>()
                .filter_map(|maybe_event| {
                    if let Ok(event) = maybe_event {
                        if event.core == core_id {
                            Some(event.amount)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .sum();

            eprintln!(
                "Successfully claimed {} TNKR for core #{}",
                planck_to_unit(total_claimed),
                core_id
            );
        }

        (Some(_), true, false) => unimplemented!(),

        (None, false, true) => unimplemented!(),

        _ => return Err(CliError::Unknown),
    }

    Ok(())
}
