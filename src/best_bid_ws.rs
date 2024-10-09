//! Best bid websocket connector
//!
//! This module uses websocket connection to get best bid value from the relays.
//! The best bid for the current block-slot is stored in the `BestBidCell` and can be read at any time.
use alloy_primitives::{utils::format_ether, U256};
use rbuilder::{
    live_builder::block_output::bid_value_source::interfaces::{BidValueObs, BidValueSource},
    utils::reconnect::{run_loop_with_reconnect, RunCommand},
};
use serde::Deserialize;
use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, trace, warn};
use tungstenite::{
    client::{connect_with_config, IntoClientRequest},
    http::Request,
    Message,
};

use crate::metrics::inc_non_0_competition_bids;

const MAX_IO_ERRORS: usize = 5;

const WS_CONNECT_MAX_REDIRECTS: u8 = 3;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BestBidValue {
    pub block_number: u64,
    pub slot_number: u64,
    pub block_top_bid: U256,
}

#[derive(Debug)]
struct Subscription {
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
    connection_request: Request<()>,
    subscriptions: Mutex<Vec<Subscription>>,
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

    pub fn run_ws_stream(
        &self,
        // We must try_send on every non 0 bid or the process will be killed
        watch_dog_sender: flume::Sender<()>,
        cancellation_token: CancellationToken,
    ) {
        run_loop_with_reconnect(
            "ws_top_bid_connection",
            || {
                connect_with_config(
                    self.connection_request.clone(),
                    None,
                    WS_CONNECT_MAX_REDIRECTS,
                )
                .map(|(c, _)| c)
            },
            |mut conn| -> RunCommand {
                let mut io_error_count = 0;
                loop {
                    if cancellation_token.is_cancelled() {
                        break;
                    }
                    if io_error_count >= MAX_IO_ERRORS {
                        warn!("Too many read errors, reconnecting");
                        return RunCommand::Reconnect;
                    }

                    let res = conn.read();
                    let message = match res {
                        Ok(message) => message,
                        Err(err) => {
                            warn!(?err, "Error reading WS stream");
                            io_error_count += 1;
                            continue;
                        }
                    };
                    let data = match message {
                        Message::Text(msg) => msg.into_bytes(),
                        Message::Binary(msg) => msg,
                        Message::Ping(data) => {
                            trace!("Received ping");
                            match conn.write(Message::Pong(data)) {
                                Ok(()) => {}
                                Err(err) => {
                                    warn!(?err, "Error writing pong");
                                    io_error_count += 1;
                                }
                            }
                            continue;
                        }
                        Message::Pong(_) | Message::Frame(_) => {
                            continue;
                        }
                        Message::Close(_) => {
                            warn!("Connection closed, reconnecting");
                            return RunCommand::Reconnect;
                        }
                    };

                    let bid_value: BestBidValue = match serde_json::from_slice(&data) {
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

                    for sub in self.subscriptions.lock().unwrap().deref_mut() {
                        if sub.block_number == bid_value.block_number
                            && sub.slot_number == bid_value.slot_number
                        {
                            sub.obs.update_new_bid(bid_value.block_top_bid);
                        }
                    }
                }
                RunCommand::Finish
            },
            cancellation_token.clone(),
        );
    }
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
