use crate::{
    optchain::{
        block::{
            Info,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            transaction_block::TransactionBlock,
        },
    },
    types::hash::{H256, Hashable},
};
use std::{
    time::SystemTime,
};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum VersaBlock {   
    PropBlock(ProposerBlock),
    ExAvaiBlock(AvailabilityBlock),
    InAvaiBlock(AvailabilityBlock),
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum VersaHash {
    PropHash(H256),
    ExHash(H256),
    InHash(H256),
}

impl Default for VersaBlock {
    fn default() -> Self {
        VersaBlock::PropBlock(ProposerBlock::default())
    }
}

impl Hashable for VersaBlock {
    fn hash(&self) -> H256 {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.hash(),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.hash(),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.hash(),
        }
    }
}

impl std::fmt::Debug for VersaBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.fmt(f),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.fmt(f),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.fmt(f),
        }
    }
}


impl VersaBlock {

    pub fn verify_hash(&self) -> bool {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.verify_hash(),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.verify_hash(),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.verify_hash(),
        }
    }

    pub fn get_shard_id(&self) -> Option<usize> {
        match self {
            VersaBlock::PropBlock(_) => None,
            VersaBlock::ExAvaiBlock(avai_block) => Some(avai_block.get_shard_id()),
            VersaBlock::InAvaiBlock(avai_block) => Some(avai_block.get_shard_id()),
        }
    }

    pub fn get_prop_parent(&self) -> Option<H256> {
        match self {
            VersaBlock::PropBlock(prop_block) => Some(prop_block.get_prop_parent()),
            VersaBlock::ExAvaiBlock(_) => None,
            VersaBlock::InAvaiBlock(_) => None,
        }
    }

    pub fn get_inter_parent(&self) -> Option<H256> {
        match self {
            VersaBlock::PropBlock(_) => None,
            VersaBlock::ExAvaiBlock(avai_block) => Some(avai_block.get_inter_parent()),
            VersaBlock::InAvaiBlock(avai_block) => Some(avai_block.get_inter_parent()),
        }
    }

    pub fn get_global_parents(&self) -> Option<Vec<(H256, usize)>> {
        match self {
            VersaBlock::PropBlock(_) => None,
            VersaBlock::ExAvaiBlock(_) => None,
            VersaBlock::InAvaiBlock(avai_block) => Some(avai_block.get_global_parents()),
        }
    }

    pub fn get_prop_root(&self) -> Option<H256> {
        match self {
            VersaBlock::PropBlock(prop_block) => Some(prop_block.get_prop_root()),
            VersaBlock::ExAvaiBlock(_) => None,
            VersaBlock::InAvaiBlock(_) => None,
        }
    }

    pub fn get_avai_root(&self) -> Option<H256> {
        match self {
            VersaBlock::PropBlock(_) => None,
            VersaBlock::ExAvaiBlock(avai_block) => Some(avai_block.get_avai_root()),
            VersaBlock::InAvaiBlock(avai_block) => Some(avai_block.get_avai_root()),
        }
    }

    pub fn get_cmt_root(&self) -> Option<H256> {
        match self {
            VersaBlock::PropBlock(_) => None,
            VersaBlock::ExAvaiBlock(_) => None,
            VersaBlock::InAvaiBlock(_) => None,
        }
    }

    pub fn get_timestamp(&self) -> SystemTime {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.get_timestamp(),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.get_timestamp(),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.get_timestamp(),
        }
    }

    pub fn get_info_hash(&self) -> Vec<H256> {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.get_info_hash(),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.get_info_hash(),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.get_info_hash(),
        }
    }

    pub fn get_tx_blocks(&self) -> Vec<TransactionBlock> {
        match self {
            VersaBlock::PropBlock(prop_block) => prop_block.get_prop_tx_set(),
            VersaBlock::ExAvaiBlock(avai_block) => avai_block.get_avai_tx_set(),
            VersaBlock::InAvaiBlock(avai_block) => avai_block.get_avai_tx_set(),
        }
    }
}
