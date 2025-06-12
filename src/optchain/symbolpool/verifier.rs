use log::{info, debug};
use crossbeam::channel::{
    unbounded, 
    Receiver, 
    Sender, 
    TryRecvError
};
use std::{
    time::{self, SystemTime}, 
    thread, 
    sync::{Arc, Mutex},
    collections::HashMap,
};
use crate::{        
    optchain::{
        multichain::Multichain,
        configuration::Configuration,
        network::{
            server::Handle as ServerHandle,
            message::Message,
            worker::{SampleIndex},
        },
        symbolpool::{
            SymbolIndex,
            Symbol,
            SymbolPool,
        },
        block::{
            Info,
            Content,
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            versa_block::VersaBlock,
        }
    },
    types::hash::{H256, Hashable},
};
use rand::Rng;


pub struct Context {
    multichain: Multichain,
    config: Configuration,
    server: ServerHandle,
    symbol_pool: Arc<Mutex<SymbolPool>>,
}


pub fn new(multichain: &Multichain,
    server: &ServerHandle,
    config: &Configuration,
    symbol_pool: &Arc<Mutex<SymbolPool>>) -> Context 
{
    Context {
        multichain: multichain.clone(),
        server: server.clone(),
        config: config.clone(),
        symbol_pool: Arc::clone(symbol_pool,)
    }
}



impl Context {
    //need to polish here
    pub fn start(mut self) {
       thread::Builder::new()
            .name("Symbol-Verifier".to_string())
            .spawn(move || {
                self.monitor_sample();
            })
            .unwrap();
        info!("Symbol monitor started");
    }
    fn monitor_sample(&mut self) {
        loop {
            //check if there are any unverified blocks, if yes, request the samples
            let prop_ref_tx_blks = self.multichain.get_all_prop_refer_tx_blks();
            if !prop_ref_tx_blks.is_empty() {
                let mut req_symbol_indexs: Vec<SymbolIndex> = vec![];
                for tx_blk in prop_ref_tx_blks.iter() {
                    let shard_id = tx_blk.get_shard_id();
                    let cmt_root = tx_blk.get_cmt_root();
                    
                    match self.symbol_pool
                        .lock()
                        .unwrap()
                        .get_unreceived_symbols(&cmt_root, shard_id == self.config.shard_id) 
                    {
                        Some(indexs) => {
                            for index in indexs {
                                req_symbol_indexs.push(
                                    SymbolIndex::new(cmt_root.clone(), index)
                                );
                            }
                        }
                        None => {}
                    }
                    
                }
                if !req_symbol_indexs.is_empty() {
                    self.server.broadcast(Message::GetSymbols(req_symbol_indexs));
                }
            } else {
                //info!("no unverified blocks");
            }
            let interval = time::Duration::from_micros(30000000);
            thread::sleep(interval);
        }
    }
}



