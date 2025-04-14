#![allow(unexpected_cfgs)]

use ctor::ctor;
use lazy_static::lazy_static;
use metrics_macros::register_metrics;
use prometheus::{HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGaugeVec, Opts};
use rbuilder::{
    telemetry::{linear_buckets_range, REGISTRY},
    utils::build_info::Version,
};
use time::Duration;

register_metrics! {
    pub static BLOCK_API_ERRORS: IntCounterVec = IntCounterVec::new(
        Opts::new("block_api_errors", "counter of the block processor errors"),
        &[]
    )
    .unwrap();

    pub static NON_0_COMPETITION_BIDS: IntCounter = IntCounter::new(
        "non_0_competition_bids",
        "Counter of non 0 bids seen on SCP stream"
    )
    .unwrap();

    pub static BIDDING_SERVICE_VERSION: IntGaugeVec = IntGaugeVec::new(
        Opts::new("bidding_service_version", "Version of the bidding service"),
        &["git", "git_ref", "build_time_utc"]
    )
    .unwrap();

    pub static TRIGGER_TO_BID_ROUND_TRIP_TIME: HistogramVec = HistogramVec::new(
        HistogramOpts::new("trigger_to_bid_round_trip_time_us", "Time (in microseconds) it takes from a trigger (new block or competition bid) to get a new bid to make")
            .buckets(linear_buckets_range(50.0, 2000.0, 100)),
        &["builder_name"]
    )
    .unwrap();

}

pub fn add_trigger_to_bid_round_trip_time(duration: Duration) {
    TRIGGER_TO_BID_ROUND_TRIP_TIME
        .with_label_values(&[])
        .observe(duration.as_seconds_f64() * 1_000_000.0);
}

pub fn inc_blocks_api_errors() {
    BLOCK_API_ERRORS.with_label_values(&[]).inc()
}

pub fn inc_non_0_competition_bids() {
    NON_0_COMPETITION_BIDS.inc();
}

pub(super) fn set_bidding_service_version(version: Version) {
    BIDDING_SERVICE_VERSION
        .with_label_values(&[
            &version.git_commit,
            &version.git_ref,
            &version.build_time_utc,
        ])
        .set(1);
}
