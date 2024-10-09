use std::collections::VecDeque;

use rbuilder::building::builders::block_building_helper::BlockBuildingHelper;

use super::traits::BlockId;

/// Maintains a map from BlockId -> BlockBuildingHelper.
/// Auto generates unique BlockIds.
/// To avoid (possible RPC) reference count handling if only keeps the last max_blocks_to_keep blocks.
/// We externally assume that we'll never need more.
/// Overflows after 2^64 blocks but it's impossible to get to that in a single slot.
pub struct BlockRegistry {
    last_generated_block_id: u64,
    /// Id of first block in blocks. The following blocks have sequential ids.
    first_block_id: u64,
    blocks: VecDeque<Box<dyn BlockBuildingHelper>>,
    max_blocks_to_keep: usize,
}

impl std::fmt::Debug for BlockRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockRegistry")
            .field("max_blocks_to_keep", &self.max_blocks_to_keep)
            .field("first_block_id", &self.first_block_id)
            .field("last_generated_block_id", &self.last_generated_block_id)
            .field("blocks_len", &self.blocks.len())
            .finish_non_exhaustive()
    }
}

impl BlockRegistry {
    pub fn new(max_blocks_to_keep: usize) -> Self {
        Self {
            max_blocks_to_keep,
            // Fist generated item will be 1 since we pre increment last_generated_block_id.
            first_block_id: 1,
            last_generated_block_id: 0,
            blocks: Default::default(),
        }
    }

    pub fn add_block(&mut self, block: Box<dyn BlockBuildingHelper>) -> BlockId {
        self.blocks.push_back(block);
        if self.blocks.len() > self.max_blocks_to_keep {
            self.blocks.pop_front();
            self.first_block_id += 1;
        }
        self.last_generated_block_id += 1;
        BlockId(self.last_generated_block_id)
    }

    pub fn _remove_older_than(&mut self, id: BlockId) {
        while self.first_block_id < id.0 && !self.blocks.is_empty() {
            self.blocks.pop_front();
            self.first_block_id += 1;
        }
    }

    pub fn get_block_clon(&self, id: BlockId) -> Option<Box<dyn BlockBuildingHelper>> {
        if id.0 < self.first_block_id || id.0 >= self.first_block_id + self.blocks.len() as u64 {
            return None;
        }
        Some(self.blocks[(id.0 - self.first_block_id) as usize].box_clone())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use rbuilder::building::builders::mock_block_building_helper::MockBlockBuildingHelper;

    use super::BlockRegistry;
    const MAX_BLOCKS: usize = 10;
    #[test]
    fn add_block() {
        let mut registry = BlockRegistry::new(MAX_BLOCKS);
        let initial_true_block_value = 0;
        let mut block_ids = Vec::new();
        for i in 0..=MAX_BLOCKS {
            let true_block_value = U256::from(i + initial_true_block_value);
            let can_add_payout_tx = i % 2 == 0;
            let block_id = registry.add_block(Box::new(MockBlockBuildingHelper::new(
                true_block_value,
                can_add_payout_tx,
            )));
            block_ids.push(block_id.clone());
            let block = registry.get_block_clon(block_id).unwrap();
            assert_eq!(block.true_block_value().unwrap(), true_block_value);
            assert_eq!(block.can_add_payout_tx(), can_add_payout_tx);
        }
        // it should remember the last MAX_BLOCKS
        for i in 1..=MAX_BLOCKS {
            let true_block_value = U256::from(i + initial_true_block_value);
            let can_add_payout_tx = i % 2 == 0;
            let block = registry.get_block_clon(block_ids[i].clone()).unwrap();
            assert_eq!(block.true_block_value().unwrap(), true_block_value);
            assert_eq!(block.can_add_payout_tx(), can_add_payout_tx);
        }
        // the oldest should not be stored.
        assert!(registry.get_block_clon(block_ids[0].clone()).is_none());
    }
}
