use crate::types::hash::{Hashable, H256};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};

use super::transaction::SignedTransaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub content: Content,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128, // unix time timestamp in second
    pub merkle_root: H256,
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized_header = bincode::serialize(self).unwrap();
        digest(&SHA256, &serialized_header).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub data: Vec<SignedTransaction>,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    use rand::Rng;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::types::merkle::MerkleTree;

    let mut rng = rand::thread_rng();
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let data: Vec<SignedTransaction> = Vec::new();

    Block {
        header: Header {
            parent: *parent,
            nonce: rng.gen(),
            difficulty: H256::from([1u8; 32]),
            timestamp: time_ms,
            merkle_root: MerkleTree::new(&data).root(),
        },
        content: Content { data },
    }
}
