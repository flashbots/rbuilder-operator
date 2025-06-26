use std::sync::Arc;

use ahash::HashMap;
use mockall::automock;
use parking_lot::Mutex;
use rbuilder::live_builder::block_output::bidding::{
    block_bid_with_stats::BlockBidWithStats,
    interfaces::{
        BiddingServiceWinControl, BlockBidWithStatsObs, LandedBlockInfo,
        MockBiddingServiceWinControl, SlotBlockId,
    },
};

use crate::block_descriptor_bidding::traits::MockUnfinishedBlockBuildingSink;

use super::traits::BiddingService;

#[automock]
pub trait PartialBiddingService {
    fn update_new_landed_blocks_detected(&self, landed_blocks: &[LandedBlockInfo]);
    fn update_failed_reading_new_landed_blocks(&self);
    fn update_new_bid(&self, bid: BlockBidWithStats);
}
/// Custom mock for BiddingService.
/// Usage:
/// Create a MockBiddingService via new.
/// Fill all the internal fields to make MockBiddingService return what you need.
#[derive(Debug)]
pub struct MockBiddingService {
    pub mock_partial_bidding_service: MockPartialBiddingService,
    pub mock_bidding_service_win_control: Arc<MockBiddingServiceWinControl>,
    /// When create_slot_bidder is called the bidder from bidders will be returned.
    /// This is though as a single use thing, don't call create_slot_bidder twice for the same slot/block.
    pub bidders: HashMap<SlotBlockId, Arc<MockUnfinishedBlockBuildingSink>>,
    /// BidMaker we received on create_slot_bidder.
    pub bid_makers:
        Arc<Mutex<HashMap<SlotBlockId, Box<dyn super::traits::BidMaker + Send + Sync>>>>,
}

impl Default for MockBiddingService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBiddingService {
    pub fn new() -> Self {
        MockBiddingService {
            mock_partial_bidding_service: MockPartialBiddingService::new(),
            mock_bidding_service_win_control: Arc::new(MockBiddingServiceWinControl::new()),
            bidders: Default::default(),
            bid_makers: Default::default(),
        }
    }
}

impl BiddingService for MockBiddingService {
    fn create_slot_bidder(
        &self,
        slot_block_id: SlotBlockId,
        _slot_timestamp: time::OffsetDateTime,
        bid_maker: Box<dyn super::traits::BidMaker + Send + Sync>,
        _cancel: tokio_util::sync::CancellationToken,
    ) -> std::sync::Arc<dyn super::traits::UnfinishedBlockBuildingSink> {
        self.bid_makers
            .lock()
            .insert(slot_block_id.clone(), bid_maker);
        self.bidders.get(&slot_block_id).unwrap().clone()
    }

    fn win_control(&self) -> Arc<dyn BiddingServiceWinControl> {
        self.mock_bidding_service_win_control.clone()
    }

    fn update_new_landed_blocks_detected(&self, landed_blocks: &[LandedBlockInfo]) {
        self.mock_partial_bidding_service
            .update_new_landed_blocks_detected(landed_blocks)
    }

    fn update_failed_reading_new_landed_blocks(&self) {
        self.mock_partial_bidding_service
            .update_failed_reading_new_landed_blocks()
    }
}

impl BlockBidWithStatsObs for MockBiddingService {
    fn update_new_bid(&self, bid: BlockBidWithStats) {
        self.mock_partial_bidding_service.update_new_bid(bid)
    }
}
