use std::sync::{Arc, Mutex};

use alloy_primitives::U256;
use rbuilder::{
    building::builders::{
        block_building_helper::BiddableUnfinishedBlock,
        UnfinishedBlockBuildingSink as FullUnfinishedBlockBuildingSink,
    },
    live_builder::block_output::{
        bid_value_source::interfaces::BidValueObs,
        bidding::interfaces::SlotBidder as FullSlotBidder,
    },
};

use super::{
    block_registry::BlockRegistry,
    traits::{BlockDescriptor, SlotBidder},
};

/// Adapter from SlotBidder to FullSlotBidder.
/// It uses a block_registry to go from BlockBuildingHelper->BlockId.
#[derive(Debug)]
pub struct SlotBidderAdapter {
    bidder: Arc<dyn SlotBidder>,
    block_registry: Arc<Mutex<BlockRegistry>>,
}

impl SlotBidderAdapter {
    pub fn new(bidder: Arc<dyn SlotBidder>, block_registry: Arc<Mutex<BlockRegistry>>) -> Self {
        Self {
            bidder,
            block_registry,
        }
    }
}

impl FullUnfinishedBlockBuildingSink for SlotBidderAdapter {
    fn new_block(&self, block: BiddableUnfinishedBlock) {
        let true_block_value = block.true_block_value();
        let can_add_payout_tx = block.can_add_payout_tx();
        let block_id = self.block_registry.lock().unwrap().add_block(block);
        self.bidder.new_block(BlockDescriptor {
            true_block_value,
            can_add_payout_tx,
            id: block_id,
        });
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.bidder.can_use_suggested_fee_recipient_as_coinbase()
    }
}

impl BidValueObs for SlotBidderAdapter {
    fn update_new_bid(&self, bid: U256) {
        self.bidder.update_new_bid(bid)
    }
}

impl FullSlotBidder for SlotBidderAdapter {}
