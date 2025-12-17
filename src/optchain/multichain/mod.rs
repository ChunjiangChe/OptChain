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
            ordering_block::OrderingBlock,
        },
    },
    types::{
        hash::{H256, Hashable},
        database::Database,
    }
};
// use std::{
//     // sync::{Arc, Mutex},
//     collections::BTreeSet,
// };
use std::time::{SystemTime};

impl Hashable for (H256, u32) {
    fn hash(&self) -> H256 {
        H256::pow_hash(&self.0, self.1)
    }
}


pub struct Multichain {
    pub config: Configuration,
    proposer_chain: Blockchain,
    availability_chains: Vec<Blockchain>,
    ordering_chain: Blockchain,
    hash2prop_cmts: Database<Vec<TransactionBlock>>, // prop_hash -> prop_tx_block_set
    hash2avai_cmts: Database<Vec<TransactionBlock>>, // avai_hash -> avai_tx_block_set
    hash2confirmed_avai_blks: Database<Vec<(H256, u32)>>, // prop_hash -> confirmed_avai_hashes
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
    pub fn new(
        proposer_chain: Blockchain,
        availability_chains: Vec<Blockchain>, 
        ordering_chain: Blockchain,
        config: &Configuration) -> Self 
    {
        assert_eq!(proposer_chain.size(), 1);
        for i in 0..availability_chains.len() {
            assert_eq!(availability_chains.get(i).unwrap().size(), 1);
        }

        let mut hash2prop_cmts: Database<Vec<TransactionBlock>> = 
          Database::<Vec<TransactionBlock>>::new(format!("node(shard-{},index-{})/multichain/hash2prop_cmts/{:?}", config.shard_id, config.node_id, SystemTime::now()));
        let mut hash2avai_cmts: Database<Vec<TransactionBlock>> = 
          Database::<Vec<TransactionBlock>>::new(format!("node(shard-{},index-{})/multichain/hash2avai_cmts/{:?}", config.shard_id, config.node_id, SystemTime::now()));
        let mut hash2confirmed_avai_blks: Database<Vec<(H256, u32)>> = 
          Database::<Vec<(H256, u32)>>::new(format!("node(shard-{},index-{})/multichain/hash2confirmed_avai_blks/{:?}", config.shard_id, config.node_id, SystemTime::now()));

        if let VersaBlock::PropBlock(proposer_genesis_block) = proposer_chain.get_genesis_block() {
            let prop_tx_set = proposer_genesis_block.get_prop_tx_set();
            hash2prop_cmts.insert(proposer_genesis_block.hash(), prop_tx_set).unwrap();
        } else {
            panic!("Proposer genesis block doesnt exist");
        }

        if let VersaBlock::ExAvaiBlock(avai_genesis_block) = availability_chains
            .get(config.shard_id)
            .unwrap()
            .get_genesis_block() {
            let avai_tx_set = avai_genesis_block.get_avai_tx_set();
            hash2avai_cmts.insert(avai_genesis_block.hash(), avai_tx_set).unwrap();
        } else {
            panic!("Proposer genesis block doesnt exist");
        }

        if let VersaBlock::OrderBlock(ordering_genesis_block) = ordering_chain.get_genesis_block() {
            let confirmed_avai_set = ordering_genesis_block.get_confirmed_avai_set();
            hash2confirmed_avai_blks.insert(ordering_genesis_block.hash(), confirmed_avai_set).unwrap();
        } else {
            panic!("Ordering genesis block doesnt exist");
        }
        

        Multichain {
            proposer_chain,
            availability_chains,
            ordering_chain,
            hash2prop_cmts,
            hash2avai_cmts,
            hash2confirmed_avai_blks,
            config: config.clone(),
        }
    }

