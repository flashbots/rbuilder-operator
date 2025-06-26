use alloy_primitives::U256;
use rbuilder::{
    live_builder::block_output::bidding::{
        block_bid_with_stats::BlockBidWithStats,
        interfaces::{
            BiddingServiceWinControl, BlockBidWithStatsObs, LandedBlockInfo as RealLandedBlockInfo,
            SlotBlockId,
        },
    },
    utils::{build_info::Version, timestamp_us_to_offset_datetime},
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tracing::error;

use crate::{
    bidding_service_wrapper::{
        bidding_service_client::BiddingServiceClient,
        conversion::{real2rpc_block_bid, real2rpc_block_hash, real2rpc_landed_block_info},
        CreateSlotBidderParams, DestroySlotBidderParams, Empty, LandedBlocksParams,
        MustWinBlockParams, NewBlockParams, UpdateNewBidParams,
    },
    block_descriptor_bidding::traits::{
        Bid, BidMaker, BiddingService, BlockId, UnfinishedBlockBuildingSink,
    },
    metrics::set_bidding_service_version,
};

use super::unfinished_block_building_sink_client::UnfinishedBlockBuildingSinkClient;

pub struct CreateSlotBidderCommandData {
    params: CreateSlotBidderParams,
    bid_maker: Box<dyn BidMaker + Send + Sync>,
    cancel: tokio_util::sync::CancellationToken,
    can_use_suggested_fee_recipient_as_coinbase: Arc<AtomicBool>,
}

#[allow(clippy::large_enum_variant)]
pub enum BiddingServiceClientCommand {
    CreateSlotBidder(CreateSlotBidderCommandData),
    NewBlock(NewBlockParams),
    UpdateNewBid(UpdateNewBidParams),
    MustWinBlock(MustWinBlockParams),
    UpdateNewLandedBlocksDetected(LandedBlocksParams),
    UpdateFailedReadingNewLandedBlocks,
    DestroySlotBidder(DestroySlotBidderParams),
}

/// Adapts [BiddingServiceClient] to [BiddingService].
/// To adapt sync world ([BiddingService]) to async ([BiddingServiceClient]) it receives commands via a channel (commands_sender)
/// which is handled by a tokio task.
/// It creates a UnfinishedBlockBuildingSinkClient implementing UnfinishedBlockBuildingSink per create_slot_bidder call.
/// For each UnfinishedBlockBuildingSinkClient created a task is created to poll callbacks (eg: bids and can_use_suggested_fee_recipient_as_coinbase updates).
/// The created UnfinishedBlockBuildingSinkClient forwards all calls to the BiddingServiceClientAdapter as commands.
#[derive(Debug)]
pub struct BiddingServiceClientAdapter {
    commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
    win_control: Arc<dyn BiddingServiceWinControl>,
    last_session_id: AtomicU64,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to connect : {0}")]
    TonicTrasport(#[from] tonic::transport::Error),
    #[error("RPC error : {0}")]
    TonicStatus(#[from] tonic::Status),
    #[error("Initialization failed  : {0}")]
    InitFailed(tonic::Status),
}

pub type Result<T> = core::result::Result<T, Error>;

impl BiddingServiceClientAdapter {
    /// @Remove async and reconnect on all create_slot_bidder calls.
    pub async fn new(
        uds_path: &str,
        landed_blocks_history: &[RealLandedBlockInfo],
    ) -> Result<Self> {
        let commands_sender = Self::init_sender_task(uds_path, landed_blocks_history).await?;
        let win_control = Arc::new(BiddingServiceWinControlAdapter {
            commands_sender: commands_sender.clone(),
        });
        Ok(Self {
            commands_sender,
            win_control,
            last_session_id: AtomicU64::new(0),
        })
    }

    fn new_session_id(&self) -> u64 {
        self.last_session_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn init_sender_task(
        uds_path: &str,
        landed_blocks_history: &[RealLandedBlockInfo],
    ) -> Result<mpsc::UnboundedSender<BiddingServiceClientCommand>> {
        let uds_path = uds_path.to_string();
        // Url us dummy but needed to create the Endpoint.
        let channel = Endpoint::try_from("http://[::]:50051")
            .unwrap()
            .connect_with_connector(service_fn(move |_: Uri| {
                // Connect to a Uds socket
                let path = PathBuf::from(uds_path.clone());
                tokio::net::UnixStream::connect(path)
            }))
            .await?;
        // Create a client
        let mut client = BiddingServiceClient::new(channel);
        let init_params = LandedBlocksParams {
            landed_block_info: landed_blocks_history
                .iter()
                .map(real2rpc_landed_block_info)
                .collect(),
        };
        let bidding_service_version = client
            .initialize(init_params)
            .await
            .map_err(Error::InitFailed)?;
        let bidding_service_version = bidding_service_version.into_inner();
        set_bidding_service_version(Version {
            git_commit: bidding_service_version.git_commit,
            git_ref: bidding_service_version.git_ref,
            build_time_utc: bidding_service_version.build_time_utc,
        });
        let (commands_sender, mut rx) = mpsc::unbounded_channel::<BiddingServiceClientCommand>();
        // Spawn a task to execute received futures
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    BiddingServiceClientCommand::CreateSlotBidder(create_slot_data) => {
                        Self::create_slot_bidder(&mut client, create_slot_data).await;
                    }
                    BiddingServiceClientCommand::NewBlock(new_block_params) => {
                        Self::handle_error(client.new_block(new_block_params).await);
                    }
                    BiddingServiceClientCommand::UpdateNewBid(update_new_bid_params) => {
                        Self::handle_error(client.update_new_bid(update_new_bid_params).await);
                    }
                    BiddingServiceClientCommand::MustWinBlock(must_win_block_params) => {
                        Self::handle_error(client.must_win_block(must_win_block_params).await);
                    }
                    BiddingServiceClientCommand::UpdateNewLandedBlocksDetected(params) => {
                        Self::handle_error(client.update_new_landed_blocks_detected(params).await);
                    }
                    BiddingServiceClientCommand::UpdateFailedReadingNewLandedBlocks => {
                        Self::handle_error(
                            client
                                .update_failed_reading_new_landed_blocks(Empty {})
                                .await,
                        );
                    }
                    BiddingServiceClientCommand::DestroySlotBidder(destroy_slot_bidder_params) => {
                        Self::handle_error(
                            client.destroy_slot_bidder(destroy_slot_bidder_params).await,
                        );
                    }
                }
            }
        });
        Ok(commands_sender)
    }

    fn parse_option_u256(limbs: Vec<u64>) -> Option<U256> {
        if limbs.is_empty() {
            None
        } else {
            Some(U256::from_limbs_slice(&limbs))
        }
    }

    /// Calls create_slot_bidder via RPC to init the bidder.
    async fn create_slot_bidder(
        client: &mut BiddingServiceClient<Channel>,
        create_slot_bidder_data: CreateSlotBidderCommandData,
    ) {
        match client
            .create_slot_bidder(create_slot_bidder_data.params)
            .await
        {
            Ok(response) => {
                let mut stream = response.into_inner();
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = create_slot_bidder_data.cancel.cancelled() => {
                                return;
                            }
                            callback = stream.next() => {
                                if let Some(Ok(callback)) = callback {
                                    if let Some(bid) = callback.bid {
                                        let payout_tx_value = Self::parse_option_u256(bid.payout_tx_value);
                                        let seen_competition_bid = Self::parse_option_u256(bid.seen_competition_bid);
                                        let trigger_creation_time = bid.trigger_creation_time_us.map(timestamp_us_to_offset_datetime);
                                        create_slot_bidder_data.bid_maker.send_bid(Bid{block_id:BlockId(bid.block_id),payout_tx_value,seen_competition_bid, trigger_creation_time });
                                    } else if let Some(can_use_suggested_fee_recipient_as_coinbase_change) = callback.can_use_suggested_fee_recipient_as_coinbase_change {
                                        create_slot_bidder_data.can_use_suggested_fee_recipient_as_coinbase.store(can_use_suggested_fee_recipient_as_coinbase_change,std::sync::atomic::Ordering::SeqCst);
                                    }
                                }
                                else {
                                    return;
                                }
                            }
                        }
                    }
                });
            }
            Err(err) => {
                Self::handle_error(Err(err));
            }
        };
    }

    /// If error logs it.
    /// return result is error
    fn handle_error(result: tonic::Result<tonic::Response<Empty>>) -> bool {
        if let Err(error) = &result {
            error!(error=?error,"RPC call error, killing process so it reconnects");
            std::process::exit(1);
        } else {
            false
        }
    }
}

