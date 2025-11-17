use crate::{
    types::{
        hash::{H256, Hashable},
        // merkle::MerkleTree,
        random::Random,
    },
    optchain::{
        block::{
            BlockHeader, 
            // BlockContent,
            Info,
        },
        // transaction::Transaction,
    }
};
use serde::{Serialize, Deserialize};
// use std::collections::HashMap;
use std::time::{SystemTime};
use rand::Rng;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct TransactionBlock {
    header: BlockHeader,
    nonce: u32,
    //txs: MerkleTree<Transaction>
}

impl Random for TransactionBlock {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let nonce: u32 = rng.gen();
        TransactionBlock {
            header: BlockHeader::random(),
            nonce,
        }
    }
}

impl Default for TransactionBlock {
    fn default() -> Self {
        TransactionBlock {
            header: BlockHeader::default(),
            nonce: 0 as u32,
        }
    }
}

impl Hashable for TransactionBlock {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Info for TransactionBlock {
    fn get_shard_id(&self) -> usize {
        self.header.get_shard_id()
    }
    fn get_prop_parent(&self) -> H256 {
        self.header.get_prop_parent()
    }
    fn get_inter_parent(&self) -> H256 {
        self.header.get_inter_parent()
    }
    fn get_global_parents(&self) -> Vec<(H256, usize)> {
        self.header.get_global_parents()
    }
    fn get_prop_root(&self) -> H256 {
        self.header.get_prop_root()
    }
    fn get_avai_root(&self) -> H256 {
        self.header.get_avai_root()
    }
    fn get_cmt_root(&self) -> H256 {
        self.header.get_cmt_root()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.header.get_timestamp()
    }
    fn get_info_hash(&self) -> Vec<H256> {
        self.header.get_info_hash()
    }
}

impl TransactionBlock {
    pub fn new(header: BlockHeader, nonce: u32) -> Self {
        // let nonce: u32 = u32::try_from(nonce_usize)
        //     .expect(&format!("Nonce {} does not fit in u32!", nonce_usize));
        TransactionBlock {
            header,
            nonce,
        }
    }
    pub fn get_mem_size(&self) -> usize {
        std::mem::size_of::<u32>() + self.header.get_mem_size()
    }
    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }
    pub fn get_cmt_root(&self) -> H256 {
        self.header.get_cmt_root()
    }
}




