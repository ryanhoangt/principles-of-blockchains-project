use rand::Rng;
use ring::signature::{self, Ed25519KeyPair, Signature};
use serde::{Deserialize, Serialize};

use super::address::Address;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender: Address,
    receiver: Address,
    value: i64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // Ignore hashing message according to ECSDA
    let serialized_tx = bincode::serialize(t).unwrap();
    return key.sign(&serialized_tx);
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    // Ignore hashing message according to ECSDA
    let serialized_tx = bincode::serialize(t).unwrap();
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    return match peer_public_key.verify(&serialized_tx, signature) {
        Ok(_) => true,
        _ => false,
    };
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    let mut rng = rand::thread_rng();
    let random_pub_key_1: [u8; 32] = [rng.gen(); 32];
    let random_pub_key_2: [u8; 32] = [rng.gen(); 32];
    let sender = Address::from_public_key_bytes(&random_pub_key_1);
    let receiver = Address::from_public_key_bytes(&random_pub_key_2);
    Transaction {
        sender,
        receiver,
        value: rng.gen(),
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