impl BiddingService for BiddingServiceClientAdapter {
    fn create_slot_bidder(
        &self,
        slot_block_id: SlotBlockId,
        slot_timestamp: time::OffsetDateTime,
        bid_maker: Box<dyn BidMaker + Send + Sync>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> Arc<dyn UnfinishedBlockBuildingSink> {
        // This default will be immediately changed by a callback.
        let can_use_suggested_fee_recipient_as_coinbase = Arc::new(AtomicBool::new(false));
        let session_id = self.new_session_id();
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::CreateSlotBidder(
                CreateSlotBidderCommandData {
                    params: CreateSlotBidderParams {
                        block: slot_block_id.block(),
                        slot: slot_block_id.slot(),
                        parent_hash: real2rpc_block_hash(*slot_block_id.parent_block_hash()),
                        session_id,
                        slot_timestamp: slot_timestamp.unix_timestamp(),
                    },
                    bid_maker,
                    cancel,
                    can_use_suggested_fee_recipient_as_coinbase:
                        can_use_suggested_fee_recipient_as_coinbase.clone(),
                },
            ));
        Arc::new(UnfinishedBlockBuildingSinkClient::new(
            session_id,
            self.commands_sender.clone(),
            can_use_suggested_fee_recipient_as_coinbase,
        ))
    }

    fn win_control(&self) -> Arc<dyn BiddingServiceWinControl> {
        self.win_control.clone()
    }

    fn update_new_landed_blocks_detected(&self, landed_blocks: &[RealLandedBlockInfo]) {
        let param = LandedBlocksParams {
            landed_block_info: landed_blocks
                .iter()
                .map(real2rpc_landed_block_info)
                .collect(),
        };
        let _ =
            self.commands_sender
                .send(BiddingServiceClientCommand::UpdateNewLandedBlocksDetected(
                    param,
                ));
    }

    fn update_failed_reading_new_landed_blocks(&self) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::UpdateFailedReadingNewLandedBlocks);
    }
}

impl BlockBidWithStatsObs for BiddingServiceClientAdapter {
    fn update_new_bid(&self, bid_with_stats: BlockBidWithStats) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::UpdateNewBid(
                real2rpc_block_bid(bid_with_stats),
            ));
    }
}
#[derive(Debug)]
struct BiddingServiceWinControlAdapter {
    commands_sender: mpsc::UnboundedSender<BiddingServiceClientCommand>,
}

impl BiddingServiceWinControl for BiddingServiceWinControlAdapter {
    fn must_win_block(&self, block: u64) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::MustWinBlock(
                MustWinBlockParams { block },
            ));
    }
}
