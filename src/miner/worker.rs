use crate::network::message::Message;
use crate::types::block::Block;
use crate::types::hash::Hashable;
use crate::{blockchain::Blockchain, network::server::Handle as ServerHandle};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self
                .finished_block_chan
                .recv()
                .expect("Receive finished block error");

            let mut _blockchain = self.blockchain.lock().unwrap();
            _blockchain.insert(&_block);
            drop(_blockchain);

            self.server
                .broadcast(Message::NewBlockHashes(vec![_block.hash()])); // blocking operation
        }
    }
}
