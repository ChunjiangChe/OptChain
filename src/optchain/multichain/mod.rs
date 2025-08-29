use crate::{
    optchain::{
        blockchain::Blockchain,
        configuration::Configuration,
        block::{
            // Info,
            versa_block::{VersaHash, VersaBlock},
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
        },
    },
    types::{
        hash::H256,
    }
};
use std::{
    // sync::{Arc, Mutex},
    collections::BTreeSet,
};


pub struct Multichain {
    pub config: Configuration,
    proposer_chain: Blockchain,
    availability_chains: Vec<Blockchain>,
    new_tx_blocks: BTreeSet<TransactionBlock>, // those transaction blocks in proposer but not in availability blocks
}

// impl Clone for Multichain {
//     fn clone(&self) -> Self {
//         let new_availability_chains: Vec<Blockchain> = self.availability_chains
//             .iter()
//             .map(|x| x.clone())
//             .collect();
//         Multichain {
//             config: self.config.clone(),
//             proposer_chain: self.proposer_chain.clone(),
//             availability_chains: new_availability_chains,
//             new_tx_blocks: BTreeSet::new(),
//         }
//     }
// }

impl Multichain {
    pub fn create(
        proposer_chain: Blockchain,
        availability_chains: Vec<Blockchain>, 
        config: &Configuration) -> Self 
    {
        let shard_id = config.shard_id;
        let prop_tx_set = proposer_chain.get_all_tx_blk_in_longest_chain_by_shard(shard_id);
        let mut avai_tx_set = availability_chains.get(shard_id).get_all_tx_blk_in_longest_chain();

        //sort avai_tx_set so we can binary search it effiently
        avai_tx_set.sort_unstable();

        //Keep only elements in A that are not in B
        let new_tx_set: Vec<TransactionBlock> = prop_tx_set
            .into_iter()
            .filter(|x| avai_tx_set.binary_search(x).is_err())
            .collect();

        Multichain {
            proposer_chain,
            availability_chains,
            config: config.clone(),
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
                match self.proposer_chain
                    .insert_block_with_parent(block, &h) {
                        Ok(true) => {

                        }
                        Err(e) => Err(e),
                    }
            }
            VersaHash::ExHash(h) => {
                self.availability_chains
                    .get(block.get_shard_id().unwrap())        
                    .unwrap()
                    .insert_block_with_parent(block, &h)
            }
            VersaHash::InHash(h) => {
                self.availability_chains
                    .get(shard_id)        
                    .unwrap()
                    .insert_block_with_parent(block, &h)
            }
        }
    }

    pub fn get_longest_proposer_chain_hash(&self) -> H256 {
        self.proposer_chain
            .tip()
    }

    pub fn all_blocks_in_longest_proposer_chain(&self) -> Vec<H256> {
        self.proposer_chain
            .all_blocks_in_longest_chain()

    }
    pub fn all_blocks_in_longest_availability_chain_by_shard(&self, shard_id: usize) -> Vec<H256> {
        self.availability_chains
            .get(shard_id)
            .unwrap()
            .all_blocks_in_longest_chain()
    }
    pub fn all_proposer_blocks_end_with_block(&self, hash: &H256) -> Option<Vec<H256>> {
        self.proposer_chain
            .all_blocks_end_with_block(hash)
    }
    pub fn get_proposer_block(&self, hash: &H256) -> Option<VersaBlock> {
        self.proposer_chain
            .get_block(hash)
    }
    pub fn get_tx_blk_in_longest_proposer_chain(
        &self, 
        blk_hash: &H256) -> Option<TransactionBlock> 
    {
        self.proposer_chain
            .get_tx_blk_in_longest_chain(blk_hash)
    }
    pub fn get_highest_prop_block(&self) -> H256 {
        self.proposer_chain
            .tip()
    }
    pub fn get_highest_avai_block(&self, shard_id: usize) -> H256 {
        self.availability_chains
            .get(shard_id)
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

    // get num available tx_blocks(cmt_root) in the proposer chain which are not included in the longest
    // availability chains already
    pub fn get_avai_tx_blocks(&self, _num: usize) -> Result<Vec<TransactionBlock>, Vec<TransactionBlock>> {
        //to be completed

        Err(vec![])
    }
    pub fn get_block_by_shard(&self, _hash: &H256, _shard_id: usize) -> Option<VersaBlock> {
        //to be completed
        None
    }
    pub fn get_prop_block(&self, _hash: &H256) -> Option<ProposerBlock> {
        //to be completed
        None
    }
    pub fn get_avai_block_by_shard(&self, _hash: &H256, _shard_id: usize) -> Option<AvailabilityBlock> {
        //to be completed
        None
    }
    pub fn verify_availability(&self, _tx_blk: &TransactionBlock) -> Result<bool, String> {
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
