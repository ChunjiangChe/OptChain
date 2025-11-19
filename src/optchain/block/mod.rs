pub mod transaction_block;
pub mod versa_block;
pub mod proposer_block;
pub mod availability_block;
pub mod ordering_block;

use serde::{Serialize, Deserialize};
use crate::{
    types::{
        hash::{H256, Hashable}, 
        merkle::MerkleTree,
        random::Random,
    },
    optchain::{
        transaction::Transaction,
        block::transaction_block::TransactionBlock,
    },
};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;

/*
------------
------------
------------
Block definition
------------
------------
------------
*/

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct BlockHeader {
    shard_id: u32, //the node's affiliated shard
    prop_parent: H256, //the hash of the highest proposer block
    inter_parent: H256, //the hash of the highest availability block in the shard_id-th shard
    global_parents: Vec<(H256, u32)>, //a set containning the hashes of the highest availability blocks across all shards.
    order_parent: H256, //the hash of the highest ordering block
    prop_root: H256, //the root of a Merkle tree generated from prop_tx_set
    avai_root: H256, //the root of a Merkle tree generated from avai_tx_set
    order_root: H256, //the root of a Merkle tree generated from confirm_avai_set
    cmt_root: H256, //the root of a CMT generated from data_blob (is currently replaced by a normal Merkle root)
    // nonce: u32,
    // difficulty: H256,
    timestamp: SystemTime,
    // merkle_root: H256,
}
#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct BlockContent {
    prop_tx_set: MerkleTree<TransactionBlock>, //a set of cmt_root of transaction blocks linked by proposer chain.
    avai_tx_set: MerkleTree<TransactionBlock>, //a set of cmt_root of transaction blocks linked by availability chains.
    confirmed_avai_set: Vec<(H256, u32)>, //a set of cmt_root of confirmed availability blocks, shard_id
    txs: Vec<Vec<Transaction>>, //a set of transactions
    symbol_merkle_tree: MerkleTree<H256>,
}


#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct Block {
    header: BlockHeader,
    content: BlockContent,
    hash: H256,
}

pub trait Content {
    fn get_prop_tx_set(&self) -> Vec<TransactionBlock>;
    fn get_prop_tx_set_ref(&self) -> &Vec<TransactionBlock>;

    fn get_avai_tx_set(&self) -> Vec<TransactionBlock>;
    fn get_avai_tx_set_ref(&self) -> &Vec<TransactionBlock>;

    fn get_confirmed_avai_set(&self) -> Vec<(H256, u32)>;
    fn get_confirmed_avai_set_ref(&self) -> &Vec<(H256, u32)>;

    fn get_txs(&self) -> Vec<Transaction>;
    // fn get_txs_ref(&self) -> &Vec<Transaction>;

    fn get_prop_merkle_tree(&self) -> MerkleTree<TransactionBlock>;
    fn get_avai_merkle_tree(&self) -> MerkleTree<TransactionBlock>;
    // fn get_txs_merkle_tree(&self) -> MerkleTree<Transaction>;
}

pub trait Info {
    fn get_shard_id(&self) -> usize;
    fn get_prop_parent(&self) -> H256;
    fn get_inter_parent(&self) -> H256;
    fn get_global_parents(&self) -> Vec<(H256, usize)>;
    fn get_order_parent(&self) -> H256;
    fn get_prop_root(&self) -> H256;
    fn get_avai_root(&self) -> H256;
    fn get_order_root(&self) -> H256;
    fn get_cmt_root(&self) -> H256;
    fn get_timestamp(&self) -> SystemTime;
    fn get_info_hash(&self) -> Vec<H256>;
}


/*
------------
------------
------------
Block Header
------------
------------
------------
*/

impl Random for BlockHeader {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let shard_id: u32 = rng.gen();
        let prop_parent = H256::random();
        let inter_parent = H256::random();
        let global_parents = vec![(prop_parent.clone(), shard_id)];
        let order_parent = H256::random();
        let prop_root = H256::random();
        let avai_root = H256::random();
        let order_root = H256::random();
        let cmt_root = H256::random();
        BlockHeader {
            shard_id,
            prop_parent,
            inter_parent,
            global_parents,
            order_parent,
            prop_root,
            avai_root,
            order_root,
            cmt_root,
            timestamp: SystemTime::now(),
        }
    }
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BlockHeader(shard_id: {}, hash: {:?})", self.shard_id, self.hash())
    }
}



