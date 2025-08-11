use crate::{
    // optchain::{
    //     transaction::*,
    // },
    types::{
        random::Random,
        hash::H256,
    },
};


#[test]
fn test_block_header() {
    // let shard_id: u32 = 7;
    let prop_parent: H256 = H256::random();
    println!("random hash:{:?}", prop_parent);
}