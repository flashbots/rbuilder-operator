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
    true_block_value: U256,
    can_add_payout_tx: bool,
    id: BlockId,
    /// For metrics
    creation_time: OffsetDateTime,
}

impl BlockDescriptor {
    pub fn new(true_block_value: U256, can_add_payout_tx: bool, id: BlockId) -> Self {
        Self {
            true_block_value,
            can_add_payout_tx,
            id,
            creation_time: OffsetDateTime::now_utc(),
        }
    }

    pub fn new_for_deserialization(
        true_block_value: U256,
        can_add_payout_tx: bool,
        id: BlockId,
        creation_time: OffsetDateTime,
    ) -> Self {
        Self {
            true_block_value,
            can_add_payout_tx,
            id,
            creation_time,
        }
    }

    pub fn true_block_value(&self) -> U256 {
        self.true_block_value
    }

    pub fn can_add_payout_tx(&self) -> bool {
        self.can_add_payout_tx
    }

    pub fn id(&self) -> &BlockId {
        &self.id
    }

    pub fn creation_time(&self) -> OffsetDateTime {
        self.creation_time
    }
}
/// Simplified version of [rbuilder::live_builder::block_output::bidding::interfaces::Bid]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Bid {
    pub block_id: BlockId,
    pub payout_tx_value: Option<U256>,
    pub seen_competition_bid: Option<U256>,
    /// When this bid is a reaction so some event (eg: new block, new competition bid) we put here
    /// the creation time of that event so we can measure our reaction time.
    pub trigger_creation_time: Option<OffsetDateTime>,
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
