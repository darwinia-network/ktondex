use anyhow::Context;
use clap::{Parser, Subcommand};
use dip7_datalens_indexer::runtime::{migrate, run, smoke_datalens};

#[derive(Debug, Parser)]
#[command(name = "dip7-datalens-indexer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        #[arg(long)]
        once: bool,
    },
    Migrate,
    SmokeDatalens,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging()?;
    let cli = Cli::parse();

    match cli.command {
        Command::Run { once } => run(once).await,
        Command::Migrate => migrate().await,
        Command::SmokeDatalens => smoke_datalens().await,
    }
}

fn init_logging() -> anyhow::Result<()> {
    tracing_log::LogTracer::init().context("initialize log tracer")?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize tracing subscriber: {error}"))
}
