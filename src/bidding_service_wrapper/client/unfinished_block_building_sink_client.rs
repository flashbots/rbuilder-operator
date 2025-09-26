use rbuilder::{
    live_builder::block_output::bidding_service_interface::{
        BuiltBlockDescriptorForSlotBidder, SlotBidder,
    },
    utils::offset_datetime_to_timestamp_us,
};
use time::OffsetDateTime;
use tokio::sync::{mpsc, watch};

use crate::bidding_service_wrapper::{DestroySlotBidderParams, NewBlockParams};

use super::bidding_service_client_adapter::BiddingServiceClientCommand;

/// Implementation of SlotBidder.
/// Commands are forwarded everything to a UnboundedSender<BiddingServiceClientCommand>.
/// BidMaker is wrapped with ... that contains a poling task that makes the bids.
#[derive(Debug)]
pub struct UnfinishedBlockBuildingSinkClient {
    session_id: u64,
    block_sender: watch::Sender<NewBlockParams>,
    /// Used to send DestroySlotBidder on drop.
    destroy_bidder_commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
}

impl UnfinishedBlockBuildingSinkClient {
    pub fn new(
        session_id: u64,
        block_sender: watch::Sender<NewBlockParams>,
        destroy_bidder_commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
    ) -> Self {
        UnfinishedBlockBuildingSinkClient {
            block_sender,
            session_id,
            destroy_bidder_commands_sender,
        }
    }
}

impl SlotBidder for UnfinishedBlockBuildingSinkClient {
    fn notify_new_built_block(&self, block_descriptor: BuiltBlockDescriptorForSlotBidder) {
        let _ = self.block_sender.send(NewBlockParams {
            session_id: self.session_id,
            true_block_value: block_descriptor.true_block_value.as_limbs().to_vec(),
            can_add_payout_tx: true,
            block_id: block_descriptor.id.0,
            creation_time_us: offset_datetime_to_timestamp_us(block_descriptor.creation_time),
            protocol_send_time_us: offset_datetime_to_timestamp_us(OffsetDateTime::now_utc()),
        });
    }
}

impl Drop for UnfinishedBlockBuildingSinkClient {
    fn drop(&mut self) {
        let _ = self.destroy_bidder_commands_sender.send(
            BiddingServiceClientCommand::DestroySlotBidder(DestroySlotBidderParams {
                session_id: self.session_id,
            }),
        );
    }
}
