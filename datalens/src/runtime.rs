use anyhow::Context;

use crate::{
    checkpoint::{Checkpoint, plan_next_range},
    config::{ChainConfig, FinalityMode, RuntimeConfig},
    planner::{EVM_LOGS_DATASET, PlannedDatalensLogQuery, plan_dip7_log_queries},
};

pub async fn run(run_once: bool) -> anyhow::Result<()> {
    let config = RuntimeConfig::from_env().context("load DIP7 indexer runtime config")?;

    log::info!(
        "starting DIP7 Datalens indexer endpoint={} application={} token_configured={} database_configured={} chains={} batch_size={} finality={} run_once={}",
        config.datalens.endpoint,
        config.datalens.application,
        config.datalens.token.is_some(),
        config.database_url.is_some(),
        config.enabled_chains.len(),
        config.batch_size,
        config.finality_mode.as_datalens_value(),
        run_once,
    );

    for chain in &config.enabled_chains {
        let plans = plan_startup_log_queries(chain, chain.finality_mode)?;
        log::info!(
            "planned DIP7 Datalens query chain_id={} start_block={} dataset={} contracts={} topics={}",
            chain.chain_id,
            chain.start_block,
            plans[0].dataset,
            plans[0].query.contracts.len(),
            plans[0].query.topics.len(),
        );
    }

    Ok(())
}

pub fn plan_startup_log_queries(
    chain: &ChainConfig,
    finality_mode: FinalityMode,
) -> anyhow::Result<Vec<PlannedDatalensLogQuery>> {
    let range = plan_next_range(
        &Checkpoint {
            chain_id: chain.chain_id,
            dataset: EVM_LOGS_DATASET.to_owned(),
            next_block: chain.start_block,
        },
        chain.batch_size,
    )?;

    plan_dip7_log_queries(
        chain.chain_id,
        range.from_block,
        range.to_block,
        chain.batch_size,
        finality_mode,
    )
}

pub async fn migrate() -> anyhow::Result<()> {
    let _database_url = RuntimeConfig::database_url_from_env()
        .context("load DIP7 indexer database config for migrations")?;
    log::info!("DIP7 indexer migrations are not defined yet");
    Ok(())
}

pub async fn smoke_datalens() -> anyhow::Result<()> {
    let config = RuntimeConfig::from_env().context("load DIP7 indexer runtime config")?;
    log::info!(
        "DIP7 Datalens config loaded endpoint={} application={} token_configured={}",
        config.datalens.endpoint,
        config.datalens.application,
        config.datalens.token.is_some(),
    );
    Ok(())
}