impl Hashable for BlockHeader {
    fn hash(&self) -> H256 {
        let info_vec = self.get_info_hash(); 
        let info_hash: H256 = H256::multi_hash(&info_vec);
        let global_parents_vec: Vec<H256> = self.global_parents
                                             .iter()
                                             .map(|(hash, _)| hash.clone())
                                             .collect();
        let global_hash: H256 = H256::multi_hash(&global_parents_vec);
        let all_hashes: Vec<H256> = vec![
            info_hash, 
            self.prop_parent.clone(), 
            self.inter_parent.clone(), 
            global_hash, 
            self.prop_root.clone(), 
            self.avai_root.clone(),
            self.cmt_root.clone(),
            ];
        let all_hash: H256 = H256::multi_hash(&all_hashes);
        all_hash
    }
}

impl Default for BlockHeader {
    fn default() -> Self {
        BlockHeader {
            // parent: H256::default(),
            // nonce: 0 as u32,
            // difficulty: H256::default(),
            // merkle_root: H256::default(),
            shard_id: 0 as u32,
            prop_parent: H256::default(),
            inter_parent: H256::default(),
            global_parents: vec![],
            order_parent: H256::default(),
            prop_root: H256::default(),
            avai_root: H256::default(),
            order_root: H256::default(),
            cmt_root: H256::default(),
            timestamp: SystemTime::from(UNIX_EPOCH + Duration::new(0,0)),
        }
    }
}


impl BlockHeader {
    pub fn create(
        shard_id: usize,
        prop_parent: H256,
        inter_parent: H256,
        global_parents: Vec<(H256, usize)>,
        order_parent: H256,
        prop_root: H256,
        avai_root: H256,
        order_root: H256,
        cmt_root: H256,
        // parent: H256, 
        // nonce: usize, 
        // difficulty: H256,  
        timestamp: SystemTime,
        // merkle_root: H256
    ) -> Self {
        let global_parents = global_parents
                                .iter()
                                .map(|(hash, shard_id)| (hash.clone(), *shard_id as u32))
                                .collect();
        let shard_id: u32 = u32::try_from(shard_id).expect("Shard id does not fit in u32!");
        BlockHeader {
            // parent, 
            // nonce: nonce as u32,
            // difficulty,
            shard_id,
            prop_parent,
            inter_parent,
            global_parents,
            order_parent,
            prop_root,
            avai_root,
            order_root,
            cmt_root,
            timestamp,
            // merkle_root
        }
    }
    pub fn get_mem_size(&self) -> usize {
        H256::get_mem_size() * (5+self.global_parents.len())
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<SystemTime>()
    }
    pub fn set_shard_id(&mut self, shard_id: usize) {
        self.shard_id = shard_id as u32;
    }
}

impl Info for BlockHeader {
    
    fn get_shard_id(&self) -> usize {
        self.shard_id as usize
    }
    fn get_prop_parent(&self) -> H256 {
        self.prop_parent.clone()
    }
    fn get_inter_parent(&self) -> H256 {
        self.inter_parent.clone()
    }
    fn get_global_parents(&self) -> Vec<(H256, usize)> {
        self.global_parents.iter()
                           .map(|(hash, shard_id)| (hash.clone(), *shard_id as usize))
                           .collect()
    }
    fn get_order_parent(&self) -> H256 {
        self.order_parent.clone()
    }
    fn get_prop_root(&self) -> H256 {
        self.prop_root.clone()
    }
    fn get_avai_root(&self) -> H256 {
        self.avai_root.clone()
    }
    fn get_order_root(&self) -> H256 {
        self.order_root.clone()
    }
    fn get_cmt_root(&self) -> H256 {
        self.cmt_root.clone()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.timestamp.clone()
    }
    
    fn get_info_hash(&self) -> Vec<H256> {
        let time_str = format!("{:?}", self.timestamp);
        let time_hash: H256 = ring::digest::digest(
            &ring::digest::SHA256,
            time_str.as_bytes()
        ).into();
        let shard_id_hash :H256 = ring::digest::digest(
            &ring::digest::SHA256,
            &self.shard_id.to_be_bytes()
        ).into();
        vec![
            // self.difficulty.clone(),
            time_hash,
            shard_id_hash,
            // self.merkle_root.clone(),
        ]
    }
}

/*
------------
------------
------------
Block Content
------------
------------
------------
*/


impl Default for BlockContent {
    fn default() -> Self {
        BlockContent {
            prop_tx_set: MerkleTree::<TransactionBlock>::new(&[]),
            avai_tx_set: MerkleTree::<TransactionBlock>::new(&[]),
            confirmed_avai_set: vec![],
            txs: vec![],
            symbol_merkle_tree: MerkleTree::<H256>::new(&[])
        }
    }
}

impl Content for BlockContent {
    fn get_prop_tx_set(&self) -> Vec<TransactionBlock> {
        self.prop_tx_set.data.clone()
    }
    fn get_prop_tx_set_ref(&self) -> &Vec<TransactionBlock> {
        &self.prop_tx_set.data
    }

    fn get_avai_tx_set(&self) -> Vec<TransactionBlock> {
        self.avai_tx_set.data.clone()
    }
    fn get_avai_tx_set_ref(&self) -> &Vec<TransactionBlock> {
        &self.avai_tx_set.data
    }

