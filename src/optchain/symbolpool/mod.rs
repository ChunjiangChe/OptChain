pub mod verifier;

use std::{
    time::{self, SystemTime},
    thread,
    sync::{Arc,Mutex},
    collections::HashMap,
};
use serde::{Serialize, Deserialize};
use rand::{
    thread_rng,
    seq::IteratorRandom,
    Rng,
};
use crate::{        
    optchain::{
        transaction::Transaction,
        database::Database,
        configuration::Configuration,
    },
    types::{
        hash::{H256, Hashable},
        merkle::MerkleTree,
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
    block_size: u32,
}

impl Hashable for SymbolIndex {
    fn hash(&self) -> H256 {
        H256::pow_hash(&self.root, self.index as u32)
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
            block_size: config.block_size as u32,
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
        MerkleTree::<Transaction>::verify(
            &self.index.get_root(), 
            &hash, 
            &self.merkle_proof, 
            self.index.get_index(), 
            self.block_size as usize,
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
        let mut hash2symbol: Database::<Symbol> = 
          Database::<Symbol>::new(format!("{:?}/symbolpool/hash2symbol", now));
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
    pub fn get_unreceived_symbols(&mut self, hash: &H256, ex_or_in: bool) -> Option<Vec<usize>> {
        match self.root2index.get(hash) {
            Some(indexs) => {
                let mut unreceived_indexs: Vec<usize> = vec![];
                for idx in indexs.iter() {
                    let symbol_index = SymbolIndex::new(hash.clone(), *idx);
                    match self.hash2symbol.get(&symbol_index.hash()) {
                        Some(_) => {}
                        None => unreceived_indexs.push(*idx),
                    }
                }
                if unreceived_indexs.is_empty() {
                    return None;
                } else {
                    return Some(unreceived_indexs);
                }

            }
            None => {
                let mut rng = thread_rng();
                let req_num = match ex_or_in {
                    true => self.config.ex_req_num,
                    false => self.config.in_req_num,
                };
                let request_indexs = (0..self.config.block_size).choose_multiple(&mut rng, req_num);
                self.root2index.insert(hash.clone(), request_indexs.clone());
                Some(request_indexs)
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
                        return Err(String::from("Symbol already existis"));
                    }
                    None => {
                        self.hash2symbol.insert(symbol_hash, sym);
                        return Ok(true);
                    }
                }
            }
            None => {
                return Err(String::from("Not an valid cmt_root"));
            }
        }
    }
}

