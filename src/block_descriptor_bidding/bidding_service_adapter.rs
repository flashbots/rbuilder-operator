//! We use FullXXX aliases for the original versions using full block info.
use std::sync::Arc;

use parking_lot::Mutex;
use rbuilder::{
    building::builders::UnfinishedBlockBuildingSink as FullUnfinishedBlockBuildingSink,
    live_builder::block_output::bidding::{
        block_bid_with_stats::BlockBidWithStats,
        interfaces::{
            BidMaker as FullBidMaker, BiddingService as FullBiddingService,
            BiddingServiceWinControl, BlockBidWithStatsObs, LandedBlockInfo, SlotBlockId,
        },
    },
};

use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use crate::block_descriptor_bidding::{
    bid_maker_adapter::BidMakerAdapter, block_registry::BlockRegistry,
};

use super::{
    traits::BiddingService,
    unfinished_block_building_sink_adapter::UnfinishedBlockBuildingSinkAdapter,
};

/// We need to make sure that a block is not deleted between the bidder sees it and tries to use it.
/// We assume that after getting a better block the bidder will stop using the previews one so he can only use
/// the previous one if he was already making the bid.
/// Worst case is making a bid via RPC, that takes no more than .5ms (usually .15ms).
/// Generating a new block usually takes way more than .5ms but let's assume 10 blocks in .5ms.
/// Assuming 5 building algorithms worst case would be 10 x 5 = 50. To play it safe we go with 100.
/// This shouldn't be a lot of mem, just a few MB.
const MAX_ACTIVE_BLOCKS: usize = 100;

/// Adapter from simplified world to full world.
#[derive(Debug)]
pub struct BiddingServiceAdapter {
    bidding_service: Arc<dyn BiddingService>,
}

impl BiddingServiceAdapter {
    pub fn new(bidding_service: Arc<dyn BiddingService>) -> Self {
        Self { bidding_service }
    }
}

impl FullBiddingService for BiddingServiceAdapter {
    fn create_slot_bidder(
        &self,
        slot_block_id: SlotBlockId,
        slot_timestamp: OffsetDateTime,
        bid_maker: Box<dyn FullBidMaker + Send + Sync>,
        cancel: CancellationToken,
    ) -> Arc<dyn FullUnfinishedBlockBuildingSink> {
        let block_registry = Arc::new(Mutex::new(BlockRegistry::new(MAX_ACTIVE_BLOCKS)));
        let wrapped_bid_maker = Box::new(BidMakerAdapter::new(bid_maker, block_registry.clone()));

        let bidder = self.bidding_service.create_slot_bidder(
            slot_block_id,
            slot_timestamp,
            wrapped_bid_maker,
            cancel,
        );
        Arc::new(UnfinishedBlockBuildingSinkAdapter::new(
            bidder,
            block_registry,
        ))
    }

    fn win_control(&self) -> Arc<dyn BiddingServiceWinControl> {
        self.bidding_service.win_control()
    }

    fn update_new_landed_blocks_detected(&self, landed_blocks: &[LandedBlockInfo]) {
        self.bidding_service
            .update_new_landed_blocks_detected(landed_blocks)
    }

    fn update_failed_reading_new_landed_blocks(&self) {
        self.bidding_service
            .update_failed_reading_new_landed_blocks()
    }
}

impl BlockBidWithStatsObs for BiddingServiceAdapter {
    fn update_new_bid(&self, bid: BlockBidWithStats) {
        self.bidding_service.update_new_bid(bid)
    }
}
