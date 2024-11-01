//! This module is responsible for syncing the best true value bid between the local state and redis.

use alloy_primitives::U256;

use rbuilder::utils::{
    reconnect::{run_loop_with_reconnect, RunCommand},
    u256decimal_serde_helper,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tracing::{error, trace};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BestTrueValue {
    pub timestamp_ms: u64,
    pub block_number: u64,
    pub slot_number: u64,
    /// Best true value of submitted block (has subtracted the payout tx cost)
    #[serde(with = "u256decimal_serde_helper")]
    pub best_true_value: U256,
    /// Best bid seen on the relay at the moment of choosing the bid for our block
    #[serde(with = "u256decimal_serde_helper")]
    pub best_relay_value: U256,
    pub builder: String,
    pub slot_end_timestamp: u64,
}

impl BestTrueValue {
    pub fn new(
        block_number: u64,
        slot_number: u64,
        best_true_value: U256,
        best_relay_value: U256,
        slot_end_timestamp: u64,
    ) -> Self {
        BestTrueValue {
            timestamp_ms: (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as u64,
            block_number,
            slot_number,
            best_true_value,
            best_relay_value,
            // @Todo real builder name
            builder: String::from("tmp"),
            slot_end_timestamp,
        }
    }

    /// Compares things related to bidding: block_number,slot_number,best_true_value and best_relay_value
    pub fn is_same_bid_info(&self, other: &Self) -> bool {
        self.block_number == other.block_number
            && self.slot_number == other.slot_number
            && self.best_true_value == other.best_true_value
            && self.best_relay_value == other.best_relay_value
    }
}

#[derive(Debug, Default, Clone)]
pub struct BestTrueValueCell {
    data: Arc<Mutex<BestTrueValue>>,
}

impl BestTrueValueCell {
    pub fn update_value_safe(&self, value: BestTrueValue) {
        let mut best_value = self.data.lock().unwrap();
        if value.slot_number < best_value.slot_number {
            // don't update value for the past slot
            return;
        }
        if value.slot_number == best_value.slot_number
            && value.block_number == best_value.block_number
            && value.best_true_value < best_value.best_true_value
        {
            // don't update value if its lower
            return;
        }
        *best_value = value;
    }

    pub fn read(&self) -> BestTrueValue {
        self.data.lock().unwrap().clone()
    }
}

/// BestTrueValueRedisSync keeps best true value for the current block synced with other builders
/// using redis.
#[derive(Debug, Clone)]
pub struct BestTrueValuePusher<BackendType> {
    /// Best value we got from our building algorithms.
    best_local_value: BestTrueValueCell,
    backend: BackendType,

    cancellation_token: CancellationToken,
}

const PUSH_INTERVAL: Duration = Duration::from_millis(50);
const MAX_IO_ERRORS: usize = 5;

/// Trait to connect and publish new tbv data (as a &str)
/// For simplification mixes a little the factory role and the publish role.
pub trait Backend {
    type Connection;
    type BackendError: std::error::Error;
    /// Creates a new connection to the sink of tbv info.
    fn connect(&self) -> Result<Self::Connection, Self::BackendError>;
    /// Call with the connection obtained by connect()
    fn publish(
        &self,
        connection: &mut Self::Connection,
        best_true_value: &BestTrueValue,
    ) -> Result<(), Self::BackendError>;
}

impl<BackendType: Backend> BestTrueValuePusher<BackendType> {
    pub fn new(
        best_local_value: BestTrueValueCell,
        backend: BackendType,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            best_local_value,
            backend,
            cancellation_token,
        }
    }

    /// Run the task that pushes the best true value bid to redis.
    /// The value is read from best_local_value and pushed to redis.
    pub fn run_push_task(self) {
        run_loop_with_reconnect(
            "push_best_bid",
            || -> Result<BackendType::Connection, BackendType::BackendError> {
                self.backend.connect()
            },
            |mut conn| -> RunCommand {
                let mut io_errors = 0;
                let mut last_pushed_value: Option<BestTrueValue> = None;
                loop {
                    if self.cancellation_token.is_cancelled() {
                        break;
                    }

                    if io_errors > MAX_IO_ERRORS {
                        return RunCommand::Reconnect;
                    }

                    sleep(PUSH_INTERVAL);
                    let best_local_value = self.best_local_value.read();
                    if last_pushed_value
                        .as_ref()
                        .map_or(true, |value| !value.is_same_bid_info(&best_local_value))
                    {
                        last_pushed_value = Some(best_local_value.clone());
                        match self.backend.publish(&mut conn, &best_local_value) {
                            Ok(()) => {
                                trace!(?best_local_value, "Pushed best local value");
                            }
                            Err(err) => {
                                error!(?err, "Failed to publish best true value bid");
                                io_errors += 1;
                            }
                        }
                    }
                }
                RunCommand::Finish
            },
            self.cancellation_token.clone(),
        )
    }
}
