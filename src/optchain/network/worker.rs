use crate::{
    types::{
        hash::{H256, Hashable},
    },
    optchain::{
        network::{
            message::Message,
            peer,
            server::Handle as ServerHandle,
        },
        block::{
            Info, 
            transaction_block::TransactionBlock,
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
        configuration::Configuration,
        validator::{Validator},
        mempool::Mempool,
        multichain::Multichain,
        symbolpool::{
            SymbolPool,
            SymbolIndex,
            Symbol,
        },
    }
};
use log::{debug, warn, error, info};
use std::{
    thread,
    sync::{Arc,Mutex},
    collections::{HashMap, VecDeque},
};

//#[cfg(any(test,test_utilities))]
//use super::peer::TestReceiver as PeerTestReceiver;
//#[cfg(any(test,test_utilities))]
//use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    multichain: Multichain,
    blk_buff: HashMap<VersaHash, Vec<VersaBlock>>,
    mempool: Arc<Mutex<Mempool>>,
    symbolpool: Arc<Mutex<SymbolPool>>,
    config: Configuration,
    validator: Validator,
}

pub type SampleIndex = (H256, u32, u32); //block_hash, tx_index, shard_id
pub type Sample = (u32, H256);

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        multichain: &Multichain,
        mempool: &Arc<Mutex<Mempool>>,
        symbolpool: &Arc<Mutex<SymbolPool>>,
        config: &Configuration,
    ) -> Self {
        let validator = Validator::new(multichain, mempool, config);
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            multichain: multichain.clone(),
            blk_buff: HashMap::new(),
            mempool: Arc::clone(mempool),
            symbolpool: Arc::clone(symbolpool),
            config: config.clone(),
            validator,
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let mut cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }


    fn worker_loop(&mut self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewTxBlockHash(tx_blk_hashs) => {
                    //debug!("New tx block hashs");
                    if let Some(response) = self
                        .handle_new_tx_blk_hash(tx_blk_hashs) {
                        peer.write(response);
                    } 
                }
                Message::GetTxBlocks(tx_blk_hashs) => {
                    //debug!("Get tx blocks");
                    if let Some(response) = self
                        .handle_get_tx_blks(tx_blk_hashs) {
                        peer.write(response);
                    } 
                }
                Message::TxBlocks(tx_blks) => {
                    //debug!("Comming tx blocks");
                    if let Some(response) = self.handle_tx_blocks(tx_blks) {
                        if let Message::NewTxBlockHash(new_tx_blk_hashs) = response {
                            self.server.broadcast(
                                Message::NewTxBlockHash(new_tx_blk_hashs)
                            )
                        } 
                    }
                }

                
                Message::NewBlockHash(hash_vec) => {
                    //debug!("New versa block hash");
                    if let Some(response) = self
                        .handle_new_block_hash(hash_vec) {
                        peer.write(response);
                    }
                }
                Message::GetBlocks(hash_vec) => {
                    //debug!("Get versa blocks");
                    if let Some(response) = self
                        .handle_get_blocks(hash_vec) {
                        peer.write(response);
                    }
                }
                Message::Blocks(blocks) => {
                    //debug!("Coming versa blocks");
                    let (response_1, response_2) = self
                        .handle_blocks(blocks); 
                    if let Some(new_blks) = response_1 {
                        self.server.broadcast(new_blks.clone());
                    }
                    

                    //handle missing blocks
                    if let Some(missing_blks) = response_2 {
                        peer.write(missing_blks);
                    }
                }
                
                Message::NewSymbols(symbol_indexs) => {
                    //debug!("New Samples");
                    if let Some(response) = self
                        .handle_new_symbols(symbol_indexs) {
                        peer.write(response);
                    }
                }
                Message::GetSymbols(symbol_indexs) => {
                    //debug!("Get Samples");
                    if let Some(response) = self
                        .handle_get_symbols(symbol_indexs) {
                        peer.write(response);
                    }
                }
                Message::Symbols(samples) => {
                    //debug!("Coming Samples");
                    let response_1 = self
                        .handle_symbols(samples);
                    if let Some(res_1) = response_1 {
                        self.server.broadcast(res_1);
                    }
                }
                Message::NewMissBlockHash((miss_blk_vec, shard_id)) => {
                    for blk in miss_blk_vec {
                        match self.multichain.get_block_by_shard(
                            &blk,
                            shard_id as usize
                        ) {
                            Some(versa_block) => {
                                peer.write(Message::Blocks(vec![versa_block]));
                            }
                            None => {}
                        }
                    }
                }
                // _ => unimplemented!()
            }
        }
    }
   
    //handle transaction message
    fn handle_new_tx_blk_hash(
        &self, 
        tx_blk_hashes: Vec<H256>) -> Option<Message> 
    {
        let mut unreceived_tx_blks: Vec<H256> = Vec::new();
        for tx_blk_hash in tx_blk_hashes.iter() {
            if self.mempool.lock().unwrap().check(tx_blk_hash) {
                continue;
            }
            if let Some(_) = self.multichain
                .get_tx_blk_in_longest_proposer_chain(tx_blk_hash) {
                continue;
            }
            unreceived_tx_blks.push(tx_blk_hash.clone());
        }
        if !unreceived_tx_blks.is_empty() {
            Some(Message::GetTxBlocks(unreceived_tx_blks))
        } else {
            None
        }
    }
    fn handle_get_tx_blks(
        &self, 
        tx_blk_hashes: Vec<H256>) -> Option<Message> 
    {
        let mut res_tx_blks: Vec<TransactionBlock> = Vec::new();
        for tx_blk_hash in tx_blk_hashes.iter() {
            //find tx in mempool
            if let Some(blk) = self.mempool.lock().unwrap().get_tx_blk(tx_blk_hash) {
                res_tx_blks.push(blk);
                continue;
            }
            //find tx in blockchain
            if let Some(blk) = self.multichain.get_tx_blk_in_longest_proposer_chain(tx_blk_hash) {
                res_tx_blks.push(blk);
            }
        }
        if !res_tx_blks.is_empty() {
            Some(Message::TxBlocks(res_tx_blks))
        } else {
            None
        }
    }
    fn handle_tx_blocks(
        &self, 
        tx_blks: Vec<TransactionBlock>) -> Option<Message> 
    {
        let mut new_tx_blk_hashes: Vec<H256> = Vec::new();
        for blk in tx_blks.iter() {
            //find tx in mempool
            let hash = blk.hash();
            if let Some(_) = self.mempool.lock().unwrap().get_tx_blk(&hash) {
                continue;
            }
            //2.find tx in the longest proposer chain
            if let Some(_) = self.multichain.get_tx_blk_in_longest_proposer_chain(&hash){
                continue;
            }
            // match self.validator.validate_tx(tx, None, None, ValidationSource::FromTransaction) {
            //     Ok(_) => {}
            //     Err(_) => {
            //         continue;
            //     }
            // }
            new_tx_blk_hashes.push(hash);
            self.mempool.lock().unwrap().insert_tx_blk(blk.clone());
        }
        if !new_tx_blk_hashes.is_empty() {
            Some(Message::NewTxBlockHash(new_tx_blk_hashes))
        } else {
            None
        }
    }
    fn handle_new_block_hash(
        &self, 
        block_hash_vec: Vec<VersaHash>) -> Option<Message> 
    {
        if block_hash_vec.is_empty() {
            return None;
        }

        let mut unreceived_blks: Vec<VersaHash> = vec![];

        for versa_hash in block_hash_vec {
            match versa_hash.clone() {
                VersaHash::PropHash(prop_hash) => {
                    match self.multichain.get_prop_block(
                        &prop_hash) {
                        Some(_) => {}
                        None => unreceived_blks.push(
                            versa_hash
                        ),
                    }
                }
                VersaHash::ExHash(ex_hash) => {
                    let mut is_found = false;
                    //not sure the shard id of the exclusive block based on its hash
                    for id in 0..self.config.shard_num {
                        match self.multichain.get_avai_block_by_shard(
                            &ex_hash,
                            id
                        ){
                            Some(_) => {
                                is_found = true;
                                break;
                            }
                            None => {}
                        }
                    }
                    if !is_found {
                        unreceived_blks.push(
                            versa_hash
                        );
                    }
                }
                VersaHash::InHash(in_hash) => {
                    let mut is_found = false;
                    //not sure the shard id of the exclusive block based on its hash
                    for id in 0..self.config.shard_num {
                        match self.multichain.get_avai_block_by_shard(
                            &in_hash,
                            id
                        ){
                            Some(_) => {
                                is_found = true;
                                break;
                            }
                            None => {}
                        }
                    }
                    if !is_found {
                        unreceived_blks.push(
                            versa_hash
                        );
                    }
                }
            }
        }
            
        

        if !unreceived_blks.is_empty() {
            Some(Message::GetBlocks(unreceived_blks))
        } else {
            None
        }
    }

    fn handle_get_blocks(&self, hash_vec: Vec<VersaHash>) 
        -> Option<Message>
    {
        if hash_vec.is_empty() {
            return None;
        }
        

        let mut res_blks: Vec<VersaBlock> = vec![];

        for versa_hash in hash_vec {
            match versa_hash {
                VersaHash::PropHash(prop_hash) => {
                    match self.multichain.get_prop_block(
                            &prop_hash
                    ){
                        Some(block) => res_blks.push(VersaBlock::PropBlock(block)),
                        None => {}
                    }
                }
                VersaHash::ExHash(ex_hash) => {
                    for id in 0..self.config.shard_num {
                        match self.multichain.get_avai_block_by_shard(
                            &ex_hash, 
                            id
                        ){
                            Some(block) => {
                                res_blks.push(VersaBlock::ExAvaiBlock(block));
                                break;
                            }
                            None => {}
                        }
                    }
                }
                VersaHash::InHash(in_hash) => {
                    for id in 0..self.config.shard_num {
                        match self.multichain.get_avai_block_by_shard(
                            &in_hash, 
                            id
                        ){
                            Some(block) => {
                                res_blks.push(VersaBlock::InAvaiBlock(block));
                                break;
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        
        

        if !res_blks.is_empty() {
            Some(Message::Blocks(res_blks))
        } else {
            None
        }
    }

    fn handle_blocks(&mut self, blocks: Vec<VersaBlock>) 
        -> (Option<Message>, Option<Message>) 
    //new_block_hash, missing block
    {
        if blocks.is_empty() {
            return (None, None);
        }

        //key: hash of the block, value: shard_id
        let mut new_hashs: Vec<VersaHash> = vec![];
        // let mut missing_parents: HashMap<usize, Vec<H256>> = HashMap::new();
        let mut missing_parents: Vec<VersaHash> = vec![];
        // return tx
        for block in blocks {
            //verification
            // let shard_id = block.get_shard_id();
            
            //check whether the parent exits
            let parents: Vec<(VersaHash, usize)> = match block.clone() {
                VersaBlock::PropBlock(prop_block) => {
                    vec![(VersaHash::PropHash(prop_block.get_prop_parent()), 0)]
                }
                VersaBlock::ExAvaiBlock(ex_block) => {
                    vec![(VersaHash::ExHash(ex_block.get_inter_parent()), block.get_shard_id().unwrap())]
                }
                VersaBlock::InAvaiBlock(in_block) => {
                    in_block.get_global_parents()   
                            .into_iter()
                            .map(|(key, item)| (VersaHash::InHash(key), item))
                            .collect()
                }
            };
            
            for item in parents {
                let parent_hash = item.0;
                let inserted_shard_id = item.1;
                // //this is important
                // //the inclusive block can not be inserted in his own shard
                // if let VersaBlock::InBlock(_) = block {
                //     if inserted_shard_id == self.config.shard_id &&
                //         block.get_shard_id() == self.config.shard_id {
                //         continue;
                //     }
                // }
                
                //check whether the parent exits
                let mut parent_not_exisit = false;
                match parent_hash.clone() {
                    VersaHash::PropHash(prop_hash) => {
                        match self.multichain.get_prop_block(&prop_hash) {
                            Some(_) => {}
                            None => {
                                parent_not_exisit = true;
                            }
                        }
                    }
                    VersaHash::ExHash(ex_hash) => {
                        match self.multichain.get_avai_block_by_shard(&ex_hash, inserted_shard_id) {
                            Some(_) => {}
                            None => {
                                parent_not_exisit = true;
                            }
                        }
                    }
                    VersaHash::InHash(in_hash) => {
                        match self.multichain.get_avai_block_by_shard(&in_hash, inserted_shard_id) {
                            Some(_) => {}
                            None => {
                                parent_not_exisit = true;
                            }
                        }
                    }
                }

                //put the block in buff
                if parent_not_exisit {
                    match self.blk_buff.get(&parent_hash) {
                        Some(old_blks) => {
                            if !old_blks.contains(&block) {
                                let mut new_blks = old_blks.clone();
                                new_blks.push(block.clone());
                                self.blk_buff.insert(parent_hash.clone(), new_blks);
                            }
                        }
                        None => {
                            self.blk_buff.insert(parent_hash.clone(), vec![block.clone()]);
                        }
                    }
                    
                    info!("block insertion failure in shard {}: parent {:?} not fould", self.config.shard_id, parent_hash);
                    if !missing_parents.contains(&parent_hash) {
                        missing_parents.push(parent_hash.clone());
                    }
                    continue;
                }                

                match self.validator.validate_block(&block) {
                    Ok(_) => {}
                    Err(_) => {
                        info!("block insertion failure: the verification fails");
                        continue;
                    }
                }

                let mut inserted_blks: VecDeque<VersaBlock> = VecDeque::new();
                inserted_blks.push_back(block.clone());
                let mut removed_buff: Vec<VersaHash> = vec![];
                while !inserted_blks.is_empty() {
                    let inserted_blk = inserted_blks.pop_front().unwrap();
                    match self.multichain.insert_block_with_parent(
                        inserted_blk.clone(),
                        &parent_hash,
                        inserted_shard_id
                    ) {
                        Ok(_) => {
                            let new_hash = match inserted_blk.clone() {
                                VersaBlock::PropBlock(_) 
                                    => VersaHash::PropHash(inserted_blk.hash()),
                                VersaBlock::ExAvaiBlock(_)
                                    => VersaHash::ExHash(inserted_blk.hash()),
                                VersaBlock::InAvaiBlock(_)
                                    => VersaHash::InHash(inserted_blk.hash()),
                            };
                            new_hashs.push(new_hash.clone());
                            // info!("successfully inserting block: {:?}", new_hash);
                            

                            //if there are some blocks in the buff whose parent is the new block,
                            //continue to insert it
                            match self.blk_buff.get(&new_hash) {
                                Some(child_blks) => {
                                    for child_blk in child_blks {
                                        inserted_blks.push_back(child_blk.clone());
                                    }
                                    removed_buff.push(new_hash);
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            info!("Reject block {:?} in shard {}: insertion fails: {}", inserted_blk.hash(), self.config.shard_id, e);
                            break;
                        }
                    }
                }
                for item2 in removed_buff {
                    self.blk_buff.remove(&item2);
                }
            }
        }


        let res_new_hashes = match new_hashs.is_empty() {
            true => None,
            false => Some(Message::NewBlockHash(new_hashs)),
        };

        let res_missing_blks = match missing_parents.is_empty() {
            true => None,
            false => Some(Message::GetBlocks(missing_parents)),
        };
        

        (res_new_hashes, res_missing_blks)
    }



    


    fn handle_new_symbols(&self, symbol_indexs: Vec<SymbolIndex>) -> Option<Message> {
        let mut unreceived_symbols: Vec<SymbolIndex> = vec![];

        for index in symbol_indexs.iter() {
            //check if the symbol is a requested one 
            if self.symbolpool.lock()
                              .unwrap()
                              .check_if_requested(index) 
            {
                unreceived_symbols.push(index.clone());
            }   
        }
        if !unreceived_symbols.is_empty() {
            Some(Message::GetSymbols(unreceived_symbols))
        } else {
            None
        }
    }

    fn handle_get_symbols(&self, symbol_indexs: Vec<SymbolIndex>) -> Option<Message> {
        let mut res_symbols: Vec<Symbol> = vec![];
        for index in symbol_indexs.iter() {
            match self.symbolpool.lock()
                                 .unwrap()
                                 .get_symbol(index) 
            {
                Ok(symbol) => {
                    res_symbols.push(symbol);
                }
                Err(_) => {}
            }
        }

        if !res_symbols.is_empty() {
            Some(Message::Symbols(res_symbols))
        } else {
            None
        }
    }

    fn handle_symbols(&mut self, symbols: Vec<Symbol>) 
        -> Option<Message> //new_sample_hash
    {
        let mut new_symbols: Vec<SymbolIndex> = vec![];

        for symbol in symbols {
            let symbol_index = symbol.get_index();
            if self.symbolpool.lock()
                              .unwrap()
                              .check_if_requested(&symbol_index) 
            {
                self.symbolpool.lock()
                               .unwrap()
                               .insert_symbol(symbol)
                               .unwrap();
                new_symbols.push(symbol_index);
            }
        }
        
        match new_symbols.is_empty() {
            false => Some(Message::NewSymbols(new_symbols)),
            true => None,
        }
        
    }
}