    fn get_confirmed_avai_set(&self) -> Vec<(H256, u32)> {
        self.confirmed_avai_set.clone()
    }
    fn get_confirmed_avai_set_ref(&self) -> &Vec<(H256, u32)> {
        &self.confirmed_avai_set
    }



    fn get_txs(&self) -> Vec<Transaction> {
        self.txs.clone().concat()
    }
    // fn get_txs_ref(&self) -> &Vec<Transaction> {
    //     &self.txs.data
    // }

    fn get_prop_merkle_tree(&self) -> MerkleTree<TransactionBlock> {
        self.prop_tx_set.clone()
    }
    fn get_avai_merkle_tree(&self) -> MerkleTree<TransactionBlock> {
        self.avai_tx_set.clone()
    }
    // fn get_txs_merkle_tree(&self) -> MerkleTree<Transaction> {
    //     self.txs.clone()
    // }
}

impl BlockContent {
    pub fn create(
        prop_tx_set: MerkleTree<TransactionBlock>,
        avai_tx_set: MerkleTree<TransactionBlock>,
        confirmed_avai_set: Vec<(H256, u32)>,
        txs: Vec<Vec<Transaction>>
    ) -> Self {
        let symbols: Vec<H256> = txs.iter()
                            .map(|tx_block| {
                                let tx_hashs = tx_block
                                                .iter()
                                                .map(|tx| tx.hash())
                                                .collect();
                                H256::multi_hash(&tx_hashs)
                            })
                            .collect();
        let symbol_merkle_tree = MerkleTree::<H256>::new(symbols.as_slice());
        Self {
            prop_tx_set,
            avai_tx_set,
            confirmed_avai_set,
            txs,
            symbol_merkle_tree,
        }
    }
    pub fn get_prop_merkle_root(&self) -> H256 {
        self.prop_tx_set.root.clone()
    }
    pub fn get_prop_merkle_proof(&self, index: usize) -> Vec<H256> {
        self.prop_tx_set.proof(index)
    }
    pub fn prop_merkle_prove(&self, datum: &H256, proof: &Vec<H256>, index: usize) ->bool {
        self.prop_tx_set.merkle_prove(datum, proof, index)
    }

    pub fn get_avai_merkle_root(&self) -> H256 {
        self.avai_tx_set.root.clone()
    }
    pub fn get_avai_merkle_proof(&self, index: usize) -> Vec<H256> {
        self.avai_tx_set.proof(index)
    }
    pub fn avai_merkle_prove(&self, datum: &H256, proof: &Vec<H256>, index: usize) ->bool {
        self.avai_tx_set.merkle_prove(datum, proof, index)
    }

    pub fn get_symbol_merkle_root(&self) -> H256 {
        self.symbol_merkle_tree.root.clone()
    }
    pub fn get_symbol_merkle_proof(&self, tx_index: usize) -> Vec<H256> {
        self.symbol_merkle_tree.proof(tx_index)
    }
    pub fn symbol_merkle_prove(&self, datum: &H256, proof: &Vec<H256>, index: usize) ->bool {
        self.symbol_merkle_tree.merkle_prove(datum, proof, index)
    }

    pub fn get_symbol_txs(&self, index: usize) -> Result<Vec<Transaction>, String> {
        match self.txs.get(index) {
            Some(txs) => Ok(txs.clone()),
            None => Err(format!("Index {} is out of boundary", index)),
        }
    }

    // pub fn get_tx_merkle_root(&self) -> H256 {
    //     self.txs_merkle_tree.root.clone()
    // }
    // pub fn get_tx_merkle_proof(&self, tx_index: usize) -> Vec<H256> {
    //     self.txs.proof(tx_index)
    // }
    // pub fn tx_merkle_prove(&self, datum: &H256, proof: &Vec<H256>, index: usize) ->bool {
    //     self.txs.merkle_prove(datum, proof, index)
    // }
}

/*
------------
------------
------------
Block
------------
------------
------------
*/

impl Content for Block {
    fn get_prop_tx_set(&self) -> Vec<TransactionBlock> {
        self.content.get_prop_tx_set()
    }
    fn get_prop_tx_set_ref(&self) -> &Vec<TransactionBlock> {
        self.content.get_prop_tx_set_ref()
    }

    fn get_avai_tx_set(&self) -> Vec<TransactionBlock> {
        self.content.get_avai_tx_set()
    }
    fn get_avai_tx_set_ref(&self) -> &Vec<TransactionBlock> {
        self.content.get_avai_tx_set_ref()
    }

    fn get_confirmed_avai_set(&self) -> Vec<(H256, u32)> {
        self.content.get_confirmed_avai_set()
    }
    fn get_confirmed_avai_set_ref(&self) -> &Vec<(H256, u32)> {
        self.content.get_confirmed_avai_set_ref()
    }


