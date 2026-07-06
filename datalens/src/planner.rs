use anyhow::{bail, ensure};

use crate::config::{ChainConfig, FinalityMode};

pub const COLLATOR_STAKING_HUB_ADDRESS: &str = "0xa4fFAC7A5Da311D724eD47393848f694Baee7930";
pub const EVM_LOGS_DATASET: &str = "evm.logs";

pub const ADD_COLLATOR_TOPIC: &str =
    "0x63ae7a710e54d1dc138fb1366e2080ca02620195c869a63741773f847e48d149";
pub const COMMISSION_UPDATED_TOPIC: &str =
    "0x87342df63874e81545b667a9f796b2b67e7e4430236e63aaf0e3f6bb137bc76f";
pub const INITIALIZED_TOPIC: &str =
    "0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2";
pub const NOMINATION_POOL_CREATED_TOPIC: &str =
    "0xe98f990113b5d754d6a61817da01d2de91a7116d658ae3aee0287b5233cc607c";
pub const REMOVE_COLLATOR_TOPIC: &str =
    "0x9d9a0c771f16c1cba20b04a7140434803e044176521733d8455ab7b195de1d9a";
pub const REWARD_DISTRIBUTED_TOPIC: &str =
    "0xe34918ff1c7084970068b53fd71ad6d8b04e9f15d3886cbf006443e6cdc52ea6";
pub const STAKED_TOPIC: &str = "0x6e613e504dcbe267f60e295b08e0a211b63db8690d660e5ed4f864d409bb6620";
pub const UNSTAKED_TOPIC: &str =
    "0xb9f634dc06666f582d41ccfbbb32947903698e0dbd4cc2a28f7c48999e044ffe";
pub const UPDATE_COLLATOR_TOPIC: &str =
    "0xb5004b418bb75a941f46ca1db10f59c7dc873456b0e3e419a6aa73ab0b790bce";

pub const DIP7_EVENT_TOPICS: &[&str] = &[
    ADD_COLLATOR_TOPIC,
    COMMISSION_UPDATED_TOPIC,
    INITIALIZED_TOPIC,
    NOMINATION_POOL_CREATED_TOPIC,
    REMOVE_COLLATOR_TOPIC,
    REWARD_DISTRIBUTED_TOPIC,
    STAKED_TOPIC,
    UNSTAKED_TOPIC,
    UPDATE_COLLATOR_TOPIC,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedDatalensLogQuery {
    pub dataset: String,
    pub query: DatalensLogQuery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatalensLogQuery {
    pub chain_id: u64,
    pub from_block: u64,
    pub to_block: u64,
    pub contracts: Vec<String>,
    pub topics: Vec<&'static str>,
    pub finality_mode: FinalityMode,
}

pub fn default_chain_config(chain_id: u64) -> anyhow::Result<ChainConfig> {
    let start_block = match chain_id {
        44 => 3_681_931,
        46 => 3_675_271,
        701 => 1_075_463,
        _ => bail!("unconfigured DIP7 chain {chain_id}"),
    };

    Ok(ChainConfig {
        chain_id,
        start_block,
        batch_size: 1_000,
        contracts: vec![COLLATOR_STAKING_HUB_ADDRESS.to_owned()],
        topics: DIP7_EVENT_TOPICS
            .iter()
            .map(|topic| (*topic).to_owned())
            .collect(),
        finality_mode: FinalityMode::Finalized,
    })
}

pub fn plan_dip7_log_queries(
    chain_id: u64,
    from_block: u64,
    to_block: u64,
    max_range_len: u64,
    finality_mode: FinalityMode,
) -> anyhow::Result<Vec<PlannedDatalensLogQuery>> {
    ensure!(
        max_range_len > 0,
        "max range length must be greater than zero"
    );
    ensure!(
        from_block <= to_block,
        "from block must be less than or equal to to block"
    );
    default_chain_config(chain_id)?;

    let mut plans = Vec::new();
    let mut next_from = from_block;
    while next_from <= to_block {
        let range_end = next_from.saturating_add(max_range_len - 1).min(to_block);
        plans.push(PlannedDatalensLogQuery {
            dataset: EVM_LOGS_DATASET.to_owned(),
            query: DatalensLogQuery {
                chain_id,
                from_block: next_from,
                to_block: range_end,
                contracts: vec![COLLATOR_STAKING_HUB_ADDRESS.to_owned()],
                topics: DIP7_EVENT_TOPICS.to_vec(),
                finality_mode,
            },
        });

        if range_end == u64::MAX {
            break;
        }
        next_from = range_end + 1;
    }

    Ok(plans)
}
