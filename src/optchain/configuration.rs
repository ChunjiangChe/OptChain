use crate::types::hash::H256;


#[derive(Debug, Default, Clone)]
pub struct Configuration {
    pub tx_diff: H256,
    pub prop_diff: H256,
    pub avai_diff: H256,
    pub in_avai_diff: H256,
    pub block_size: usize,
    pub symbol_size: usize,
    pub num_symbol_per_block: usize,
    pub prop_size: usize,
    pub avai_size: usize,
    pub ex_req_num: usize,
    pub in_req_num: usize,
    pub k: usize,
    pub shard_id: usize,
    pub node_id: usize,
    pub shard_num: usize,
    pub shard_size: usize,
    pub exper_number: usize,
    pub exper_iter: usize,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration {
            tx_diff: H256::default(),
            prop_diff: H256::default(), 
            avai_diff: H256::default(),
            in_avai_diff: H256::default(),
            block_size: 0,
            symbol_size: 0,
            num_symbol_per_block: 0,
            prop_size: 0,
            avai_size: 0,
            ex_req_num: 0,
            in_req_num: 0,
            k: 6,
            shard_id: 0,
            node_id: 0,
            shard_num: 0,
            shard_size: 0,
            exper_number: 0,
            exper_iter: 0,
        }
    }
}
