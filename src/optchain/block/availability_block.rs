use crate::{
    types::{
        hash::{H256, Hashable},
        merkle::MerkleTree,
        random::Random,
    },
    optchain::{
        block::{
            Info,
            BlockHeader,
            transaction_block::TransactionBlock,
        },
    },
};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime};

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct AvailabilityBlock {
    header: BlockHeader,
    nonce: u32,
    hash: H256,
    avai_tx_set: MerkleTree<TransactionBlock>,
    //txs: MerkleTree<Transaction>
}

impl std::fmt::Debug for AvailabilityBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str_vec: Vec<String> = self.avai_tx_set.data
            .iter()
            .map(|x| format!("{:?}", x.hash()))
            .collect();
        let avai_tx_set_str = str_vec.join(" ");
        write!(f, "AvailabilityBlock(block_header: {:?}, nonce: {}, avai_tx_set: {})", self.header, self.nonce, avai_tx_set_str)
    }
}

impl Info for AvailabilityBlock {
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

impl Default for AvailabilityBlock{
    fn default() -> Self {
        let header = BlockHeader::default(); 
        AvailabilityBlock {
            nonce: 0 as u32,
            hash: H256::pow_hash(&header.hash(), 0),
            avai_tx_set: MerkleTree::<TransactionBlock>::new((vec![]).as_slice()),
            header,
        }
    }
}

impl Random for AvailabilityBlock{
    fn random() -> Self {
        let header = BlockHeader::random();
        AvailabilityBlock {
            nonce: 0 as u32,
            hash: H256::pow_hash(&header.hash(), 0),
            avai_tx_set: MerkleTree::<TransactionBlock>::new((vec![]).as_slice()),
            header,
        }
    }
}

impl Hashable for AvailabilityBlock {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl AvailabilityBlock {
    pub fn new(header: BlockHeader, nonce: u32, avai_tx_set: MerkleTree<TransactionBlock>) -> Self {
        AvailabilityBlock {
            hash: H256::pow_hash(&header.hash(), nonce),
            header,
            nonce,
            avai_tx_set,
        }
    }

    // pub fn get_mem_size(&self) -> usize {
    //     std::mem::size_of::<u32>() + self.header.get_mem_size() + H256::get_mem_size() * self.avai_tx_set.len()
    // }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn get_avai_tx_set(&self) -> Vec<TransactionBlock> {
        self.avai_tx_set.data.clone()
    }

    pub fn verify_hash(&self) -> bool {
        H256::pow_hash(&self.header.hash(), self.nonce) == self.hash
    }
}




