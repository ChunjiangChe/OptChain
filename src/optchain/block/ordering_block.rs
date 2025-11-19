use crate::{
    types::{
        hash::{H256, Hashable},
        random::Random,
    },
    optchain::{
        block::{
            Info,
            BlockHeader,
        },
    },
};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime};

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct OrderingBlock {
    header: BlockHeader,
    nonce: u32,
    hash: H256,
    confirmed_avai_set: Vec<(H256, u32)>,
    //txs: MerkleTree<Transaction>
}

impl std::fmt::Debug for OrderingBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str_vec: Vec<String> = self.confirmed_avai_set
            .iter()
            .map(|x| format!("{:?}", x))
            .collect();
        let confirmed_avai_set_str = str_vec.join(" ");
        write!(f, "OrderingBlock(block_header: {:?}, nonce: {}, confirmed_avai_set: {})", self.header, self.nonce, confirmed_avai_set_str)
    }
}

impl Default for OrderingBlock{
    fn default() -> Self {
        let header = BlockHeader::default(); 
        OrderingBlock {
            nonce: 1 as u32, //to avoid the same default block as the default availability block in shard 0
            confirmed_avai_set: vec![],
            hash: H256::pow_hash(&header.hash(), 1),
            header,
        }
    }
}

impl Random for OrderingBlock {
    fn random() -> Self {
        let header = BlockHeader::random(); 
        OrderingBlock {
            nonce: 0 as u32,
            confirmed_avai_set: vec![],
            hash: H256::pow_hash(&header.hash(), 0),
            header,
        }
    }
}

impl Hashable for OrderingBlock {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl Info for OrderingBlock {
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
    fn get_order_parent(&self) -> H256 {
        self.header.get_order_parent()
    }
    fn get_prop_root(&self) -> H256 {
        self.header.get_prop_root()
    }
    fn get_avai_root(&self) -> H256 {
        self.header.get_avai_root()
    }
    fn get_order_root(&self) -> H256 {
        self.header.get_order_root()
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

impl OrderingBlock {
    pub fn new(header: BlockHeader, nonce: u32, confirmed_avai_set: Vec<(H256, u32)>) -> Self {
        OrderingBlock {
            hash: H256::pow_hash(&header.hash(), nonce),
            header,
            nonce,
            confirmed_avai_set,
        }
    }

    // pub fn get_mem_size(&self) -> usize {
    //     std::mem::size_of::<u32>() + self.header.get_mem_size() + H256::get_mem_size() * self.prop_tx_set.data.len()
    // }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn get_confirmed_avai_set(&self) -> Vec<(H256, u32)> {
        self.confirmed_avai_set.clone()
    }

    pub fn verify_hash(&self) -> bool {
        H256::pow_hash(&self.header.hash(), self.nonce) == self.hash
    }
}



