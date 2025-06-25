use std::sync::{atomic::AtomicBool, Arc};

use rbuilder::utils::offset_datetime_to_timestamp_us;
use tokio::sync::mpsc;

use crate::{
    bidding_service_wrapper::{DestroySlotBidderParams, NewBlockParams},
    block_descriptor_bidding::traits::{BlockDescriptor, UnfinishedBlockBuildingSink},
};

use super::bidding_service_client_adapter::BiddingServiceClientCommand;

/// Implementation of SlotBidder.
/// Commands are forwarded everything to a UnboundedSender<BiddingServiceClientCommand>.
/// BidMaker is wrapped with ... that contains a poling task that makes the bids.
#[derive(Debug)]
pub struct UnfinishedBlockBuildingSinkClient {
    session_id: u64,
    commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
    can_use_suggested_fee_recipient_as_coinbase: Arc<AtomicBool>,
}

impl UnfinishedBlockBuildingSinkClient {
    pub fn new(
        session_id: u64,
        commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
        can_use_suggested_fee_recipient_as_coinbase: Arc<AtomicBool>,
    ) -> Self {
        UnfinishedBlockBuildingSinkClient {
            commands_sender,
            can_use_suggested_fee_recipient_as_coinbase,
            session_id,
        }
    }
}

impl UnfinishedBlockBuildingSink for UnfinishedBlockBuildingSinkClient {
    fn new_block(&self, block_descriptor: BlockDescriptor) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::NewBlock(NewBlockParams {
                session_id: self.session_id,
                true_block_value: block_descriptor.true_block_value().as_limbs().to_vec(),
                can_add_payout_tx: block_descriptor.can_add_payout_tx(),
                block_id: block_descriptor.id().0,
                creation_time_us: offset_datetime_to_timestamp_us(block_descriptor.creation_time()),
            }));
    }

    fn can_use_suggested_fee_recipient_as_coinbase(&self) -> bool {
        self.can_use_suggested_fee_recipient_as_coinbase
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Drop for UnfinishedBlockBuildingSinkClient {
    fn drop(&mut self) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::DestroySlotBidder(
                DestroySlotBidderParams {
                    session_id: self.session_id,
                },
            ));
    }
}
