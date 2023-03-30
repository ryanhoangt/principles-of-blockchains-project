use std::collections::{HashMap, VecDeque};

use ring::digest;

use super::hash::{Hashable, H256};

#[derive(Debug, Default)]
pub struct MerkleNode {
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    value: H256,
}

impl MerkleNode {
    pub fn new_from_val(val: H256) -> Self {
        MerkleNode {
            left: None,
            right: None,
            value: val,
        }
    }

    // create a new node with separate copies of children
    pub fn new_from_children(left: &MerkleNode, right: &MerkleNode) -> Self {
        let left_val_bytes = <[u8; 32]>::from(left.value);
        let right_val_bytes = <[u8; 32]>::from(right.value);
        let mut buffer: [u8; 64] = [0; 64];
        buffer[..32].copy_from_slice(&left_val_bytes);
        buffer[32..].copy_from_slice(&right_val_bytes);
        let cat_hash = digest::digest(&digest::SHA256, &buffer).into();

        MerkleNode {
            left: Some(Box::new(left.clone())),
            right: Some(Box::new(right.clone())),
            value: cat_hash,
        }
    }
}

impl Clone for MerkleNode {
    fn clone(&self) -> Self {
        MerkleNode {
            left: self.left.clone(),
            right: self.right.clone(),
            value: self.value.clone(),
        }
    }
}

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    // MerkleTree can be implemented using MerkleNode instances
    // with references back to their parents.
    // In that case, use Rc<RefCell<>> in place of Box<>
    root: MerkleNode,
    level_to_nodes: HashMap<usize, VecDeque<MerkleNode>>,
    height: usize,
    leaf_level_size: usize,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self
    where
        T: Hashable,
    {
        let mut level_to_nodes = HashMap::new();
        let mut cur_height: usize = 0;
        let leaf_level_size = data.len();

        if data.len() == 0 {
            return MerkleTree {
                root: MerkleNode {
                    left: None,
                    right: None,
                    value: Default::default(),
                },
                level_to_nodes,
                leaf_level_size,
                height: cur_height,
            };
        }

        let mut cur_level_nodes: VecDeque<MerkleNode> = VecDeque::new(); // queue of nodes on current level
        data.into_iter().for_each(|item| {
            let temp_node = MerkleNode::new_from_val(item.hash());
            cur_level_nodes.push_back(temp_node);
        }); // ownership of nodes

        // if cur_level_nodes.len() % 2 == 1 {
        //     let last_node = cur_level_nodes.pop_back().unwrap();
        //     cur_level_nodes.push_back(last_node);
        //     cur_level_nodes.push_back(last_node.clone());
        // }

        while cur_level_nodes.len() > 1 {
            let mut parent_nodes: VecDeque<MerkleNode> = VecDeque::new();
            for i in (0..cur_level_nodes.len()).step_by(2) {
                let left = cur_level_nodes.get(i).unwrap();
                let right = cur_level_nodes.get(i + 1).unwrap_or(left);
                let parent = MerkleNode::new_from_children(left, right);
                parent_nodes.push_back(parent);
            }
            level_to_nodes.insert(cur_height, cur_level_nodes);
            cur_height += 1;
            cur_level_nodes = parent_nodes;
        }
        level_to_nodes.insert(cur_height, cur_level_nodes.clone());

        MerkleTree {
            root: cur_level_nodes[0].clone(),
            level_to_nodes,
            leaf_level_size,
            height: cur_height,
        }
    }

    pub fn root(&self) -> H256 {
        self.root.value
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof: Vec<H256> = vec![];
        let mut cur_idx = index;
        let mut cur_height = 0;
        let mut cur_level_size = self.leaf_level_size;

        while cur_height < self.height {
            let cur_level_nodes = self.level_to_nodes.get(&cur_height).unwrap();

            if cur_idx % 2 == 1 {
                proof.push(cur_level_nodes.get(cur_idx - 1).unwrap().value);
            } else if cur_idx != cur_level_size - 1 {
                proof.push(cur_level_nodes.get(cur_idx + 1).unwrap().value);
            }

            cur_idx /= 2;
            cur_level_size = (cur_level_size + 1) / 2;
            cur_height += 1;
        }

        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut cur_hash = *datum;
    let mut cur_idx = index;
    let mut cur_level_size = leaf_size;
    let mut proof_vec = Vec::from(proof);
    proof_vec.reverse();

    // reconstruct root hash according to proof slice
    while cur_level_size > 1 {
        let mut ctx = digest::Context::new(&digest::SHA256);
        if cur_idx % 2 == 1 {
            let even_hash = proof_vec.pop().unwrap();
            ctx.update(even_hash.as_ref());
            ctx.update(cur_hash.as_ref());
        } else if cur_idx == cur_level_size - 1 {
            ctx.update(cur_hash.as_ref());
            ctx.update(cur_hash.as_ref()); // duplicate itself
        } else {
            let snd_hash = proof_vec.pop().unwrap();
            ctx.update(cur_hash.as_ref());
            ctx.update(snd_hash.as_ref());
        }
        cur_hash = ctx.finish().into();
        cur_idx /= 2;
        cur_level_size = (cur_level_size + 1) / 2;
    }

    cur_hash == *root
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::hash::H256;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(
            proof,
            vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(
            &merkle_tree.root(),
            &input_data[0].hash(),
            &proof,
            0,
            input_data.len()
        ));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
