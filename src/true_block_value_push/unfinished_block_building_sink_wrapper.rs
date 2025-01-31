use std::sync::Arc;

use super::best_true_value_pusher::{BestTrueValue, BestTrueValueCell};
use rbuilder::{
    building::builders::{
        block_building_helper::{BiddableUnfinishedBlock, BlockBuildingHelper},
        UnfinishedBlockBuildingSink,
    },
    live_builder::block_output::bid_value_source::{
        best_bid_sync_source::BestBidSyncSource, interfaces::BidValueSource,
    },
};

/// Wraps a UnfinishedBlockBuildingSink updating all the new block info on a BestTrueValueCell.
/// Updates on every new_block but not on competition best bid changes.
pub struct UnfinishedBlockBuildingSinkWrapper {
    sink: Arc<dyn UnfinishedBlockBuildingSink>,
    best_local_value: BestTrueValueCell,
    best_bid_sync_source: BestBidSyncSource,
    block_number: u64,
    slot_number: u64,
    slot_timestamp: u64,
}

impl std::fmt::Debug for UnfinishedBlockBuildingSinkWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnfinishedBlockBuildingSinkWrapper")
            .field("block_number", &self.block_number)
            .field("slot_number", &self.slot_number)
            .finish_non_exhaustive()
    }
}

impl UnfinishedBlockBuildingSinkWrapper {
    pub fn new(
        sink: Arc<dyn UnfinishedBlockBuildingSink>,
        best_local_value: BestTrueValueCell,
        competition_bid_value_source: Arc<dyn BidValueSource + Send + Sync>,
        block_number: u64,
        slot_number: u64,
        slot_timestamp: u64,
    ) -> Self {
        Self {
            sink,
            best_local_value,
            best_bid_sync_source: BestBidSyncSource::new(
                competition_bid_value_source,
                block_number,
                slot_number,
            ),
            block_number,
            slot_number,
            slot_timestamp,
        }
    }
}

impl UnfinishedBlockBuildingSink for UnfinishedBlockBuildingSinkWrapper {
    /// Update self.best_local_value and forward to self.sink.
    fn new_block(&self, block: BiddableUnfinishedBlock) {
        let best_true_value = BestTrueValue::new(
            self.block_number,
            self.slot_number,
            block.true_block_value(),
            self.best_bid_sync_source
                .best_bid_value()
                .unwrap_or_default(),
            self.slot_timestamp,
        );
        self.best_local_value.update_value_safe(best_true_value);
        self.sink.new_block(block);
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.sink.can_use_suggested_fee_recipient_as_coinbase()
    }
}
