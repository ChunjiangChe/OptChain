use crate::{
    optchain::{
        multichain::Multichain,
        transaction::{Transaction, UtxoInput, TxFlag},
        block::{
            Info, 
            Content, 
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            transaction_block::TransactionBlock,
            versa_block::VersaBlock,
        },
        configuration::Configuration,
        mempool::Mempool,
        symbolpool::{
            SymbolIndex,
            Symbol,
        },
    },
    types::{
        hash::{Hashable, H256},
        merkle::MerkleTree,
    },
};
use std::{
    sync::{Arc, Mutex},
    collections::HashMap,
};
use log::{info, debug};

pub struct Validator {
    multichain: Multichain,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
}

impl Clone for Validator {
    fn clone(&self) -> Self {
        Validator {
            multichain: self.multichain.clone(),
            mempool: Arc::clone(&self.mempool),
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
        multichain: &Multichain,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration
    ) -> Self {
        Validator {
            multichain: multichain.clone(),
            mempool: Arc::clone(mempool),
            config: config.clone(),
        }
    }

    pub fn validate_block(&self, block: &VersaBlock) -> Result<bool, String> {
        //check whether the PoW is valid
        let blk_hash = block.hash();
        
        //check the hash value is corrent
        if !block.verify_hash() {
            return Err(String::from("Incorrect hash"));
        }
        
        //For availability block, check whether the symbols are received.
        match block {
            VersaBlock::PropBlock(_) => return Ok(true),
            VersaBlock::ExAvaiBlock(avai_block) => {
                for tx_blk_hash in avai_block.get_avai_tx_set().iter() {
                    match self.multichain.verify_availability(tx_blk_hash) {
                        Ok(_) => {}
                        Err(_) => {
                            return Err(String::from("some symbols are not received"));
                        }
                    }
                }
            }
            VersaBlock::InAvaiBlock(avai_block) => {
                for tx_blk_hash in avai_block.get_avai_tx_set().iter() {
                    match self.multichain.verify_availability(tx_blk_hash) {
                        Ok(_) => {}
                        Err(_) => {
                            return Err(String::from("some symbols are not received"));
                        }
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


