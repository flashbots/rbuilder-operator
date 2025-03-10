//! Best bid websocket connector
//!
//! This module uses websocket connection to get best bid value from the relays.
//! The best bid for the current block-slot is stored in the `BestBidCell` and can be read at any time.
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use alloy_primitives::utils::format_ether;
use rbuilder::live_builder::block_output::bid_value_source::interfaces::{BidValueObs, BidValueSource};
use serde::Deserialize;
use crate::reconnect::{run_async_loop_with_reconnect, RunCommand};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{connect_async_with_config, tungstenite::protocol::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace, warn};
use alloy_primitives::U256;

use crate::metrics::inc_non_0_competition_bids;

type Connection = WebSocketStream<MaybeTlsStream<TcpStream>>;

const MAX_IO_ERRORS: usize = 5;

// time that we wait for a new value before reconnecting
const READ_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BestBidValue {
    pub block_number: u64,
    pub slot_number: u64,
    pub block_top_bid: U256,
}

#[derive(Debug)]
pub struct Subscription {
    pub block_number: u64,
    pub slot_number: u64,
    pub obs: Arc<dyn BidValueObs>,
}


/// Struct that connects to a websocket feed with best bids from the competition.
/// Allows to subscribe so listen for changes on a particular slot.
/// Usage:
/// - call sub = subscribe
///     - monitor the value as long as needed:
///         - await wait_for_change. This will wake when a change happens, no need for polling.
///         - Ask top_bid
/// - call unsubscribe(sub)
#[derive(Debug)]
pub struct BestBidWSConnector {
    connection_request: Request,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
}

impl BestBidWSConnector {
    pub fn new(url: &str, basic_auth: &str) -> eyre::Result<Self> {
        let mut connection_request = url.into_client_request()?;
        connection_request
            .headers_mut()
            .insert("Authorization", format!("Basic {}", basic_auth).parse()?);

        Ok(Self {
            connection_request,
            subscriptions: Default::default(),
        })
    }

    pub async fn run_ws_stream(
        &self,
        // We must try_send on every non 0 bid or the process will be killed
        watch_dog_sender: flume::Sender<()>,
        cancellation_token: CancellationToken,
    ) {
	run_async_loop_with_reconnect(
            "ws_top_bid_connection",
            || connect(self.connection_request.clone()),
            |conn| run_command(conn, cancellation_token.clone(), watch_dog_sender.clone(), self.subscriptions.clone()),
            None,
            cancellation_token.clone(),
    )
	    .await;
    }
}

async fn connect<R>(request: R) -> Result<Connection, Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(
        request, None, true, // TODO: naggle, decide
    )
    .await
    .map(|(c, _)| c)
}

async fn run_command(
    mut conn: Connection,
    cancellation_token: CancellationToken,
    watch_dog_sender: flume::Sender<()>,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
) -> RunCommand {
    let mut io_error_count = 0;
    loop {
        if cancellation_token.is_cancelled() {
            break;
        }
        if io_error_count >= MAX_IO_ERRORS {
            warn!("Too many read errors, reconnecting");
            return RunCommand::Reconnect;
        }

	let next_message = tokio::time::timeout(READ_TIMEOUT, conn.next());
        let res = match next_message.await {
	    Ok(res) => res,
	    Err(err) => {
		warn!(?err, "Timeout error");
                return RunCommand::Reconnect;
	    }
	};
        let message = match res {
            Some(Ok(message)) => message,
            Some(Err(err)) => {
                warn!(?err, "Error reading WS stream");
                io_error_count += 1;
                continue;
            },
	    None => {
		warn!("Connection read stream is closed, reconnecting");
                return RunCommand::Reconnect;
	    },
        };
        let data = match &message {
            Message::Text(msg) => msg.as_bytes(),
            Message::Binary(msg) => msg.as_ref(),
            Message::Ping(_) => {
                error!(ws_message = "ping", "Received unexpected message");
                continue;
            }
            Message::Pong(_) => {
                error!(ws_message = "pong", "Received unexpected message");
                continue;
            }
            Message::Frame(_) => {
                error!(ws_message = "frame", "Received unexpected message");
                continue;
            }
            Message::Close(_) => {
                warn!("Connection closed, reconnecting");
                return RunCommand::Reconnect;
            }
        };

        let bid_value: BestBidValue = match serde_json::from_slice(data) {
            Ok(value) => value,
            Err(err) => {
                error!(?err, "Failed to parse best bid value");
                continue;
            }
        };

        if !bid_value.block_top_bid.is_zero() {
            inc_non_0_competition_bids();
            let _ = watch_dog_sender.try_send(());
        }

        trace!(
            block = bid_value.block_number,
            slot = bid_value.slot_number,
            value = format_ether(bid_value.block_top_bid),
            "Updated best bid value"
        );

        for sub in subscriptions.lock().unwrap().deref_mut() {
            if sub.block_number == bid_value.block_number
                && sub.slot_number == bid_value.slot_number
            {
                sub.obs.update_new_bid(bid_value.block_top_bid);
            }
        }
    }
    RunCommand::Finish
}


impl BidValueSource for BestBidWSConnector {
    fn subscribe(&self, block_number: u64, slot_number: u64, obs: Arc<dyn BidValueObs>) {
        self.subscriptions.lock().unwrap().push(Subscription {
            block_number,
            slot_number,
            obs,
        });
    }

    fn unsubscribe(&self, obs: Arc<dyn BidValueObs>) {
        self.subscriptions
            .lock()
            .unwrap()
            .retain(|s| !Arc::ptr_eq(&s.obs, &obs));
    }
}


