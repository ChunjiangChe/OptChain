use crate::{
    optchain::{
        multichain::Multichain,
        block::{
            // Content, 
            versa_block::VersaBlock,
        },
        configuration::Configuration,
        mempool::Mempool,
        symbolpool::SymbolPool,
        // symbolpool::{
        //     SymbolIndex,
        //     Symbol,
        // },
    },
    // types::{
    //     hash::{Hashable},
    //     // merkle::MerkleTree,
    // },
};
use std::{
    sync::{Arc, Mutex},
    // collections::HashMap,
};
// use log::{info, debug};

pub struct Validator {
    multichain: Arc<Mutex<Multichain>>,
    mempool: Arc<Mutex<Mempool>>,
    symbol_pool: Arc<Mutex<SymbolPool>>,
    config: Configuration,
}

impl Clone for Validator {
    fn clone(&self) -> Self {
        Validator {
            multichain: Arc::clone(&self.multichain),
            mempool: Arc::clone(&self.mempool),
            symbol_pool: Arc::clone(&self.symbol_pool),
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub enum ValidationSource {
    FromBlock,
    FromTransaction,
}

pub enum CrossUtxoStatus {
    Available,
    Confirmed,
}


impl Validator {
    pub fn new(
        multichain: &Arc<Mutex<Multichain>>,
        mempool: &Arc<Mutex<Mempool>>,
        symbol_pool: &Arc<Mutex<SymbolPool>>,
        config: &Configuration
    ) -> Self {
        Validator {
            multichain: Arc::clone(multichain),
            mempool: Arc::clone(mempool),
            symbol_pool: Arc::clone(symbol_pool),
            config: config.clone(),
        }
    }

    pub fn validate_block(&self, block: &VersaBlock) -> Result<bool, String> {
        //check whether the PoW is valid
        // let blk_hash = block.hash();
        
        //check the hash value is corrent
        if !block.verify_hash() {
            return Err(String::from("Incorrect hash"));
        }
        
        match block {
            VersaBlock::PropBlock(_) => return Ok(true),
            VersaBlock::ExAvaiBlock(avai_block) => {
                for tx_blk in avai_block.get_avai_tx_set().iter() {
                    if !self.symbol_pool
                        .lock()
                        .unwrap()
                        .verify_availability(&tx_blk.get_cmt_root()) {
                            return Err(String::from("some symbols are not received"));
                    }
                }
            }
            VersaBlock::InAvaiBlock(avai_block) => {
                for tx_blk in avai_block.get_avai_tx_set().iter() {
                    if !self.symbol_pool
                        .lock()
                        .unwrap()
                        .verify_availability(&tx_blk.get_cmt_root()) {
                            return Err(String::from("some symbols are not received"));
                    }
                }
            }
        }


        // match self.multichain.get_block(parent) {
        //     Some(_) => {}
        //     None => {
        //         return Err(String::from("validation: parent not found"));              
        //     }
        // }


        
        Ok(true)
    }

}


