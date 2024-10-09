use ctor::ctor;
use lazy_static::lazy_static;
use metrics_macros::register_metrics;
use prometheus::{IntCounter, IntCounterVec, Opts};
use rbuilder::telemetry::REGISTRY;

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

}

pub fn inc_blocks_api_errors() {
    BLOCK_API_ERRORS.with_label_values(&[]).inc()
}

pub fn inc_non_0_competition_bids() {
    NON_0_COMPETITION_BIDS.inc();
}
