use dip7_datalens_indexer::{
    config::{FinalityMode, RuntimeConfig},
    runtime::plan_startup_log_queries,
};

#[test]
fn test_plan_startup_log_queries_uses_full_configured_batch() {
    let config = RuntimeConfig::from_env_map(&Default::default()).expect("default config parses");
    let chain = config.chain(46).expect("chain 46");

    let plans =
        plan_startup_log_queries(chain, FinalityMode::Finalized).expect("startup query plans");

    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].query.from_block, chain.start_block);
    assert_eq!(
        plans[0].query.to_block,
        chain.start_block + chain.batch_size - 1
    );
}
