use std::{collections::BTreeMap, env, fmt, str::FromStr, time::Duration};

use anyhow::{Context, bail};

use crate::planner::default_chain_config;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    pub datalens: DatalensConfig,
    pub database_url: Option<SecretString>,
    pub enabled_chains: Vec<ChainConfig>,
    pub batch_size: u64,
    pub start_block: u64,
    pub finality_mode: FinalityMode,
    pub poll_interval: Duration,
}

impl RuntimeConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let env = env::vars().collect::<BTreeMap<_, _>>();
        Self::from_env_map(&env)
    }

    pub fn database_url_from_env() -> anyhow::Result<SecretString> {
        let env = env::vars().collect::<BTreeMap<_, _>>();
        optional_env(&env, "DIP7INDEXER_DATABASE_URL")
            .map(SecretString::new)
            .context("DIP7INDEXER_DATABASE_URL must be configured")
    }

    pub fn from_env_map(env: &BTreeMap<String, String>) -> anyhow::Result<Self> {
        let start_block = optional_u64(env, "DIP7INDEXER_START_BLOCK")?;
        let batch_size = optional_u64(env, "DIP7INDEXER_BATCH_SIZE")?.unwrap_or(1_000);
        if batch_size == 0 {
            bail!("DIP7INDEXER_BATCH_SIZE must be greater than zero");
        }

        let finality_mode = optional_env(env, "DIP7INDEXER_FINALITY_MODE")
            .as_deref()
            .map(str::parse)
            .transpose()?
            .unwrap_or(FinalityMode::Finalized);
        let chain_ids = optional_list(env, "DIP7INDEXER_ENABLED_CHAINS");
        let chain_ids = if chain_ids.is_empty() {
            vec!["44".to_owned(), "46".to_owned(), "701".to_owned()]
        } else {
            chain_ids
        };
        let enabled_chains = chain_ids
            .into_iter()
            .map(|value| {
                let chain_id = value
                    .parse::<u64>()
                    .with_context(|| format!("parse DIP7INDEXER_ENABLED_CHAINS item: {value}"))?;
                ChainConfig::from_env_map(env, chain_id, start_block, batch_size, finality_mode)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let datalens_timeout_secs =
            optional_u64(env, "DIP7INDEXER_DATALENS_TIMEOUT_SECS")?.unwrap_or(300);
        if datalens_timeout_secs == 0 {
            bail!("DIP7INDEXER_DATALENS_TIMEOUT_SECS must be greater than zero");
        }

        Ok(Self {
            datalens: DatalensConfig {
                endpoint: optional_env(env, "DIP7INDEXER_DATALENS_ENDPOINT")
                    .unwrap_or_else(|| "http://localhost:8080".to_owned())
                    .trim_end_matches('/')
                    .to_owned(),
                application: optional_env(env, "DIP7INDEXER_DATALENS_APPLICATION")
                    .unwrap_or_else(|| "dip7indexer".to_owned()),
                token: optional_env(env, "DIP7INDEXER_DATALENS_TOKEN").map(SecretString::new),
                timeout: Duration::from_secs(datalens_timeout_secs),
            },
            database_url: optional_env(env, "DIP7INDEXER_DATABASE_URL").map(SecretString::new),
            enabled_chains,
            batch_size,
            start_block: start_block.unwrap_or(0),
            finality_mode,
            poll_interval: Duration::from_secs(
                optional_u64(env, "DIP7INDEXER_POLL_INTERVAL_SECS")?.unwrap_or(30),
            ),
        })
    }

    pub fn chain(&self, chain_id: u64) -> Option<&ChainConfig> {
        self.enabled_chains
            .iter()
            .find(|chain| chain.chain_id == chain_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatalensConfig {
    pub endpoint: String,
    pub application: String,
    pub token: Option<SecretString>,
    pub timeout: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub start_block: u64,
    pub batch_size: u64,
    pub contracts: Vec<String>,
    pub topics: Vec<String>,
    pub finality_mode: FinalityMode,
}

impl ChainConfig {
    fn from_env_map(
        env: &BTreeMap<String, String>,
        chain_id: u64,
        default_start_block: Option<u64>,
        default_batch_size: u64,
        default_finality_mode: FinalityMode,
    ) -> anyhow::Result<Self> {
        let prefix = format!("DIP7INDEXER_CHAIN_{chain_id}");
        let default = default_chain_config(chain_id)?;
        let start_block = optional_u64(env, &format!("{prefix}_START_BLOCK"))?
            .or(default_start_block)
            .unwrap_or(default.start_block);
        let batch_size =
            optional_u64(env, &format!("{prefix}_BATCH_SIZE"))?.unwrap_or(default_batch_size);
        if batch_size == 0 {
            bail!("{prefix}_BATCH_SIZE must be greater than zero");
        }
        let finality_mode = optional_env(env, &format!("{prefix}_FINALITY_MODE"))
            .as_deref()
            .map(str::parse)
            .transpose()?
            .unwrap_or(default_finality_mode);

        Ok(Self {
            chain_id,
            start_block,
            batch_size,
            contracts: default.contracts,
            topics: default.topics,
            finality_mode,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalityMode {
    Finalized,
    Durable,
    Safe,
    Latest,
}

impl FinalityMode {
    pub fn as_datalens_value(self) -> &'static str {
        match self {
            Self::Finalized => "finalized",
            Self::Durable => "durable",
            Self::Safe => "safe",
            Self::Latest => "latest",
        }
    }
}

impl FromStr for FinalityMode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "finalized" => Ok(Self::Finalized),
            "durable" => Ok(Self::Durable),
            "safe" => Ok(Self::Safe),
            "latest" => Ok(Self::Latest),
            value => bail!("invalid DIP7 finality mode {value}"),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted>")
    }
}

fn optional_env(env: &BTreeMap<String, String>, key: &str) -> Option<String> {
    env.get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn optional_u64(env: &BTreeMap<String, String>, key: &str) -> anyhow::Result<Option<u64>> {
    optional_env(env, key)
        .map(|value| {
            value
                .parse::<u64>()
                .with_context(|| format!("parse {key} as u64"))
        })
        .transpose()
}

fn optional_list(env: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    optional_env(env, key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
