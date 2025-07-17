//! This module contains a simplified version of the bidding world that passes a BlockDescriptor instead of a full BlockBuildingHelper which is a
//! much heavier object.
//! BlockDescriptor contains all the information that is needed for most simple bidding services that don't really look inside the block to bid.
//! A wrapper between both worlds if provided here.

mod bid_maker_adapter;
pub mod bidding_service_adapter;
mod block_registry;
pub mod mock_bidding_service;
pub mod mock_slot_bidder;
mod slot_bidder_adapter;
pub mod traits;

pub type SlotBidderId = u64;
