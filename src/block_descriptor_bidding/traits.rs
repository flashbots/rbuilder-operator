//! To keep comment consistency comments from the original elements are not copied.
use mockall::automock;
use std::sync::Arc;

use alloy_primitives::U256;
use rbuilder::live_builder::block_output::{
    bid_value_source::interfaces::BidValueObs,
    bidding::interfaces::{BiddingServiceWinControl, LandedBlockInfo},
};
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

/// This design abuses of the fact that the u64 is going to come handy on RPC serialization.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BlockId(pub u64);

/// Selected information coming from a BlockBuildingHelper.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BlockDescriptor {
    pub true_block_value: U256,
    pub can_add_payout_tx: bool,
    pub id: BlockId,
}

/// Simplified version of [rbuilder::live_builder::block_output::bidding::interfaces::Bid]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Bid {
    pub block_id: BlockId,
    pub payout_tx_value: Option<U256>,
    pub seen_competition_bid: Option<U256>,
}

/// Simplified version of [rbuilder::live_builder::block_output::bidding::interfaces::BidMaker]
#[automock]
pub trait BidMaker: std::fmt::Debug {
    fn send_bid(&self, bid: Bid);
}

/// Simplified version of [rbuilder::building::builders::UnfinishedBlockBuildingSink]
#[automock]
pub trait UnfinishedBlockBuildingSink: std::fmt::Debug + Send + Sync {
    fn new_block(&self, block_descriptor: BlockDescriptor);
    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool;
}

/// Simplified version of [rbuilder::live_builder::block_output::bidding::interfaces::SlotBidder]
pub trait SlotBidder: UnfinishedBlockBuildingSink + BidValueObs {}

/// Simplified version of [rbuilder::live_builder::block_output::bidding::interfaces::BiddingService]
pub trait BiddingService: std::fmt::Debug + Send + Sync {
    fn create_slot_bidder(
        &mut self,
        block: u64,
        slot: u64,
        slot_timestamp: OffsetDateTime,
        bid_maker: Box<dyn BidMaker + Send + Sync>,
        cancel: CancellationToken,
    ) -> Arc<dyn SlotBidder>;

    // Consider moving these 3 func could be on a parent trait (I didn't want to modify the original BiddingService yet).

    fn win_control(&self) -> Arc<dyn BiddingServiceWinControl>;

    fn update_new_landed_blocks_detected(&mut self, landed_blocks: &[LandedBlockInfo]);

    fn update_failed_reading_new_landed_blocks(&mut self);
}
