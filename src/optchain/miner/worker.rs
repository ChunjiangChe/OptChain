use crossbeam::channel::Receiver;
use log::{info};
use crate::{
    optchain::{
        block::{
            Info,
            Content,
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
        network::{
            server::Handle as ServerHandle,
            message::Message,
        },
        multichain::Multichain,
        miner::MinerMessage,
        transaction::Transaction,
        configuration::Configuration,
        mempool::Mempool,
        symbolpool::{
            SymbolPool,
            SymbolIndex,
            Symbol,
        },
    }
};
use std::{
    thread, 
    sync::{Arc, Mutex},
    collections::HashMap,
};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<MinerMessage>,
    multichain: Multichain,
    mempool: Arc<Mutex<Mempool>>,
    symbolpool: Arc<Mutex<SymbolPool>>,
    config: Configuration,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<MinerMessage>,
        multichain: &Multichain,
        mempool: &Arc<Mutex<Mempool>>,
        symbolpool: &Arc<Mutex<SymbolPool>>,
        config: &Configuration,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            multichain: multichain.clone(),
            mempool: Arc::clone(mempool),
            symbolpool: Arc::clone(symbolpool),
            config: config.clone(),
        }
    }

    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&mut self) {
        loop {
            let message = self.finished_block_chan
                .recv()
                .expect("Receive finished block error");
             
            match message {
                MinerMessage::TxBlk((tx_block, content)) => {
                    let cmt_root = tx_block.get_cmt_root();
                    self.mempool.lock()
                                .unwrap()
                                .insert_tx_blk(tx_block.clone());
                    //request all symbols of it
                    let indexs: Vec<usize> = (0..self.config.block_size).collect();
                    self.symbolpool.lock()
                                   .unwrap()
                                   .request_symbols(&cmt_root, indexs.clone());
                    //insert all symbols to the symbolpool
                    for index in indexs {
                        let tx = content.get_txs_ref().get(index).unwrap();
                        let symbol_index = SymbolIndex::new(cmt_root.clone(), index);
                        let symbol = Symbol::new(
                            symbol_index, 
                            vec![tx.clone()], 
                            content.get_tx_merkle_proof(index),
                            &self.config,
                        );
                        self.symbolpool.lock()
                                       .unwrap()
                                       .insert_symbol(symbol)
                                       .unwrap();
                    }
                    //
                    self.server.broadcast(Message::TxBlocks(vec![tx_block]));
                }
                MinerMessage::VersaBlk(versa_block) => {
                    match versa_block.clone() {
                        VersaBlock::InAvaiBlock(avai_block) => {
                            let global_parents = avai_block.get_global_parents();
                            for (inter_parent, shard_id) in global_parents {
                                match self.multichain.insert_block_with_parent(
                                    versa_block.clone(),
                                    &VersaHash::InHash(inter_parent),
                                    shard_id
                                ) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        info!("inserting myself fail: {}", e);    
                                    }
                                }
                            }
                            self.server.broadcast(
                                Message::Blocks(vec![versa_block])
                            );
                        }
                        VersaBlock::ExAvaiBlock(avai_block) => {
                            //exclusive avaialbility block
                            let inter_parent = avai_block.get_inter_parent();
                            match self.multichain.insert_block_with_parent(
                                versa_block.clone(),
                                &VersaHash::ExHash(inter_parent),
                                self.config.shard_id
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("inserting myself fail: {}", e);
                                }
                            }
                            self.server.broadcast(
                                Message::Blocks(vec![versa_block])
                            );
                        }
                        VersaBlock::PropBlock(prop_block) => {
                            let prop_parent = prop_block.get_prop_parent();
                            match self.multichain.insert_block_with_parent(
                                versa_block.clone(),
                                &VersaHash::PropHash(prop_parent),
                                self.config.shard_id
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("inserting myself fail: {}", e);
                                }
                            }
                            self.server.broadcast(
                                Message::Blocks(vec![versa_block])
                            );
                        }
                    }
                }    
            }

        }
    }
}
