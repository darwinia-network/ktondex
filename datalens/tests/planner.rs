use dip7_datalens_indexer::{
    config::FinalityMode,
    planner::{
        COLLATOR_STAKING_HUB_ADDRESS, DIP7_EVENT_TOPICS, EVM_LOGS_DATASET, plan_dip7_log_queries,
    },
};

#[test]
fn test_plan_dip7_log_queries_uses_collator_hub_contract_and_all_topics() {
    let plans = plan_dip7_log_queries(46, 3_675_271, 3_676_270, 500, FinalityMode::Durable)
        .expect("query plans");

    assert_eq!(plans.len(), 2);
    assert!(plans.iter().all(|plan| plan.dataset == EVM_LOGS_DATASET));
    assert!(plans.iter().all(|plan| plan.query.chain_id == 46));
    assert!(
        plans
            .iter()
            .all(|plan| { plan.query.contracts == vec![COLLATOR_STAKING_HUB_ADDRESS.to_owned()] })
    );
    assert!(
        plans
            .iter()
            .all(|plan| plan.query.topics == DIP7_EVENT_TOPICS)
    );
    assert_eq!(plans[0].query.from_block, 3_675_271);
    assert_eq!(plans[0].query.to_block, 3_675_770);
    assert_eq!(plans[1].query.from_block, 3_675_771);
    assert_eq!(plans[1].query.to_block, 3_676_270);
}

#[test]
fn test_plan_dip7_log_queries_rejects_empty_range_limit() {
    let error = plan_dip7_log_queries(46, 1, 10, 0, FinalityMode::Finalized)
        .expect_err("zero range is rejected");

    assert!(error.to_string().contains("max range length"));
}
