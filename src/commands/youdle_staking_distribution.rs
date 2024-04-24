use crate::{
    commands::{
        consts::{youdle_consts::*, TINKERNET_WEBSOCKET},
        get_key_interactive, ExtraArgs,
    },
    keystore::Keystore,
    tinkernet::{
        self,
        ocif_staking::storage::types::general_staker_info::GeneralStakerInfo,
        runtime_types::{
            pallet_balances::pallet::Call as BalancesCall,
            pallet_ocif_staking::{pallet::Call as OcifStakingCall, primitives::CoreStakeInfo},
            pallet_utility::pallet::Call as UtilityCall,
            tinkernet_runtime::RuntimeCall,
        },
    },
};
use itertools::{EitherOrBoth, Itertools};
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use subxt::{
    dynamic::Value,
    ext::sp_core::{
        crypto::{AccountId32, Pair as PairTrait, Ss58Codec},
        sr25519::Pair,
    },
    tx::PairSigner,
    OnlineClient, PolkadotConfig,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Youdle {
    id: String,
    owner: String,
    metadata_properties: Option<Properties>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Properties {
    #[serde(alias = "Base Reputation")]
    base_rep: BaseRep,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaseRep {
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Eye20 {
    id: String,
    rootowner: String,
    parent: Option<Parent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Parent {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Background {
    id: String,
    metadata_name: String,
    rootowner: String,
    parent: Option<Parent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Banner {
    id: String,
    rootowner: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProcessedBanner {
    rootowner: String,
    amount: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProcessedYoudle {
    id: String,
    owner: String,
    core_rep: f32,
    staker_rep: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddressRewardPair {
    address: String,
    reward: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GQLResponse<Data> {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnclaimedCoreGQLData {
    pub cores: Vec<UnclaimedGQLDataInner>,
    pub stakers: Vec<UnclaimedGQLDataInner>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UnclaimedGQLDataInner {
    pub totalUnclaimed: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct YoudlesGQLData {
    pub backgrounds: Vec<Background>,
    pub banners: Vec<Banner>,
    pub eyes: Vec<Eye20>,
    pub og_youdles: Vec<Youdle>,
    pub youdles: Vec<Youdle>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variables {}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GQLQuery {
    pub operationName: &'static str,
    pub variables: Variables,
    pub query: &'static str,
}

#[derive(Debug, Serialize)]
struct CsvRecord {
    id: String,
    owner: String,
    core_rep: f32,
    staker_rep: f32,
    #[serde(rename = "")]
    empty_0: (),
    #[serde(rename = "")]
    empty_1: (),
    #[serde(rename = "")]
    empty_2: (),
    address: Option<String>,
    reward: Option<u128>,
    reward_in_units: Option<f64>,
}

pub async fn youdle_staking_distribution_command(
    account: Option<String>,
    csv: Option<Option<String>>,
    extra: ExtraArgs,
) {
    let keystore = Keystore::open();
    let key = get_key_interactive(&keystore, account).unwrap();

    let unclaimed_res: GQLResponse<UnclaimedCoreGQLData> = surf::post(TINKERNET_OCIF_SQUID)
        .body_json(&UNCLAIMED_CORE_QUERY)
        .unwrap()
        .await
        .unwrap()
        .body_json()
        .await
        .unwrap();

    let unclaimed_core: u128 = unclaimed_res.data.cores[0]
        .totalUnclaimed
        .parse::<u128>()
        .unwrap_or(0u128);
    let unclaimed_staker: u128 = unclaimed_res.data.stakers[0]
        .totalUnclaimed
        .parse::<u128>()
        .unwrap_or(0u128);

    let api = OnlineClient::<PolkadotConfig>::from_url(
        extra.endpoint.unwrap_or(TINKERNET_WEBSOCKET.to_string()),
    )
    .await
    .unwrap();

    let keys: Vec<Value> = vec![YOUDLE_DAO_ID.into()];
    let core_storage_query = subxt::dynamic::storage("OcifStaking", "CoreEraStake", keys);

    let mut results = api
        .storage()
        .at_latest()
        .await
        .unwrap()
        .iter(core_storage_query)
        .await
        .unwrap();

    let (mut min, mut max) = (0, 0);

    while let Some(Ok(kv)) = results.next().await {
        if !kv
            .value
            .as_type::<CoreStakeInfo<u128>>()
            .unwrap()
            .reward_claimed
        {
            let era = kv.keys[1].as_u128().unwrap() as u32;

            if era < min || min == 0 {
                min = era
            }
            if era > max {
                max = era
            }
        }
    }

    let claim_core_call = RuntimeCall::Utility(UtilityCall::batch {
        calls: {
            let mut calls: Vec<RuntimeCall> = Vec::new();

            for era in min..max {
                calls.push(RuntimeCall::OcifStaking(
                    OcifStakingCall::core_claim_rewards {
                        core_id: YOUDLE_DAO_ID,
                        era,
                    },
                ))
            }

            calls
        },
    });

    let staker_storage_query = subxt::dynamic::storage(
        "OcifStaking",
        "GeneralStakerInfo",
        vec![
            YOUDLE_DAO_ID.into(),
            AccountId32::from_string(YOUDLE_DAO_ADDRESS)
                .unwrap()
                .encode()
                .into(),
        ],
    );

    let results = api
        .storage()
        .at_latest()
        .await
        .unwrap()
        .fetch(&staker_storage_query)
        .await
        .unwrap()
        .unwrap()
        .as_type::<GeneralStakerInfo>()
        .unwrap();

    let min: u32 = results
        .stakes
        .into_iter()
        .map(|stake| stake.era)
        .min()
        .unwrap();

    let claim_staker_call = RuntimeCall::Utility(UtilityCall::batch {
        calls: {
            let mut calls: Vec<RuntimeCall> = Vec::new();

            for _ in min..max {
                calls.push(RuntimeCall::OcifStaking(
                    OcifStakingCall::staker_claim_rewards {
                        core_id: YOUDLE_DAO_ID,
                    },
                ))
            }

            calls
        },
    });

    let res: GQLResponse<YoudlesGQLData> = surf::post(KUSAMA_RMRK_GRAPHQL)
        .body_json(&YOUDLES_QUERY)
        .unwrap()
        .await
        .unwrap()
        .body_json()
        .await
        .unwrap();

    let data = res.data;

    let core_rewards: u128 = (unclaimed_core * 60) / 100; // 60%
    let staker_rewards: u128 = (unclaimed_staker * 60) / 100; // 60%

    let remainder_to_stake: u128 = ((unclaimed_core + unclaimed_staker) * 38) / 100; // 38% (to ensure we never use up all balance)

    let mut distribution: HashMap<String, u128> = HashMap::new();

    let youdle_list: Vec<Youdle> = data
        .og_youdles
        .into_iter()
        .chain(data.youdles.into_iter())
        .collect();
    let eye20_list: Vec<Eye20> = data.eyes;
    let bg_list: Vec<Background> = data.backgrounds;
    let banner_list: Vec<Banner> = data.banners;

    let mut youdle_list: Vec<ProcessedYoudle> = youdle_list
        .iter()
        .map(|youdle| {
            let rep = youdle
                .metadata_properties
                .clone()
                .map(|p| p.base_rep.value.parse::<u32>().unwrap())
                .unwrap_or(1u32);

            ProcessedYoudle {
                id: youdle.id.clone(),
                owner: youdle.owner.clone(),
                core_rep: rep as f32,
                staker_rep: rep as f32,
            }
        })
        .collect();

    for eye in eye20_list {
        if let Some(parent) = eye.parent.map(|p| p.id) {
            let (index, found_youdle) = youdle_list
                .iter()
                .enumerate()
                .find(|y| y.1.id == parent)
                .unwrap();

            let new_rep = match found_youdle.core_rep as u32 {
                1 | 50 => 100f32,
                100 => 150f32,
                _ => panic!("shouldn't happen"),
            };

            youdle_list[index] = ProcessedYoudle {
                id: found_youdle.id.clone(),
                owner: found_youdle.owner.clone(),
                core_rep: new_rep,
                staker_rep: new_rep,
            }
        } else {
            let (index, found_youdle) = youdle_list
                .iter()
                .enumerate()
                .rev()
                .find(|y| y.1.owner == eye.rootowner)
                .unwrap();

            let new_rep = match found_youdle.core_rep as u32 {
                1 | 50 => 100f32,
                100 => 150f32,
                _ => panic!("shouldn't happen"),
            };

            youdle_list[index] = ProcessedYoudle {
                id: found_youdle.id.clone(),
                owner: found_youdle.owner.clone(),
                core_rep: new_rep,
                staker_rep: new_rep,
            }
        }
    }

    for bg in bg_list {
        if let Some(parent) = bg.parent.map(|p| p.id) {
            let (index, found_youdle) = youdle_list
                .iter()
                .enumerate()
                .find(|y| y.1.id == parent)
                .unwrap();

            youdle_list[index] = ProcessedYoudle {
                id: found_youdle.id.clone(),
                owner: found_youdle.owner.clone(),
                core_rep: if bg.metadata_name.ends_with("19") {
                    found_youdle.core_rep * 2.0
                } else {
                    found_youdle.core_rep
                },
                staker_rep: if bg.metadata_name.ends_with("18") {
                    found_youdle.staker_rep * 2.0
                } else {
                    found_youdle.staker_rep
                },
            }
        } else {
            let (index, found_youdle) = youdle_list
                .iter()
                .enumerate()
                .rev()
                .find(|y| y.1.owner == bg.rootowner)
                .unwrap();

            youdle_list[index] = ProcessedYoudle {
                id: found_youdle.id.clone(),
                owner: found_youdle.owner.clone(),
                core_rep: if bg.metadata_name.ends_with("19") {
                    found_youdle.core_rep * 2.0
                } else {
                    found_youdle.core_rep
                },
                staker_rep: if bg.metadata_name.ends_with("18") {
                    found_youdle.staker_rep * 2.0
                } else {
                    found_youdle.staker_rep
                },
            }
        }
    }

    let mut h: HashMap<String, ProcessedBanner> = HashMap::new();

    for banner in banner_list {
        let mut pb = h
            .get(&banner.rootowner)
            .unwrap_or(&ProcessedBanner {
                rootowner: banner.rootowner.clone(),
                amount: 0,
            })
            .clone();

        pb.amount += 1;

        h.insert(banner.rootowner, pb);
    }

    for banner in h.values() {
        if let Some((index, found_youdle)) = youdle_list
            .iter()
            .enumerate()
            .rev()
            .find(|y| y.1.owner == banner.rootowner)
        {
            youdle_list[index] = ProcessedYoudle {
                id: found_youdle.id.clone(),
                owner: found_youdle.owner.clone(),
                core_rep: found_youdle.core_rep
                    + ((0.05 * banner.amount as f32) * found_youdle.core_rep),
                staker_rep: found_youdle.staker_rep
                    + ((0.05 * banner.amount as f32) * found_youdle.staker_rep),
            }
        }
    }

    let total_core_rep: f32 = youdle_list.iter().map(|youdle| youdle.core_rep).sum();
    let total_staker_rep: f32 = youdle_list.iter().map(|youdle| youdle.staker_rep).sum();

    for youdle in youdle_list.clone() {
        let prev_value = distribution.get(&youdle.owner).unwrap_or(&0);

        distribution.insert(
            youdle.owner,
            prev_value
                + ((core_rewards / total_core_rep as u128) * youdle.core_rep as u128)
                + ((staker_rewards / total_staker_rep as u128) * youdle.staker_rep as u128),
        );
    }

    let mut send_rewards_calls = {
        let mut calls: Vec<RuntimeCall> = Vec::new();

        distribution
            .clone()
            .into_iter()
            .for_each(|(address, value)| {
                calls.push(RuntimeCall::Balances(BalancesCall::transfer {
                    dest: subxt::ext::sp_runtime::MultiAddress::Id(
                        AccountId32::from_string(&address).unwrap(),
                    )
                    .into(),
                    value,
                }))
            });

        calls
    };

    let stake_remainder_call = RuntimeCall::OcifStaking(OcifStakingCall::stake {
        core_id: YOUDLE_DAO_ID,
        value: remainder_to_stake,
    });

    let complete_batch_call = RuntimeCall::Utility(UtilityCall::batch_all {
        calls: {
            let mut vec = vec![claim_core_call, claim_staker_call];

            vec.append(&mut send_rewards_calls);

            vec.push(stake_remainder_call);

            vec
        },
    });

    if let Some(maybe_path) = csv {
        if let Some(path) = maybe_path {
            let mut wtr = csv::Writer::from_path(path).unwrap();

            write_csv(&mut wtr, youdle_list, distribution);
        } else {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());

            write_csv(&mut wtr, youdle_list, distribution);
        };
    }

    let keypair = Pair::from_string(key.as_str(), None).unwrap();

    let signer = PairSigner::<PolkadotConfig, _>::new(keypair);

    let proposal_tx = tinkernet::tx().inv4().operate_multisig(
        YOUDLE_DAO_ID,
        None,
        tinkernet::runtime_types::pallet_inv4::fee_handling::FeeAsset::Native,
        complete_batch_call,
    );

    let events = api
        .tx()
        .sign_and_submit_then_watch_default(&proposal_tx, &signer)
        .await
        .unwrap()
        .wait_for_finalized_success()
        .await
        .unwrap();

    let event = events
        .find_first::<tinkernet::inv4::events::MultisigVoteStarted>()
        .unwrap();

    if let Some(event) = event {
        println!(
            "YoudleDAO distribution proposal created with hash: {}",
            hex::encode(event.call_hash.as_bytes())
        );
    }
}

fn write_csv<W: std::io::Write>(
    wtr: &mut csv::Writer<W>,
    youdle_list: Vec<ProcessedYoudle>,
    distribution: HashMap<String, u128>,
) {
    youdle_list
        .into_iter()
        .zip_longest(distribution.clone())
        .for_each(|maybe_both| match maybe_both {
            EitherOrBoth::Both(
                ProcessedYoudle {
                    id,
                    owner,
                    core_rep,
                    staker_rep,
                },
                (address, reward),
            ) => {
                wtr.serialize(CsvRecord {
                    id,
                    owner,
                    core_rep,
                    staker_rep,
                    empty_0: (),
                    empty_1: (),
                    empty_2: (),
                    address: Some(address),
                    reward: Some(reward),
                    reward_in_units: Some(reward as f64 / ONE_WITH_DECIMALS as f64),
                })
                .unwrap();
            }

            EitherOrBoth::Left(ProcessedYoudle {
                id,
                owner,
                core_rep,
                staker_rep,
            }) => {
                wtr.serialize(CsvRecord {
                    id,
                    owner,
                    core_rep,
                    staker_rep,
                    empty_0: (),
                    empty_1: (),
                    empty_2: (),
                    address: None,
                    reward: None,
                    reward_in_units: None,
                })
                .unwrap();
            }

            _ => {}
        });
}
