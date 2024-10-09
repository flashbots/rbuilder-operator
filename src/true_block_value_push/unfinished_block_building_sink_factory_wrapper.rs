use std::sync::Arc;

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
    best_true_value_redis::{BestTrueValueCell, BestTrueValueRedisSync},
    unfinished_block_building_sink_wrapper::UnfinishedBlockBuildingSinkWrapper,
};

/// Wrapper over a UnfinishedBlockBuildingSinkFactory that wraps the generated [UnfinishedBlockBuildingSink]s so they push the tbv to redis.
#[derive(Debug)]
pub struct UnfinishedBlockBuildingSinkFactoryWrapper {
    factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
    best_local_value: BestTrueValueCell,
    competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to init redis connection : {0}")]
    RedisError(#[from] RedisError),
}

pub type Result<T> = core::result::Result<T, Error>;

impl UnfinishedBlockBuildingSinkFactoryWrapper {
    // Constructor for UnfinishedBlockBuildingSinkFactoryWrapper
    pub fn new(
        factory: Box<dyn UnfinishedBlockBuildingSinkFactory>,
        competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
        tbv_push_redis_url: String,
        tbv_push_redis_channel: String,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let best_local_value = BestTrueValueCell::default();
        let best_true_value_redis = redis::Client::open(tbv_push_redis_url)?;

        let redis_sync = BestTrueValueRedisSync::new(
            best_local_value.clone(),
            best_true_value_redis,
            tbv_push_redis_channel,
            cancellation_token,
        );
        std::thread::spawn(move || redis_sync.run_push_task());

        Ok(UnfinishedBlockBuildingSinkFactoryWrapper {
            factory,
            competition_bid_value_source,
            best_local_value,
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
