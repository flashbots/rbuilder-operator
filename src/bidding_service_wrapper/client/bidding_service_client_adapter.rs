use alloy_primitives::U256;
use rbuilder::{
    building::builders::BuiltBlockId,
    live_builder::block_output::bidding_service_interface::{
        BiddingService, BlockSealInterfaceForSlotBidder, LandedBlockInfo as RealLandedBlockInfo,
        ScrapedRelayBlockBidWithStats, SlotBidder, SlotBidderSealBidCommand, SlotBlockId,
    },
    utils::{
        build_info::Version, offset_datetime_to_timestamp_us, timestamp_us_to_offset_datetime,
    },
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use time::OffsetDateTime;
use tokio::sync::{mpsc, watch};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tracing::{error, warn};

use crate::{
    bidding_service_wrapper::{
        bidding_service_client::BiddingServiceClient,
        conversion::{real2rpc_block_bid, real2rpc_block_hash, real2rpc_landed_block_info},
        CreateSlotBidderParams, DestroySlotBidderParams, Empty, LandedBlocksParams,
        MustWinBlockParams, NewBid, NewBlockParams, UpdateNewBidsParams,
    },
    metrics::set_bidding_service_version,
};

use super::unfinished_block_building_sink_client::UnfinishedBlockBuildingSinkClient;

pub struct CreateSlotBidderCommandData {
    params: CreateSlotBidderParams,
    block_seal_handle: Box<dyn BlockSealInterfaceForSlotBidder + Send + Sync>,
    cancel: tokio_util::sync::CancellationToken,
}

impl std::fmt::Debug for CreateSlotBidderCommandData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateSlotBidderCommandData")
            .field("params", &self.params)
            .finish()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum BiddingServiceClientCommand {
    CreateSlotBidder(CreateSlotBidderCommandData),
    NewBlock(NewBlockParams),
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
    bids_sender: mpsc::UnboundedSender<NewBid>,
    block_sender: watch::Sender<NewBlockParams>,
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
/// Max number of bids to send in a single call to update_new_bids.
const SEND_BIDS_BULK_MAX: usize = 1;
impl BiddingServiceClientAdapter {
    /// @Remove async and reconnect on all create_slot_bidder calls.
    pub async fn new(
        uds_path: &str,
        landed_blocks_history: &[RealLandedBlockInfo],
    ) -> Result<Self> {
        let (commands_sender, block_sender, bids_sender) =
            Self::init_sender_task(uds_path, landed_blocks_history).await?;
        Ok(Self {
            commands_sender,
            block_sender,
            last_session_id: AtomicU64::new(0),
            bids_sender,
        })
    }

    fn new_session_id(&self) -> u64 {
        self.last_session_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn init_sender_task(
        uds_path: &str,
        landed_blocks_history: &[RealLandedBlockInfo],
    ) -> Result<(
        mpsc::UnboundedSender<BiddingServiceClientCommand>,
        watch::Sender<NewBlockParams>,
        mpsc::UnboundedSender<NewBid>,
    )> {
        let (block_sender, mut block_recv) = watch::channel(NewBlockParams {
            session_id: 0,
            true_block_value: vec![],
            can_add_payout_tx: false,
            block_id: 0,
            creation_time_us: 0,
            protocol_send_time_us: 0,
        });
        let uds_path = uds_path.to_string();
        // Url us dummy but needed to create the Endpoint.
        let channel = Endpoint::try_from("http://[::]:50051")
            .unwrap()
            .tcp_nodelay(true)
            .connect_with_connector(service_fn(move |_: Uri| {
                // Connect to a Uds socket
                let path = PathBuf::from(uds_path.clone());
                warn!("DX connect to unix socket");
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
        let (bids_sender, mut bids_recv) = mpsc::unbounded_channel::<NewBid>();
        // Spawn a task to execute received futures
        tokio::spawn(async move {
            loop {
                let mut new_bids = Vec::new();
                tokio::select! {
                    command = rx.recv() => {
                        if let Some(command) = command {
                            Self::handle_command(command, &mut client).await;
                        }else{
                            return;
                        }
                    }
                    bids = bids_recv.recv_many(&mut new_bids, SEND_BIDS_BULK_MAX) => {
                        if bids == 0 {
                            return;
                        }
                        let update_new_bids_params = UpdateNewBidsParams {
                            bids: new_bids,
                            protocol_send_time_us: offset_datetime_to_timestamp_us(OffsetDateTime::now_utc()),
                        };
                        Self::handle_error(client.update_new_bids(update_new_bids_params).await);
                    }
                    block_changed_res = block_recv.changed() => {
                        match block_changed_res {
                            Ok(()) => {
                                let block = (*block_recv.borrow_and_update()).clone();
                                Self::handle_new_block(block, &mut client).await;
                            }
                            Err(_) => {
                                return;
                            }
                        }
                    }
                }
            }
        });
        Ok((commands_sender, block_sender, bids_sender))
    }

    async fn handle_new_block(
        mut new_block_params: NewBlockParams,
        client: &mut BiddingServiceClient<Channel>,
    ) {
        let traveling_us = offset_datetime_to_timestamp_us(OffsetDateTime::now_utc())
            - new_block_params.protocol_send_time_us;
        if traveling_us > 1000 {
            warn!(
                traveling_us,
                "DX init_sender_task NewBlock queued time too long",
            );
        }
        new_block_params.protocol_send_time_us =
            offset_datetime_to_timestamp_us(OffsetDateTime::now_utc());
        Self::handle_error(client.new_block(new_block_params).await);
    }

    async fn handle_command(
        command: BiddingServiceClientCommand,
        client: &mut BiddingServiceClient<Channel>,
    ) {
        let now = Instant::now();
        let command_name = match command {
            BiddingServiceClientCommand::CreateSlotBidder(create_slot_data) => {
                Self::create_slot_bidder(client, create_slot_data).await;
                "CreateSlotBidder"
            }
            BiddingServiceClientCommand::NewBlock(new_block_params) => {
                Self::handle_new_block(new_block_params, client).await;
                "NewBlock"
            }
            BiddingServiceClientCommand::MustWinBlock(must_win_block_params) => {
                Self::handle_error(client.must_win_block(must_win_block_params).await);
                "MustWinBlock"
            }
            BiddingServiceClientCommand::UpdateNewLandedBlocksDetected(params) => {
                Self::handle_error(client.update_new_landed_blocks_detected(params).await);
                "UpdateNewLandedBlocksDetected"
            }
            BiddingServiceClientCommand::UpdateFailedReadingNewLandedBlocks => {
                Self::handle_error(
                    client
                        .update_failed_reading_new_landed_blocks(Empty {})
                        .await,
                );
                "UpdateFailedReadingNewLandedBlocks"
            }
            BiddingServiceClientCommand::DestroySlotBidder(destroy_slot_bidder_params) => {
                Self::handle_error(client.destroy_slot_bidder(destroy_slot_bidder_params).await);
                "DestroySlotBidder"
            }
        };
        let duration = now.elapsed();
        if duration > Duration::from_millis(1) {
            warn!(
                duration = duration.as_micros(),
                command_name, "DX BiddingServiceClientAdapter::handle_command took too long",
            );
        }
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
                                let recv_time = offset_datetime_to_timestamp_us(OffsetDateTime::now_utc());
                                if let Some(Ok(callback)) = callback {
                                    if let Some(bid) = callback.bid {
                                        let traveling_us = recv_time - bid.protocol_send_time_us;
                                        if traveling_us > 1000 {
                                            warn!(
                                                traveling_us,
                                                "DX create_slot_bidder task bid traveling time too long",
                                            );
                                        }
                                        let payout_tx_value = Self::parse_option_u256(bid.payout_tx_value);
                                        let seen_competition_bid = Self::parse_option_u256(bid.seen_competition_bid);
                                        let trigger_creation_time = bid.trigger_creation_time_us.map(timestamp_us_to_offset_datetime);
                                        let payout_tx_value = if let Some(payout_tx_value) = payout_tx_value {
                                            payout_tx_value
                                        } else {
                                            warn!("payout_tx_value is None");
                                            continue;
                                        };
                                        let seal_command = SlotBidderSealBidCommand {
                                            block_id: BuiltBlockId(bid.block_id),
                                            payout_tx_value,
                                            seen_competition_bid,
                                            trigger_creation_time,
                                        };
                                        create_slot_bidder_data.block_seal_handle.seal_bid(seal_command);
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

    pub async fn must_win_block(&self, block: u64) {
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::MustWinBlock(
                MustWinBlockParams { block },
            ));
    }
}

impl BiddingService for BiddingServiceClientAdapter {
    fn create_slot_bidder(
        &self,
        slot_block_id: SlotBlockId,
        slot_timestamp: OffsetDateTime,
        block_seal_handle: Box<dyn BlockSealInterfaceForSlotBidder + Send + Sync>,
        cancel: CancellationToken,
    ) -> Arc<dyn SlotBidder> {
        // This default will be immediately changed by a callback.
        let session_id = self.new_session_id();
        let _ = self
            .commands_sender
            .send(BiddingServiceClientCommand::CreateSlotBidder(
                CreateSlotBidderCommandData {
                    params: CreateSlotBidderParams {
                        block: slot_block_id.block,
                        slot: slot_block_id.slot,
                        parent_hash: real2rpc_block_hash(slot_block_id.parent_block_hash),
                        session_id,
                        slot_timestamp: slot_timestamp.unix_timestamp(),
                    },
                    block_seal_handle,
                    cancel,
                },
            ));
        Arc::new(UnfinishedBlockBuildingSinkClient::new(
            session_id,
            self.block_sender.clone(),
            self.commands_sender.clone(),
        ))
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

    fn observe_relay_bids(&self, bid_with_stats: Vec<ScrapedRelayBlockBidWithStats>) {
        for bid in bid_with_stats {
            let _ = self.bids_sender.send(real2rpc_block_bid(bid));
        }
    }
}
