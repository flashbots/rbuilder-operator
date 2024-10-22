use ctor::ctor;
use lazy_static::lazy_static;
use metrics_macros::register_metrics;
use prometheus::{IntCounter, IntCounterVec, IntGaugeVec, Opts};
use rbuilder::{telemetry::REGISTRY, utils::build_info::Version};

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
