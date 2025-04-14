use std::sync::{atomic::AtomicBool, Arc};

use alloy_primitives::U256;
use rbuilder::live_builder::block_output::bid_value_source::interfaces::BidValueObs;
use tokio::sync::mpsc;

use crate::{
    bidding_service_wrapper::{DestroySlotBidderParams, NewBlockParams, UpdateNewBidParams},
    block_descriptor_bidding::traits::{BlockDescriptor, SlotBidder, UnfinishedBlockBuildingSink},
};

use super::bidding_service_client_adapter::BiddingServiceClientCommand;

/// Implementation of SlotBidder.
/// Commands are forwarded everything to a UnboundedSender<BiddingServiceClientCommand>.
/// BidMaker is wrapped with ... that contains a poling task that makes the bids.
#[derive(Debug)]
pub struct SlotBidderClient {
    block: u64,
    slot: u64,
    commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
    can_use_suggested_fee_recipient_as_coinbase: Arc<AtomicBool>,
}

impl SlotBidderClient {
    pub fn new(
        block: u64,
        slot: u64,
        commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
        can_use_suggested_fee_recipient_as_coinbase: Arc<AtomicBool>,
    ) -> Self {
        SlotBidderClient {
            commands_sender,
            block,
            slot,
            can_use_suggested_fee_recipient_as_coinbase,
        }
    }
}

impl UnfinishedBlockBuildingSink for SlotBidderClient {
    fn new_block(&self, block_descriptor: BlockDescriptor) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::NewBlock(NewBlockParams {
                block: self.block,
                slot: self.slot,
                true_block_value: block_descriptor.true_block_value().as_limbs().to_vec(),
                can_add_payout_tx: block_descriptor.can_add_payout_tx(),
                block_id: block_descriptor.id().0,
            }));
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.can_use_suggested_fee_recipient_as_coinbase
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl BidValueObs for SlotBidderClient {
    fn update_new_bid(&self, bid: U256) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::UpdateNewBid(
                UpdateNewBidParams {
                    block: self.block,
                    slot: self.slot,
                    bid: bid.as_limbs().to_vec(),
                },
            ));
    }
}

impl SlotBidder for SlotBidderClient {}

impl Drop for SlotBidderClient {
    fn drop(&mut self) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::DestroySlotBidder(
                DestroySlotBidderParams {
                    block: self.block,
                    slot: self.slot,
                },
            ));
    }
}
