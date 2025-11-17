use log::{info};
// use crossbeam::channel::{
//     unbounded, 
//     Receiver, 
//     Sender, 
//     TryRecvError
// };
use std::{
    time::{self}, 
    thread, 
    sync::{Arc, Mutex},
    // collections::HashMap,
};
use crate::{        
    optchain::{
        multichain::Multichain,
        configuration::Configuration,
        network::{
            server::Handle as ServerHandle,
            message::Message,
            // worker::{SampleIndex},
        },
        symbolpool::{
            SymbolIndex,
            // Symbol,
            SymbolPool,
        },
        block::{
            Info,
        }
    },
    // types::hash::{H256, Hashable},
};
// use rand::Rng;


pub struct Context {
    multichain: Arc<Mutex<Multichain>>,
    config: Configuration,
    server: ServerHandle,
    symbol_pool: Arc<Mutex<SymbolPool>>,
}


pub fn new(multichain: &Arc<Mutex<Multichain>>,
    server: &ServerHandle,
    config: &Configuration,
    symbol_pool: &Arc<Mutex<SymbolPool>>) -> Context 
{
    Context {
        multichain: Arc::clone(multichain),
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
            let highest_prop_block_hash = self.multichain
                .lock()
                .unwrap()
                .get_highest_prop_block();
            let prop_ref_tx_blks = self.multichain
                .lock()
                .unwrap()
                .get_unreferred_cmt(&highest_prop_block_hash);
            if !prop_ref_tx_blks.is_empty() {
                let mut req_symbol_indexs: Vec<SymbolIndex> = vec![];
                for tx_blk in prop_ref_tx_blks.iter() {
                    let shard_id = tx_blk.get_shard_id();
                    let cmt_root = tx_blk.get_cmt_root();
                    
                    match self.symbol_pool
                        .lock()
                        .unwrap()
                        .get_unreceived_symbols(&cmt_root) 
                    {
                        Ok(symbol_indexs) => {
                            req_symbol_indexs.extend(symbol_indexs);
                        }
                        Err(_) => {
                            let new_symbol_indexs = self.symbol_pool
                                .lock()
                                .unwrap()
                                .request_symbols_for_new_cmt(&cmt_root, shard_id == self.config.shard_id)
                                .unwrap();
                            req_symbol_indexs.extend(new_symbol_indexs);
                        }
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



