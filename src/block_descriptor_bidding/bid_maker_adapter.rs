use std::sync::{Arc, Mutex};

use super::{
    block_registry::BlockRegistry,
    traits::{Bid, BidMaker},
};
use rbuilder::live_builder::block_output::bidding::interfaces::Bid as FullBid;
use rbuilder::live_builder::block_output::bidding::interfaces::BidMaker as FullBidMaker;
use tracing::error;

/// Adapts by translating BlockId -> BlockBuildingHelper via a block_registry
#[derive(Debug)]
pub struct BidMakerAdapter {
    full_bid_maker: Box<dyn FullBidMaker + Send + Sync>,
    block_registry: Arc<Mutex<BlockRegistry>>,
}

impl BidMakerAdapter {
    pub fn new(
        full_bid_maker: Box<dyn FullBidMaker + Send + Sync>,
        block_registry: Arc<Mutex<BlockRegistry>>,
    ) -> Self {
        Self {
            full_bid_maker,
            block_registry,
        }
    }
}

impl BidMaker for BidMakerAdapter {
    fn send_bid(&self, bid: Bid) {
        match self
            .block_registry
            .lock()
            .unwrap()
            .get_block_clon(bid.block_id)
        {
            Some(block) => self.full_bid_maker.send_bid(FullBid::new(
                block,
                bid.payout_tx_value,
                bid.seen_competition_bid,
            )),
            None => error!("Tried to bid with lost block"),
        }
    }
}
