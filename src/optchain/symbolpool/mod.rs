pub mod verifier;

use std::{
    time::{SystemTime},
    // thread,
    // sync::{Arc,Mutex},
    collections::HashMap,
};
use serde::{Serialize, Deserialize};
use rand::{
    thread_rng,
    seq::IteratorRandom,
    // Rng,
};
use log::info;
use crate::{        
    optchain::{
        transaction::Transaction,
        configuration::Configuration,
    },
    types::{
        hash::{H256, Hashable},
        merkle::MerkleTree,
        database::Database,
    },
};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SymbolIndex {
    index: u32, 
    root: H256,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Symbol {
    index: SymbolIndex,
    data: Vec<Transaction>,
    merkle_proof: Vec<H256>,
    num_symbol_per_block: u32,
}

impl Hashable for SymbolIndex {
    fn hash(&self) -> H256 {
        H256::pow_hash(&self.root, self.index)
    }
}

impl Hashable for Symbol {
    fn hash(&self) -> H256 {
        self.index.hash()
    }
}

impl SymbolIndex {
    pub fn new(root: H256, index: usize) -> Self {
        Self {
            index: index as u32,
            root,
        }
    }

    pub fn get_index(&self) -> usize {
        self.index as usize
    }

    pub fn get_root(&self) -> H256 {
        self.root.clone()
    }
}



impl Symbol {
    pub fn new(
        index: SymbolIndex,
        data: Vec<Transaction>,
        merkle_proof: Vec<H256>,
        config: &Configuration,
    ) -> Self {
        Self {
            index,
            data,
            merkle_proof,
            num_symbol_per_block: config.num_symbol_per_block as u32,
        }
    }
    pub fn get_index(&self) -> SymbolIndex {
        self.index.clone()
    }

    //for simplification, here data only contains one transaction
    pub fn verify(&self) -> bool {
        let hashs = self.data
                        .iter()
                        .map(|x| x.hash())
                        .collect();
        let hash = H256::multi_hash(&hashs);
        let hash_hash = hash.hash();
        MerkleTree::<H256>::verify(
            &self.index.get_root(), 
            &hash_hash, 
            &self.merkle_proof, 
            self.index.get_index(), 
            self.num_symbol_per_block as usize,
        )
    }
}

pub struct SymbolPool {
    //symbol_hash -> (idnex, symbol)s
    //received symbols
    hash2symbol: Database<Symbol>, 
    //cmt_root -> requested index
    //requested symbols
    root2index: HashMap<H256, Vec<usize>>, 
    config: Configuration,
}

impl SymbolPool {
    pub fn new (config: &Configuration) -> Self {
        let now = SystemTime::now();
        //let mut hash2blk: HashMap<H256, VersaBlock> = HashMap::new();
        let hash2symbol: Database::<Symbol> = 
          Database::<Symbol>::new(format!("node(shard-{},index-{})/symbolpool/hash2symbol/{:?}", config.shard_id, config.node_id, now));
        Self {
            hash2symbol,
            root2index: HashMap::new(),
            config: config.clone(),
        }
    }
    pub fn check_if_requested(&self, symbol_index: &SymbolIndex) -> bool {
        if let Some(value) = self.root2index.get(&symbol_index.get_root()) {
            if value.contains(&symbol_index.get_index()) {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
    pub fn get_symbol(&self, symbol_index: &SymbolIndex) -> Result<Symbol, String> {
        match self.hash2symbol.get(&symbol_index.hash()) {
            Some(value) => Ok(value),
            None => Err(String::from("Symbol root does not exisit")),
        }
    }

    //``ex_or_in`` is used to distinguish exclusive and inclusive transaction block 
    pub fn request_symbols_for_new_cmt(&mut self, hash: &H256, ex_or_in: bool) -> Result<Vec<SymbolIndex>, String> {
        match self.root2index.get(hash) {
            Some(_) => {
                Err(format!("cmt {:?} has already been requested", hash))
            }
            None => {
                let mut rng = thread_rng();
                let req_num = match ex_or_in {
                    true => self.config.ex_req_num,
                    false => self.config.in_req_num,
                };
                let request_indexs = (0..self.config.num_symbol_per_block).choose_multiple(&mut rng, req_num);
                self.root2index.insert(hash.clone(), request_indexs.clone());
                info!("cmt {:?} requested (indexs: {:?})", hash, request_indexs);
                
                let request_symbol_index: Vec<SymbolIndex> = request_indexs
                    .into_iter()
                    .map(|i| SymbolIndex::new(hash.clone(), i))
                    .collect();
                Ok(request_symbol_index)
            }
        }
    }


    pub fn get_unreceived_symbols(&self, hash: &H256) -> Result<Vec<SymbolIndex>, String> {
        match self.root2index.get(hash) {
            Some(indexs) => {
                let mut unreceived_indexs: Vec<SymbolIndex> = vec![];
                for idx in indexs.iter() {
                    let symbol_index = SymbolIndex::new(hash.clone(), *idx);
                    info!("symbol_index: {:?}", symbol_index);
                    match self.hash2symbol.get(&symbol_index.hash()) {
                        Some(_) => {}
                        None => {
                            info!("symbol doesnt exist");
                            unreceived_indexs.push(symbol_index);
                        },
                    }
                }
                return Ok(unreceived_indexs);
            }
            None => {
                Err(format!("cmt {:?} hasnt been requested yet", hash))
            }
        }
    }

    pub fn request_symbols(&mut self, root: &H256, indexs: Vec<usize>) -> Result<bool, String> {
        match self.root2index.get(root) {
            Some(_) => {
                Err(String::from("The cmt_root is already requested"))
            }
            None => {
                self.root2index.insert(root.clone(), indexs);
                Ok(true)
            }
        }
    }

    pub fn insert_symbol(&mut self, sym: Symbol) ->Result<bool, String> {
        let cmt_root = sym.get_index().get_root();
        match self.root2index.get(&cmt_root) {
            Some(indexs) => {
                let sym_index = sym.get_index().get_index();
                if !sym.verify() {
                    return Err(String::from("Incorrect symbol"));
                }
                if !indexs.contains(&sym_index) {
                    return Err(String::from("Not a requested symbol"));
                }
                let symbol_hash = sym.hash();
                match self.hash2symbol.get(&symbol_hash) {
                    Some(_) => {
                        //Symbol already existis
                        return Ok(false);
                    }
                    None => {
                        self.hash2symbol.insert(symbol_hash, sym).unwrap();
                        info!("symbol (cmt: {:?}, index: {:?}) is inserted", cmt_root, sym_index);
                        return Ok(true);
                    }
                }
            }
            None => {
                return Err(String::from("Not an valid cmt_root"));
            }
        }
    }

    // pub fn verify_availability(&self, cmt_root: &H256) -> Result<bool, String> {
    //     match self.root2index.get(cmt_root) {
    //         Some(indexs) => {
    //             for index in indexs {
    //                 let symbol_index = SymbolIndex::new(cmt_root.clone(), *index);
    //                 if let None = self.hash2symbol.get(&symbol_index.hash()) {
    //                     return Ok(false);
    //                 }
    //             }
    //             return Ok(true);
    //         }
    //         None => return Err(format!("cmt {:?} hasnt been requested", cmt_root)),
    //     }
    // }
}

