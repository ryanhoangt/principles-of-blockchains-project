use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::block::{Block, Content, Header};
use crate::types::hash::{Hashable, H256};
use crate::types::merkle::MerkleTree;
use crate::types::transaction::SignedTransaction;

pub struct Blockchain {
    tip: H256,
    max_len: u128,
    hash_to_block: HashMap<H256, Block>, // in-memory storage
    hash_to_len: HashMap<H256, u128>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let genesis_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let genesis_data: Vec<SignedTransaction> = Vec::new();
        let genesis_header = Header {
            parent: [0u8; 32].into(),
            nonce: 0u32,
            difficulty: H256::from([0u8; 32]),
            timestamp: genesis_time,
            merkle_root: MerkleTree::new(&genesis_data).root(),
        };
        let genesis_block = Block {
            header: genesis_header,
            content: Content { data: genesis_data },
        };

        let tip = genesis_block.hash();
        let max_len = 1u128;
        let mut hash_to_block: HashMap<H256, Block> = HashMap::new();
        let mut hash_to_len: HashMap<H256, u128> = HashMap::new();
        hash_to_block.insert(tip, genesis_block);
        hash_to_len.insert(tip, max_len);

        Blockchain {
            tip,
            max_len,
            hash_to_block,
            hash_to_len,
        }
    }

    /// Insert a block into blockchain
    // Assumption: the block is already validated
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let parent_hash = block.get_parent();

        let mut parent_len = 1u128;
        if self.hash_to_block.contains_key(&parent_hash) {
            parent_len = *self.hash_to_len.get(&parent_hash).unwrap();
        }

        self.hash_to_block.insert(block_hash, block.clone());
        self.hash_to_len.insert(block_hash, parent_len + 1);
        if parent_len + 1 > self.max_len {
            self.tip = block_hash;
            self.max_len = parent_len + 1;
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut res: Vec<H256> = vec![];
        let mut cur_hash = self.tip;

        while cur_hash != H256::from([0u8; 32]) {
            let parent_hash = self.hash_to_block[&cur_hash].get_parent();
            res.push(cur_hash);
            cur_hash = parent_hash;
        }
        res.reverse();
        res
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }

    #[test]
    fn test_longest_chain_rule_and_get_history() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let c1_b2 = generate_random_block(&genesis_hash);
        let c1_b3 = generate_random_block(&c1_b2.hash());
        let c1_b4 = generate_random_block(&c1_b3.hash());
        let c2_b2 = generate_random_block(&genesis_hash);
        let c2_b3 = generate_random_block(&c2_b2.hash());

        blockchain.insert(&c1_b2);
        assert_eq!(blockchain.tip(), c1_b2.hash());
        blockchain.insert(&c2_b2);
        blockchain.insert(&c2_b3);
        assert_eq!(blockchain.tip(), c2_b3.hash());
        blockchain.insert(&c1_b3);
        blockchain.insert(&c1_b4);
        assert_eq!(blockchain.tip(), c1_b4.hash());
        assert_eq!(
            blockchain.all_blocks_in_longest_chain(),
            vec![genesis_hash, c1_b2.hash(), c1_b3.hash(), c1_b4.hash()]
        )
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
