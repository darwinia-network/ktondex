use std::{collections::BTreeMap, time::Duration};

use dip7_datalens_indexer::config::{FinalityMode, RuntimeConfig};

#[test]
fn test_runtime_config_from_env_map_reads_datalens_database_and_chain_settings() {
    let env = BTreeMap::from([
        (
            "DIP7INDEXER_DATALENS_ENDPOINT".to_owned(),
            "https://datalens.example".to_owned(),
        ),
        (
            "DIP7INDEXER_DATALENS_APPLICATION".to_owned(),
            "dip7-production".to_owned(),
        ),
        (
            "DIP7INDEXER_DATALENS_TOKEN".to_owned(),
            "secret-token".to_owned(),
        ),
        (
            "DIP7INDEXER_DATALENS_TIMEOUT_SECS".to_owned(),
            "120".to_owned(),
        ),
        (
            "DIP7INDEXER_DATABASE_URL".to_owned(),
            "postgres://user:pass@localhost/dip7".to_owned(),
        ),
        ("DIP7INDEXER_ENABLED_CHAINS".to_owned(), "44,46".to_owned()),
        ("DIP7INDEXER_BATCH_SIZE".to_owned(), "250".to_owned()),
        ("DIP7INDEXER_START_BLOCK".to_owned(), "1000".to_owned()),
        ("DIP7INDEXER_FINALITY_MODE".to_owned(), "durable".to_owned()),
        (
            "DIP7INDEXER_CHAIN_46_START_BLOCK".to_owned(),
            "2000".to_owned(),
        ),
        (
            "DIP7INDEXER_CHAIN_46_BATCH_SIZE".to_owned(),
            "50".to_owned(),
        ),
    ]);

    let config = RuntimeConfig::from_env_map(&env).expect("config parses");

    assert_eq!(config.datalens.endpoint, "https://datalens.example");
    assert_eq!(config.datalens.application, "dip7-production");
    assert!(config.datalens.token.is_some());
    assert_eq!(config.datalens.timeout, Duration::from_secs(120));
    assert!(config.database_url.is_some());
    assert_eq!(config.enabled_chains.len(), 2);
    assert_eq!(config.batch_size, 250);
    assert_eq!(config.start_block, 1000);
    assert_eq!(config.finality_mode, FinalityMode::Durable);
    assert_eq!(config.chain(44).expect("chain 44").start_block, 1000);
    assert_eq!(config.chain(46).expect("chain 46").start_block, 2000);
    assert_eq!(config.chain(44).expect("chain 44").batch_size, 250);
    assert_eq!(config.chain(46).expect("chain 46").batch_size, 50);
}

#[test]
fn test_runtime_config_rejects_unknown_chain() {
    let env = BTreeMap::from([("DIP7INDEXER_ENABLED_CHAINS".to_owned(), "1".to_owned())]);

    let error = RuntimeConfig::from_env_map(&env).expect_err("unknown chain is rejected");

    assert!(error.to_string().contains("unconfigured DIP7 chain 1"));
}
