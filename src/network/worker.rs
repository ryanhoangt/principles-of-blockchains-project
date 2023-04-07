use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};

use log::{debug, error, warn};

use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(any(test, test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test, test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hash_vec) => {
                    let _blockchain = self.blockchain.lock().unwrap();

                    let missed_hash_vec: Vec<H256> = hash_vec
                        .into_iter()
                        .filter(|hash| !_blockchain.hash_to_block.contains_key(&hash))
                        .collect();

                    if !missed_hash_vec.is_empty() {
                        peer.write(Message::GetBlocks(missed_hash_vec));
                    }
                }
                Message::GetBlocks(hash_vec) => {
                    let _blockchain = self.blockchain.lock().unwrap();

                    let block_vec: Vec<Block> = hash_vec
                        .into_iter()
                        .filter(|hash| _blockchain.hash_to_block.contains_key(&hash))
                        .map(|hash| _blockchain.hash_to_block[&hash].clone()) // send a deep copy
                        .collect();

                    if !block_vec.is_empty() {
                        peer.write(Message::Blocks(block_vec));
                    }
                }
                Message::Blocks(blocks_vec) => {
                    let mut _blockchain = self.blockchain.lock().unwrap();
                    let mut new_block_vec = vec![];

                    for block_item in blocks_vec {
                        // TODO: validate block
                        // TODO: handle block's parent's missing: put to buffer, get blocks from peers

                        let item_hash = block_item.hash();
                        if !_blockchain.hash_to_block.contains_key(&item_hash) {
                            _blockchain.insert(&block_item);
                            new_block_vec.push(item_hash);
                        }
                    }

                    if !new_block_vec.is_empty() {
                        self.server
                            .broadcast(Message::NewBlockHashes(new_block_vec));
                    }
                }
                Message::NewTransactionHashes(hash_vec) => {
                    // TODO:
                }
                Message::GetTransactions(hash_vec) => {
                    // TODO:
                }
                Message::Transactions(signed_tx_vec) => {
                    // TODO:
                }
            }
        }
    }
}

#[cfg(any(test, test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>,
}
#[cfg(any(test, test_utilities))]
impl TestMsgSender {
    fn new() -> (
        TestMsgSender,
        smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    ) {
        let (s, r) = smol::channel::unbounded();
        (TestMsgSender { s }, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test, test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let _blockchain = Blockchain::new();
    let _blockchain = Arc::new(Mutex::new(_blockchain));
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let worker = Worker::new(1, msg_chan, &server, &_blockchain);
    worker.start();
    let all_hash_vec = _blockchain.lock().unwrap().all_blocks_in_longest_chain();
    (test_msg_sender, server_receiver, all_hash_vec)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;
    use ntest::timeout;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver =
            test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
