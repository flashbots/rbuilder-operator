use alloy_primitives::U256;
use rbuilder::live_builder::block_output::bid_value_source::interfaces::{
    BidValueObs, MockBidValueObs,
};

use super::traits::{
    BlockDescriptor, MockUnfinishedBlockBuildingSink, SlotBidder, UnfinishedBlockBuildingSink,
};

/// Manually implemented since #[automock] seems to have a problem with trait inherence.
#[derive(Debug)]
pub struct MockSlotBidder {
    pub unfinished_sink: MockUnfinishedBlockBuildingSink,
    pub bid_value_obs: MockBidValueObs,
}

impl MockSlotBidder {
    pub fn new() -> Self {
        Self {
            unfinished_sink: MockUnfinishedBlockBuildingSink::new(),
            bid_value_obs: MockBidValueObs::new(),
        }
    }
}

impl Default for MockSlotBidder {
    fn default() -> Self {
        Self::new()
    }
}

impl UnfinishedBlockBuildingSink for MockSlotBidder {
    fn new_block(&self, block_descriptor: BlockDescriptor) {
        self.unfinished_sink.new_block(block_descriptor)
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.unfinished_sink
            .can_use_suggested_fee_recipient_as_coinbase()
    }
}

impl BidValueObs for MockSlotBidder {
    fn update_new_bid(&self, bid: U256) {
        self.bid_value_obs.update_new_bid(bid)
    }
}

impl SlotBidder for MockSlotBidder {}
