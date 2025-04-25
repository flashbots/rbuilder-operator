use std::sync::Arc;

use ahash::HashMap;
use mockall::automock;
use parking_lot::Mutex;
use rbuilder::live_builder::block_output::bidding::interfaces::{
    BiddingServiceWinControl, LandedBlockInfo, MockBiddingServiceWinControl,
};

use super::{mock_slot_bidder::MockSlotBidder, traits::BiddingService, SlotBidderId};

#[automock]
pub trait PartialBiddingService {
    fn update_new_landed_blocks_detected(&mut self, landed_blocks: &[LandedBlockInfo]);
    fn update_failed_reading_new_landed_blocks(&mut self);
}

/// Custom mock for BiddingService.
/// Usage:
/// Create a MockBiddingService via new.
///
#[derive(Debug)]
pub struct MockBiddingService {
    pub mock_partial_bidding_service: MockPartialBiddingService,
    pub mock_bidding_service_win_control: Arc<MockBiddingServiceWinControl>,
    pub bidders: HashMap<SlotBidderId, Arc<MockSlotBidder>>,
    pub bid_makers:
        Arc<Mutex<HashMap<SlotBidderId, Box<dyn super::traits::BidMaker + Send + Sync>>>>,
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
        let id = SlotBidderId { block, slot };
        self.bid_makers.lock().insert(id.clone(), bid_maker);
        self.bidders.get(&id).unwrap().clone()
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
