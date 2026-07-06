use anyhow::{Context, bail};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub chain_id: u64,
    pub dataset: String,
    pub next_block: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockRange {
    pub from_block: u64,
    pub to_block: u64,
}

pub fn plan_next_range(checkpoint: &Checkpoint, batch_size: u64) -> anyhow::Result<BlockRange> {
    if batch_size == 0 {
        bail!("batch size must be greater than zero");
    }

    Ok(BlockRange {
        from_block: checkpoint.next_block,
        to_block: checkpoint
            .next_block
            .checked_add(batch_size - 1)
            .context("checkpoint range overflow")?,
    })
}
