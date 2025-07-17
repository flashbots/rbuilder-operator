//! Conversion real data <-> rpc data
use crate::bidding_service_wrapper::LandedBlockInfo as RPCLandedBlockInfo;

use alloy_primitives::U256;
use rbuilder::live_builder::block_output::bidding::interfaces::LandedBlockInfo as RealLandedBlockInfo;
use time::OffsetDateTime;
use tonic::Status;

pub fn real2rpc_landed_block_info(l: &RealLandedBlockInfo) -> RPCLandedBlockInfo {
    RPCLandedBlockInfo {
        block_number: l.block_number,
        block_timestamp: l.block_timestamp.unix_timestamp(),
        builder_balance: l.builder_balance.as_limbs().to_vec(),
        beneficiary_is_builder: l.beneficiary_is_builder,
    }
}

#[allow(clippy::result_large_err)]
pub fn rpc2real_landed_block_info(l: &RPCLandedBlockInfo) -> Result<RealLandedBlockInfo, Status> {
    Ok(RealLandedBlockInfo {
        block_number: l.block_number,
        block_timestamp: OffsetDateTime::from_unix_timestamp(l.block_timestamp)
            .map_err(|_| Status::invalid_argument("block_timestamp"))?,
        builder_balance: U256::from_limbs_slice(&l.builder_balance),
        beneficiary_is_builder: l.beneficiary_is_builder,
    })
}
