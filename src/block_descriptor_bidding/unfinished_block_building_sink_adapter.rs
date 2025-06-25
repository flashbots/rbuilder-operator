use std::sync::Arc;

use parking_lot::Mutex;
use rbuilder::building::builders::{
    block_building_helper::BiddableUnfinishedBlock,
    UnfinishedBlockBuildingSink as FullUnfinishedBlockBuildingSink,
};

use super::{
    block_registry::BlockRegistry,
    traits::{BlockDescriptor, UnfinishedBlockBuildingSink},
};

/// Adapter from UnfinishedBlockBuildingSink to FullUnfinishedBlockBuildingSink.
/// It uses a block_registry to go from BlockBuildingHelper->BlockId.
#[derive(Debug)]
pub struct UnfinishedBlockBuildingSinkAdapter {
    bidder: Arc<dyn UnfinishedBlockBuildingSink>,
    block_registry: Arc<Mutex<BlockRegistry>>,
}

impl UnfinishedBlockBuildingSinkAdapter {
    pub fn new(
        bidder: Arc<dyn UnfinishedBlockBuildingSink>,
        block_registry: Arc<Mutex<BlockRegistry>>,
    ) -> Self {
        Self {
            bidder,
            block_registry,
        }
    }
}

impl FullUnfinishedBlockBuildingSink for UnfinishedBlockBuildingSinkAdapter {
    fn new_block(&self, block: BiddableUnfinishedBlock) {
        let true_block_value = block.true_block_value();
        let can_add_payout_tx = block.can_add_payout_tx();
        let block_id = self.block_registry.lock().add_block(block);
        self.bidder.new_block(BlockDescriptor::new(
            true_block_value,
            can_add_payout_tx,
            block_id,
        ));
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.bidder.can_use_suggested_fee_recipient_as_coinbase()
    }
}
