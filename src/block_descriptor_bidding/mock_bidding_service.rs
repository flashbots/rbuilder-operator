use std::sync::Arc;

use ahash::HashMap;
use mockall::automock;
use parking_lot::Mutex;
use rbuilder::live_builder::block_output::bidding::interfaces::{
    BiddingServiceWinControl, LandedBlockInfo, MockBiddingServiceWinControl,
};

use super::{mock_slot_bidder::MockSlotBidder, traits::BiddingService};

#[automock]
pub trait PartialBiddingService {
    fn update_new_landed_blocks_detected(&mut self, landed_blocks: &[LandedBlockInfo]);
    fn update_failed_reading_new_landed_blocks(&mut self);
}

/// Slot + block to use on maps
#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct SlotAndBlock {
    pub block: u64,
    pub slot: u64,
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
    pub bidders: HashMap<SlotAndBlock, Arc<MockSlotBidder>>,
    /// BidMaker we received on create_slot_bidder.
    pub bid_makers:
        Arc<Mutex<HashMap<SlotAndBlock, Box<dyn super::traits::BidMaker + Send + Sync>>>>,
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
        &mut self,
        block: u64,
        slot: u64,
        _slot_timestamp: time::OffsetDateTime,
        bid_maker: Box<dyn super::traits::BidMaker + Send + Sync>,
        _cancel: tokio_util::sync::CancellationToken,
    ) -> std::sync::Arc<dyn super::traits::SlotBidder> {
        let slot_block = SlotAndBlock { block, slot };
        self.bid_makers.lock().insert(slot_block.clone(), bid_maker);
        self.bidders.get(&slot_block).unwrap().clone()
    }

    fn win_control(&self) -> Arc<dyn BiddingServiceWinControl> {
        self.mock_bidding_service_win_control.clone()
    }

    fn update_new_landed_blocks_detected(&mut self, landed_blocks: &[LandedBlockInfo]) {
        self.mock_partial_bidding_service
            .update_new_landed_blocks_detected(landed_blocks)
    }

    fn update_failed_reading_new_landed_blocks(&mut self) {
        self.mock_partial_bidding_service
            .update_failed_reading_new_landed_blocks()
    }
}
