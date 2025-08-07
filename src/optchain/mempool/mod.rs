use crate::{
    types::hash::{
        H256, Hashable,
    },
    optchain::{
        block::transaction_block::TransactionBlock,
        database::Database,
    },
};
use std::collections::{VecDeque};
// use log::{info, debug};
use std::time::{SystemTime};


pub struct Mempool {
    tx_blk_map: Database<TransactionBlock>, //the key is the hash of the tx block, while value is the
    //full block
    tx_blk_queue: VecDeque<H256>,
}


impl Mempool {
    pub fn new() -> Self {
        let now = SystemTime::now();
        let tx_blk_map: Database<TransactionBlock> = 
            Database::<TransactionBlock>::new(format!("{:?}/mempool/tx_blk_map", now));
        Mempool {
            tx_blk_map,
            tx_blk_queue: VecDeque::new(),
        }
    }

    pub fn get_size(&self) -> usize {
        self.tx_blk_map.len()
    }

    pub fn get_queue_size(&self) -> usize {
        self.tx_blk_queue.len()
    }
    
    pub fn get_tx_blocks(&self, num: usize) -> Result<Vec<TransactionBlock>, Vec<TransactionBlock>> {
        //to be completed
        Err(vec![])
    }

    pub fn insert_tx_blk(&mut self, tx_blk: TransactionBlock) -> bool {
        let hash = tx_blk.hash();
        if self.tx_blk_map.contains_key(&hash) {
            //block already exists.
            false
        } else {
            self.tx_blk_map.insert(hash.clone(), tx_blk.clone());
            self.tx_blk_queue.push_back(hash);
            true
        }
    }

    pub fn check(&self, hash: &H256) -> bool {
        self.tx_blk_map.contains_key(hash)
    }
        
    pub fn get_tx_blk(&self, hash: &H256) -> Option<TransactionBlock> {
        if self.check(hash) {
            Some(self.tx_blk_map.get(hash).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_all_tx_blks(&self) -> Vec<TransactionBlock> {

        self.tx_blk_map
            .iter()
            .map(|(_, val)| val.clone())
            .collect()

    }

    pub fn delete_txs(&mut self, tx_blk_hashs: Vec<H256>) -> bool {
        for hash in tx_blk_hashs.iter() {
            self.tx_blk_map.remove(&hash);
            self.tx_blk_queue.retain(|x| x != hash);
        }
        true
    }

    

    pub fn pop_one_tx_blk(&mut self) -> Option<TransactionBlock> {
        if self.tx_blk_queue.is_empty() {
            None
        } else {
            let hash = self.tx_blk_queue.pop_front().unwrap();
            let tx_blk = self.tx_blk_map.get(&hash).unwrap().clone();
            self.tx_blk_map.remove(&hash);
            Some(tx_blk)
        }
    }

    pub fn get_all_tx_blk_hash(&self) -> Vec<H256> {
        self.tx_blk_map
            .iter()
            .map(|(key, _)| key.clone())
            .collect()
    }
    
}

