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
pub struct ProposerBlock {
    header: BlockHeader,
    nonce: u32,
    hash: H256,
    prop_tx_set: MerkleTree<TransactionBlock>,
    //txs: MerkleTree<Transaction>
}

impl std::fmt::Debug for ProposerBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let str_vec: Vec<String> = self.prop_tx_set.data
            .iter()
            .map(|x| format!("{:?}", x.hash()))
            .collect();
        let prop_tx_set_str = str_vec.join(" ");
        write!(f, "ProposerBlock(block_header: {:?}, nonce: {}, prop_tx_set: {})", self.header, self.nonce, prop_tx_set_str)
    }
}

impl Default for ProposerBlock{
    fn default() -> Self {
        let header = BlockHeader::default(); 
        ProposerBlock {
            nonce: 1 as u32, //to avoid the same default block as the default availability block in shard 0
            prop_tx_set: MerkleTree::<TransactionBlock>::new((vec![]).as_slice()),
            hash: H256::pow_hash(&header.hash(), 1),
            header,
        }
    }
}

impl Random for ProposerBlock {
    fn random() -> Self {
        let header = BlockHeader::random(); 
        ProposerBlock {
            nonce: 0 as u32,
            prop_tx_set: MerkleTree::<TransactionBlock>::new((vec![]).as_slice()),
            hash: H256::pow_hash(&header.hash(), 0),
            header,
        }
    }
}

impl Hashable for ProposerBlock {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl Info for ProposerBlock {
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

impl ProposerBlock {
    pub fn new(header: BlockHeader, nonce: u32, prop_tx_set: MerkleTree<TransactionBlock>) -> Self {
        ProposerBlock {
            hash: H256::pow_hash(&header.hash(), nonce),
            header,
            nonce,
            prop_tx_set,
        }
    }

    // pub fn get_mem_size(&self) -> usize {
    //     std::mem::size_of::<u32>() + self.header.get_mem_size() + H256::get_mem_size() * self.prop_tx_set.data.len()
    // }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn get_prop_tx_set(&self) -> Vec<TransactionBlock> {
        self.prop_tx_set.data.clone()
    }

    pub fn verify_hash(&self) -> bool {
        H256::pow_hash(&self.header.hash(), self.nonce) == self.hash
    }
}



