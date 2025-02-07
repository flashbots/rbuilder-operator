use std::sync::Arc;

use alloy_signer_local::PrivateKeySigner;
use rbuilder::{
    building::builders::{UnfinishedBlockBuildingSink, UnfinishedBlockBuildingSinkFactory},
    live_builder::{
        block_output::bid_value_source::interfaces::BidValueSource,
        payload_events::MevBoostSlotData,
    },
};
use redis::RedisError;
use tokio_util::sync::CancellationToken;

use super::{
    best_true_value_pusher::{Backend, BuiltBlockInfoPusher, LastBuiltBlockInfoCell},
    blocks_processor_backend::{self, BlocksProcessorBackend},
    redis_backend::RedisBackend,
    unfinished_block_building_sink_wrapper::UnfinishedBlockBuildingSinkWrapper,
};

/// Wrapper over a UnfinishedBlockBuildingSinkFactory that wraps the generated [UnfinishedBlockBuildingSink]s so they push the tbv to redis.
#[derive(Debug)]
pub struct UnfinishedBlockBuildingSinkFactoryWrapper {
    factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
    best_local_value: LastBuiltBlockInfoCell,
    competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to init redis connection : {0}")]
    Redis(#[from] RedisError),
    #[error("BlocksProcessor backend error: {0}")]
    BlocksProcessor(#[from] blocks_processor_backend::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

impl UnfinishedBlockBuildingSinkFactoryWrapper {
    /// Constructor using a redis channel backend
    pub fn new_redis(
        factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
        competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
        tbv_push_redis_url: String,
        tbv_push_redis_channel: String,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let best_true_value_redis = redis::Client::open(tbv_push_redis_url)?;
        let redis_backend = RedisBackend::new(best_true_value_redis, tbv_push_redis_channel);
        Self::new(
            factory,
            competition_bid_value_source,
            redis_backend,
            cancellation_token,
        )
    }

    /// Constructor using signed JSON-RPC block-processor API
    pub fn new_block_processor(
        factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
        competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
        url: String,
        signer: PrivateKeySigner,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let backend = BlocksProcessorBackend::new(url, signer)?;
        Self::new(
            factory,
            competition_bid_value_source,
            backend,
            cancellation_token,
        )
    }

    fn new<BackendType: Backend + Send + 'static>(
        factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
        competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
        backend: BackendType,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let last_local_value = LastBuiltBlockInfoCell::default();
        let pusher =
            BuiltBlockInfoPusher::new(last_local_value.clone(), backend, cancellation_token);
        std::thread::spawn(move || pusher.run_push_task());
        Ok(UnfinishedBlockBuildingSinkFactoryWrapper {
            factory,
            competition_bid_value_source,
            best_local_value: last_local_value,
        })
    }
}

impl UnfinishedBlockBuildingSinkFactory for UnfinishedBlockBuildingSinkFactoryWrapper {
    fn create_sink(
        &mut self,
        slot_data: MevBoostSlotData,
        cancel: CancellationToken,
    ) -> Arc<dyn UnfinishedBlockBuildingSink> {
        let sink = self.factory.create_sink(slot_data.clone(), cancel);
        Arc::new(UnfinishedBlockBuildingSinkWrapper::new(
            sink,
            self.best_local_value.clone(),
            self.competition_bid_value_source.clone(),
            slot_data.block(),
            slot_data.slot(),
            slot_data.timestamp().unix_timestamp() as u64,
        ))
    }
}
