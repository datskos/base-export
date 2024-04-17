use crate::codec::BlockFileCodec;
use alloy_primitives::U256;
use alloy_rlp::Decodable;
use ethers_core::types::{Block as EthersBlock, Transaction as EthersTransaction};
use ethers_providers::{Http, Middleware, Provider};
use futures::{future, SinkExt};
use reth_primitives::{Block, Header, TransactionSigned, Withdrawals};
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Semaphore;
use tokio_util::codec::FramedWrite;

const CHUNK_SIZE: usize = 1000;
const CONCURRENCY: usize = 100;

pub struct BlockWriter {
    provider: Arc<Provider<Http>>,
}

impl BlockWriter {
    pub fn new(rpc_url: String) -> eyre::Result<Self> {
        let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);

        Ok(BlockWriter { provider })
    }

    pub async fn write(&self, block_range: RangeInclusive<u64>, path: String) -> eyre::Result<()> {
        let file = File::create(path).await?;
        let mut writer = FramedWrite::new(file, BlockFileCodec);
        let semaphore = Arc::new(Semaphore::new(CONCURRENCY));
        let blocks = block_range.collect::<Vec<_>>();

        for chunk in blocks.chunks(CHUNK_SIZE) {
            let mut futures = Vec::new();

            for &bn in chunk {
                let permit = semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("Failed to acquire semaphore");
                let provider = self.provider.clone();
                futures.push(tokio::spawn(async move {
                    let block = provider
                        .get_block_with_txs(bn)
                        .await?
                        .ok_or(eyre::eyre!("not found: {}", bn))?;
                    drop(permit);
                    Result::<_, eyre::Error>::Ok(ethers_block_to_block(block)?)
                }));
            }

            let results = future::join_all(futures).await;
            for result in results {
                writer.send(result??).await?;
            }
        }

        Ok(())
    }
}

fn ethers_block_to_block(block: EthersBlock<EthersTransaction>) -> eyre::Result<Block> {
    let header = Header {
        parent_hash: block.parent_hash.0.into(),
        number: block.number.unwrap().as_u64(),
        gas_limit: block.gas_limit.as_u64(),
        difficulty: U256::from_limbs(block.difficulty.0),
        nonce: block.nonce.unwrap().to_low_u64_be(),
        extra_data: block.extra_data.0.clone().into(),
        state_root: block.state_root.0.into(),
        transactions_root: block.transactions_root.0.into(),
        receipts_root: block.receipts_root.0.into(),
        timestamp: block.timestamp.as_u64(),
        mix_hash: block.mix_hash.unwrap().0.into(),
        beneficiary: block.author.unwrap().0.into(),
        base_fee_per_gas: block.base_fee_per_gas.map(|fee| fee.as_u64()),
        ommers_hash: block.uncles_hash.0.into(),
        gas_used: block.gas_used.as_u64(),
        logs_bloom: block.logs_bloom.unwrap_or_default().0.into(),
        withdrawals_root: block.withdrawals_root.map(|b| b.0.into()),
        blob_gas_used: block.blob_gas_used.map(|f|f.as_u64()),
        excess_blob_gas: block.excess_blob_gas.map(|f|f.as_u64()),
        parent_beacon_block_root: block.parent_beacon_block_root.map(|b| b.0.into()),
    };
    let mut body: Vec<TransactionSigned> = vec![];
    for tx in block.transactions {
        let rlp = tx.rlp();
        let mut bytes: &[u8] = rlp.0.as_ref();
        let tx2 = TransactionSigned::decode(&mut bytes)
            .map_err(|e| eyre::eyre!("could not decode: {}", e))?;
        body.push(tx2);
    }
    Ok(Block {
        header,
        body,
        ommers: vec![],
        withdrawals: match block.withdrawals {
            None => None,

            // No withdrawals on base
            Some(_) => Some(Withdrawals::new(vec![]))
        },
    })
}