    pub fn insert_block_with_parent(
        &mut self,
        block: VersaBlock,
        parent: &VersaHash,
        shard_id: usize
    ) -> Result<bool, String> {
        let blk_hash = block.hash();
        match parent.clone() {
            VersaHash::PropHash(h) => {
                match self.proposer_chain
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => {
                        //update hash2prop_cmts
                        let old_tx_blocks = self.hash2prop_cmts.get(&h).unwrap();
                        let tx_blocks = block.get_tx_blocks();
                        //choose those transaction within the current shard
                        //into_iter() takes T, .filter takes x as a references (can use &x to unwrap &x to x)
                        let filtered_tx_blocks: Vec<TransactionBlock> = tx_blocks.into_iter().filter(|x| x.get_shard_id() == self.config.shard_id).collect();
                        //combine old_tx_blocks and tx_blocks.
                        let new_tx_blocks = old_tx_blocks.iter().chain(&filtered_tx_blocks).cloned().collect();
                        self.hash2prop_cmts.insert(blk_hash, new_tx_blocks).unwrap();
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
            VersaHash::ExHash(h) => {
                match self.availability_chains
                    .get_mut(block.get_shard_id().unwrap())        
                    .unwrap()
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => {
                        if shard_id == self.config.shard_id {
                            //update hash2avai_cmts
                            let old_tx_blocks = self.hash2avai_cmts.get(&h).unwrap();
                            let tx_blocks = block.get_tx_blocks();
                            //combine old_tx_blocks and tx_blocks.
                            let new_tx_blocks = old_tx_blocks.iter().chain(&tx_blocks).cloned().collect();
                            self.hash2avai_cmts.insert(blk_hash, new_tx_blocks).unwrap();
                        } 
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
            VersaHash::InHash(h) => {
                match self.availability_chains
                    .get_mut(shard_id)        
                    .unwrap()
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => {
                        if shard_id == self.config.shard_id {
                            //update hash2avai_cmts
                            let old_tx_blocks = self.hash2avai_cmts.get(&h).unwrap();
                            let tx_blocks = block.get_tx_blocks();
                            //combine old_tx_blocks and tx_blocks.
                            let new_tx_blocks = old_tx_blocks.iter().chain(&tx_blocks).cloned().collect();
                            self.hash2avai_cmts.insert(blk_hash, new_tx_blocks).unwrap();
                        }
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
            VersaHash::OrderHash(h) => {
                match self.ordering_chain
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => {
                        //update hash2confirmed_avai_blks
                        let old_confirmed_avai_blks = self.hash2confirmed_avai_blks.get(&h).unwrap();
                        let confirmed_avai_set = block.get_confirmed_avai_set().unwrap();
                        //combine old_confirmed_avai_blks and confirmed_avai_set.
                        let new_confirmed_avai_blks = old_confirmed_avai_blks.iter().chain(&confirmed_avai_set).cloned().collect();
                        self.hash2confirmed_avai_blks.insert(blk_hash, new_confirmed_avai_blks).unwrap();
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }


    pub fn all_blocks_in_longest_proposer_chain(&self) -> Vec<H256> {
        self.proposer_chain
            .all_blocks_in_longest_chain()

    }
    pub fn all_blocks_in_longest_ordering_chain(&self) -> Vec<H256> {
        self.ordering_chain
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
        (0..self.config.shard_num)
            .into_iter()
            .map(|i| (self.availability_chains.get(i).unwrap().tip(), i))
            .collect()
    }

    pub fn get_prop_cmts(&self, prop_hash: &H256) -> Vec<TransactionBlock> {
        self.hash2prop_cmts.get(prop_hash).unwrap()
    }

    pub fn get_unreferred_cmt(&self, prop_hash: &H256) -> Vec<TransactionBlock> {
        let prop_tx_set = self.hash2prop_cmts.get(prop_hash).unwrap();
        let latest_avai_tx_set = self.hash2avai_cmts.get(
            &(self.availability_chains.get(self.config.shard_id).unwrap().tip())
        ).unwrap();
        let unreferred_cmts: Vec<TransactionBlock> = prop_tx_set.into_iter().filter(|x| !latest_avai_tx_set.contains(x)).collect();
        unreferred_cmts
    }

    // get num available tx_blocks(cmt_root) in the proposer chain which are not included in the longest
    // availability chains already
    pub fn get_avai_tx_blocks(&self, num: usize) -> Result<Vec<TransactionBlock>, Vec<TransactionBlock>> {
        let blocks_in_longest_proposer_chain = self.proposer_chain
            .all_blocks_in_longest_chain();
        let confirmed_prop_hash = if blocks_in_longest_proposer_chain.len() > self.config.k {
            blocks_in_longest_proposer_chain
                .get(blocks_in_longest_proposer_chain.len() - 1 - self.config.k)
                .unwrap()
        } else {
            blocks_in_longest_proposer_chain
                .get(0)
                .unwrap()
        };
        let unreferred_cmts = self.get_unreferred_cmt(&confirmed_prop_hash);
        if unreferred_cmts.len() >= num {
            let res: Vec<TransactionBlock> = unreferred_cmts.iter().take(num).cloned().collect();
            Ok(res)
        } else {
            Err(unreferred_cmts)
        }   
    }
    // pub fn get_block_by_shard(&self, _hash: &H256, _shard_id: usize) -> Option<VersaBlock> {
    //     //to be completed
    //     None
    // }
    pub fn get_prop_block(&self, hash: &H256) -> Option<ProposerBlock> {
        match self.proposer_chain.get_block(hash) {
            Some(versa_block) => {
                if let VersaBlock::PropBlock(prop_block) = versa_block {
                    Some(prop_block)
                } else {
                    panic!("Non-proposer block exists in proposer chain");
                }
            }
            None => None,
        }
    }
    pub fn get_order_block(&self, hash: &H256) -> Option<OrderingBlock> {
        match self.ordering_chain.get_block(hash) {
            Some(versa_block) => {
                if let VersaBlock::OrderBlock(order_block) = versa_block {
                    Some(order_block)
                } else {
                    panic!("Non-ordering block exists in ordering chain");
                }
            }
            None => None,
        }
    }
    pub fn get_avai_block_by_shard(&self, hash: &H256, shard_id: usize) -> Option<AvailabilityBlock> {
        match self.availability_chains  
            .get(shard_id)
            .unwrap()
            .get_block(hash) {
            Some(versa_block) => {
                match versa_block {
                    VersaBlock::PropBlock(_) => panic!("Non-avaibility block exists in availability chains"),
                    VersaBlock::ExAvaiBlock(ex_avai_block) => Some(ex_avai_block),
                    VersaBlock::InAvaiBlock(in_avai_block) => Some(in_avai_block),
                    VersaBlock::OrderBlock(_) => panic!("Non-avaibility block exists in availability chains"),
                }
            }
            None => None,
        } 
    }

    pub fn get_prop_size(&self) -> usize {
        self.proposer_chain.size()
    }

    pub fn get_avai_size(&self, shard_id: usize) -> usize {
        self.availability_chains.get(shard_id).unwrap().size()
    }
    
    pub fn print_proposer_chain(&self) {
        let all_proposer_hashes = self.proposer_chain.all_blocks_in_longest_chain();
        for proposer_hash in all_proposer_hashes.iter() {
            let block = self.proposer_chain.get_block(proposer_hash).unwrap();
            if let VersaBlock::PropBlock(prop_block) = block {
                println!("{:?}\n", prop_block);
            } else {
                panic!("Should be a proposer block");
            }
        }
        println!("");
    }

    pub fn print_availability_chains(&self) {
        for i in 0..self.config.shard_num {
            let all_availability_hashes = self
                .availability_chains
                .get(i)
                .unwrap()
                .all_blocks_in_longest_chain();
            for availability_hash in all_availability_hashes.iter() {
                let block = self.availability_chains
                    .get(i)
                    .unwrap()
                    .get_block(availability_hash).unwrap();
                match block {
                    VersaBlock::PropBlock(_) => {
                        panic!("Should be a proposer block");
                    }
                    VersaBlock::ExAvaiBlock(ex_avai_block) => {
                        println!("Shard {} {:?}\n", i, ex_avai_block);
                    }
                    VersaBlock::InAvaiBlock(in_avai_block) => {
                        println!("Shard {} {:?}\n", i, in_avai_block);
                    }
                    VersaBlock::OrderBlock(_) => {
                        panic!("Should be an availability block");
                    }
                }
            }
            println!("");
        }
    }

    pub fn get_proposer_forking_rate(&self) -> f64 {
        self.proposer_chain.get_forking_rate()
    }

    pub fn get_ordering_forking_rate(&self) -> f64 {
        self.ordering_chain.get_forking_rate()
    }

    pub fn get_availability_forking_rate_by_shard(&self, shard_id: usize) -> f64 {
        self.availability_chains
            .get(shard_id)
            .unwrap()
            .get_forking_rate()
    }

    pub fn get_highest_order_block(&self) -> H256 {
        self.ordering_chain
            .tip()
    }

    pub fn get_confirmed_avai_set_by_order_hash(&self, order_hash: &H256) -> Result<Vec<(H256, u32)>, String> {
        if let None = self.hash2confirmed_avai_blks.get(order_hash) {
            return Err(format!("Ordering block {:?} doesnt exist", order_hash));
        } else {
            Ok(self.hash2confirmed_avai_blks.get(order_hash).unwrap())
        }
    }

    pub fn get_new_confirmed_avai_set(&self) -> Vec<(H256, u32)> {
        let order_parent = self.ordering_chain.tip();
        let old_confirm_avai_set = self.hash2confirmed_avai_blks.get(&order_parent).unwrap();
        
        let all_confirmed_avai_set: Vec<(H256, u32)> = self.availability_chains
            .iter()
            .enumerate()
            .map(|(shard_id, chain)| {
                let all_avai_blocks = chain.all_blocks_in_longest_chain();
                match all_avai_blocks.len() <= self.config.k {
                    true => vec![],
                    false => {
                        let confirmed_avai_blocks: Vec<(H256, u32)> = all_avai_blocks
                            .iter()
                            .take(all_avai_blocks.len() - self.config.k)
                            .cloned()
                            .map(|h| (h, shard_id as u32))
                            .collect();
                        confirmed_avai_blocks
                    }
                }
            })
            .flatten()
            .collect();

        let new_confirmed_avai_set: Vec<(H256, u32)> = all_confirmed_avai_set
            .into_iter()
            .filter(|x| !old_confirm_avai_set.contains(x))
            .collect();  
        new_confirmed_avai_set
    }

    // pub fn get_all_prop_refer_tx_blks(&self) -> Vec<TransactionBlock> {
    //     //to be completed
    //     vec![]
    // }
    // pub fn log_to_file_with_shard(&self, shard_id: usize) {
    //     self.chains
    //         .get(shard_id)
    //         .unwrap()
    //         .lock()
    //         .unwrap()
    //         .log_to_file();
    // }
}