    fn get_txs(&self) -> Vec<Transaction> {
        self.content.get_txs()
    }
    // fn get_txs_ref(&self) -> &Vec<Transaction> {
    //     self.content.get_txs_ref()
    // }
    
    fn get_prop_merkle_tree(&self) -> MerkleTree<TransactionBlock> {
        self.content.get_prop_merkle_tree()
    }
    fn get_avai_merkle_tree(&self) -> MerkleTree<TransactionBlock> {
        self.content.get_avai_merkle_tree()
    }
    // fn get_txs_merkle_tree(&self) -> MerkleTree<Transaction> {
    //     self.content.get_txs_merkle_tree()
    // }
}



impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl Info for Block {
    fn get_shard_id(&self) -> usize {
        self.header.get_shard_id()
    }
    fn get_prop_parent(&self) -> H256 {
        self.header.get_prop_parent()
    }
    fn get_inter_parent(&self) -> H256 {
        self.header.get_inter_parent()
    }
    fn get_global_parents(&self) -> Vec<(H256, usize)> {
        self.header.get_global_parents()
    }
    fn get_order_parent(&self) -> H256 {
        self.header.get_order_parent()
    }
    fn get_prop_root(&self) -> H256 {
        self.header.get_prop_root()
    }
    fn get_avai_root(&self) -> H256 {
        self.header.get_avai_root()
    }
    fn get_order_root(&self) -> H256 {
        self.header.get_order_root()
    }
    fn get_cmt_root(&self) -> H256 {
        self.header.get_cmt_root()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.header.get_timestamp()
    }
    // fn get_tx_merkle_root(&self) -> H256 {
    //     self.header.get_tx_merkle_root()
    // }
    fn get_info_hash(&self) -> Vec<H256> {
        self.header.get_info_hash()
    }
}

impl Default for Block {
    fn default() -> Self {
        let header = BlockHeader::default();
        let content = BlockContent::default();

        let hash: H256 = header.hash();

        Block {
            header,
            content,
            hash,
        }
    }
}

impl Block {
    pub fn verify_hash(blk: &Block) -> bool {
        blk.hash() == blk.header.hash()
    }

    pub fn get_header(&self) -> BlockHeader {
        self.header.clone()
    }

    pub fn get_content(&self) -> BlockContent {
        self.content.clone()
    }

    // pub fn get_tx_merkle_proof(&self, tx_index: usize) -> Vec<H256> {
    //     self.content.get_tx_merkle_proof(tx_index)
    // }

    pub fn construct(
        shard_id: usize, 
        prop_parent: H256, 
        inter_parent: H256,
        global_parents: Vec<(H256, usize)>, 
        order_parent: H256,
        prop_tx_set: Vec<TransactionBlock>, 
        avai_tx_set: Vec<TransactionBlock>, 
        confirmed_avai_set: Vec<(H256, u32)>,
        txs: Vec<Vec<Transaction>>
    ) -> Block {

        // let txs = MerkleTree::<Transaction>::new(txs.as_slice());
        let symbols: Vec<H256> = txs.iter()
                            .map(|tx_block| {
                                let tx_hashs = tx_block
                                                .iter()
                                                .map(|tx| tx.hash())
                                                .collect();
                                H256::multi_hash(&tx_hashs)
                            })
                            .collect();
        let symbol_merkle_tree = MerkleTree::<H256>::new(symbols.as_slice());

        let prop_tx_set = MerkleTree::<TransactionBlock>::new(prop_tx_set.as_slice());
        let avai_tx_set = MerkleTree::<TransactionBlock>::new(avai_tx_set.as_slice());
        let confirmed_avai_hashes: Vec<H256> = confirmed_avai_set
                                        .iter()
                                        .map(|(h, shard_id)| H256::pow_hash(h, *shard_id))
                                        .collect(); 
        let confirmed_avai_root = H256::multi_hash(&confirmed_avai_hashes);
        

        let header: BlockHeader = BlockHeader {
            shard_id: shard_id as u32, 
            prop_parent,
            inter_parent,
            order_parent,
            global_parents: global_parents.iter()
                                          .map(|(hash, shard_id)| (hash.clone(), *shard_id as u32))
                                          .collect(),
            prop_root: prop_tx_set.root.clone(),
            avai_root: avai_tx_set.root.clone(),
            order_root: confirmed_avai_root,
            cmt_root: symbol_merkle_tree.root.clone(),
            timestamp: SystemTime::now(),
        };

        let content: BlockContent = BlockContent {
            prop_tx_set,
            avai_tx_set,
            confirmed_avai_set,
            txs,
            symbol_merkle_tree,
        };

        let hash: H256 = header.hash();

        Block {
            header, 
            content,
            hash,
        }
    }

    
}



