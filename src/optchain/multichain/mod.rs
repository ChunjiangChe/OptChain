use crate::{
    optchain::{
        blockchain::Blockchain,
        configuration::Configuration,
        block::{
            Info,
            versa_block::{VersaHash, VersaBlock},
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
        },
        transaction::Transaction,
    },
    types::{
        hash::H256,
    }
};
use std::{
    sync::{Arc, Mutex},
    collections::HashMap,
};


pub struct Multichain {
    pub config: Configuration,
    proposer_chain: Arc<Mutex<Blockchain>>,
    availability_chains: Vec<Arc<Mutex<Blockchain>>>,
}

impl Clone for Multichain {
    fn clone(&self) -> Self {
        let new_availability_chains: Vec<Arc<Mutex<Blockchain>>> = self.availability_chains
            .clone()
            .into_iter()
            .map(|x| Arc::clone(&x))
            .collect();
        Multichain {
            config: self.config.clone(),
            proposer_chain: Arc::clone(&self.proposer_chain),
            availability_chains: new_availability_chains,
        }
    }
}

impl Multichain {
    pub fn create(
        proposer_chain: &Arc<Mutex<Blockchain>>,
        availability_chains: Vec<&Arc<Mutex<Blockchain>>>, 
        config: &Configuration) -> Self 
    {

        let new_availability_chains: Vec<Arc<Mutex<Blockchain>>> = availability_chains
            .into_iter()
            .map(|x| Arc::clone(x))
            .collect();
        Multichain {
            proposer_chain: Arc::clone(proposer_chain),
            availability_chains: new_availability_chains,
            config: config.clone()
        }
    }

    pub fn insert_block_with_parent(
        &mut self,
        block: VersaBlock,
        parent: &VersaHash,
        shard_id: usize
    ) -> Result<bool, String> {
        match parent.clone() {
            VersaHash::PropHash(h) => {
                self.proposer_chain
                    .lock()
                    .unwrap()
                    .insert_block_with_parent(block, &h)
            }
            VersaHash::ExHash(h) => {
                self.availability_chains
                    .get(block.get_shard_id().unwrap())        
                    .unwrap()
                    .lock()
                    .unwrap()
                    .insert_block_with_parent(block, &h)
            }
            VersaHash::InHash(h) => {
                self.availability_chains
                    .get(shard_id)        
                    .unwrap()
                    .lock()
                    .unwrap()
                    .insert_block_with_parent(block, &h)
            }
        }
    }

    pub fn get_longest_proposer_chain_hash(&self) -> H256 {
        self.proposer_chain
            .lock()
            .unwrap()
            .tip()
    }

    pub fn all_blocks_in_longest_proposer_chain(&self) -> Vec<H256> {
        self.proposer_chain
            .lock()
            .unwrap()
            .all_blocks_in_longest_chain()

    }
    pub fn all_blocks_in_longest_availability_chain_by_shard(&self, shard_id: usize) -> Vec<H256> {
        self.availability_chains
            .get(shard_id)
            .unwrap()
            .lock()
            .unwrap()
            .all_blocks_in_longest_chain()
    }
    pub fn all_proposer_blocks_end_with_block(&self, hash: &H256) -> Option<Vec<H256>> {
        self.proposer_chain
            .lock()
            .unwrap()
            .all_blocks_end_with_block(hash)
    }
    pub fn get_proposer_block(&self, hash: &H256) -> Option<VersaBlock> {
        self.proposer_chain
            .lock()
            .unwrap()
            .get_block(hash)
    }
    pub fn get_tx_blk_in_longest_proposer_chain(
        &self, 
        blk_hash: &H256) -> Option<TransactionBlock> 
    {
        self.proposer_chain
            .lock()
            .unwrap()
            .get_tx_blk_in_longest_chain(blk_hash)
    }
    pub fn get_highest_prop_block(&self) -> H256 {
        self.proposer_chain
            .lock()
            .unwrap()
            .tip()
    }
    pub fn get_highest_avai_block(&self, shard_id: usize) -> H256 {
        self.availability_chains
            .get(shard_id)
            .unwrap()
            .lock()
            .unwrap()
            .tip()
    }

    pub fn get_all_highest_avai_blocks(&self) -> Vec<(H256, usize)> {
        //to be completed
        vec![]
        // self.availability_chains.iter()
        //                         .enumerate()
        //                         .map(|(index, value)| (value.clone(), index as usize))
        //                         .collect()
    }


    pub fn get_avai_tx_blocks(&self, num: usize) -> Result<Vec<TransactionBlock>, Vec<TransactionBlock>> {
        //to be completed
        Err(vec![])
    }
    pub fn get_block_by_shard(&self, hash: &H256, shard_id: usize) -> Option<VersaBlock> {
        //to be completed
        None
    }
    pub fn get_prop_block(&self, hash: &H256) -> Option<ProposerBlock> {
        //to be completed
        None
    }
    pub fn get_avai_block_by_shard(&self, hash: &H256, shard_id: usize) -> Option<AvailabilityBlock> {
        //to be completed
        None
    }
    pub fn verify_availability(&self, tx_blk: &TransactionBlock) -> Result<bool, String> {
        //to be completed
        Ok(true)
    }
    pub fn get_all_prop_refer_tx_blks(&self) -> Vec<TransactionBlock> {
        //to be completed
        vec![]
    }
    // pub fn log_to_file_with_shard(&self, shard_id: usize) {
    //     self.chains
    //         .get(shard_id)
    //         .unwrap()
    //         .lock()
    //         .unwrap()
    //         .log_to_file();
    // }
}
