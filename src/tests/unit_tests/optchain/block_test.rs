use crate::{
    // optchain::{
    //     transaction::*,
    // },
    bitcoin::block, optchain::{
        block::{
            Block, BlockHeader, Content, availability_block::AvailabilityBlock, proposer_block::ProposerBlock, transaction_block::TransactionBlock
        },
        transaction::Transaction,
    }, types::{
        hash::{
            H256,
            Hashable,
        }, random::Random
    }
};

use serde::{Serialize, Deserialize};
use bincode;
// use std::time::SystemTime;
// use rand::Rng;
// use log::debug;


// #[test]
// fn test_block_header() {
//     //test a random block header
//     let random_block_header = BlockHeader::random();
//     println!("random block header: {:?}", random_block_header);
//     debug!("test debug for block header: {:?}", random_block_header);

//     //test a hash function
//     let hash = random_block_header.hash();
//     let copy_block_header = random_block_header.clone();
//     let copy_hash = copy_block_header.hash();
//     assert_eq!(hash, copy_hash);

//     let default_block_header = BlockHeader::default();
//     let default_hash = default_block_header.hash();
//     assert_ne!(hash, default_hash);

//     // test a create function
//     // If the usize number is too large, the program will 
//     // panic (error get handled in BlockkHeader::create function)
//     // let mut rng = rand::thread_rng();
//     // let shard_id: usize = rng.gen();
//     let shard_id: usize = 10;
//     let prop_parent = H256::random();
//     let inter_parent = H256::random();
//     let global_parents = vec![(prop_parent.clone(), shard_id)];
//     let prop_root = H256::random();
//     let avai_root = H256::random();
//     let cmt_root = H256::random();
//     let timestamp = SystemTime::now();

//     let block_header = BlockHeader::create(
//         shard_id, 
//         prop_parent.clone(),
//         inter_parent.clone(),
//         global_parents.clone(),
//         prop_root.clone(),
//         avai_root.clone(),
//         cmt_root.clone(),
//         timestamp.clone()
//     );

//     //test a info trait
//     assert_eq!(shard_id, block_header.get_shard_id());
//     assert_eq!(prop_parent, block_header.get_prop_parent());
//     assert_eq!(inter_parent, block_header.get_inter_parent());
//     assert_eq!(global_parents, block_header.get_global_parents());
//     assert_eq!(prop_root, block_header.get_prop_root());
//     assert_eq!(avai_root, block_header.get_avai_root());
//     assert_eq!(cmt_root, block_header.get_cmt_root());
//     assert_eq!(timestamp, block_header.get_timestamp());

//     let _ = block_header.get_info_hash();

// }

// #[test]
// fn test_transaction_block() {
//     let block_header = BlockHeader::random();
//     let nonce: u32 = 100;
//     let transaction_block = TransactionBlock::new(
//         block_header.clone(),
//         nonce,
//     );
//     assert_eq!(block_header.get_shard_id(), transaction_block.get_shard_id());
//     assert_eq!(block_header.get_prop_parent(), transaction_block.get_prop_parent());
//     assert_eq!(block_header.get_inter_parent(), transaction_block.get_inter_parent());
//     assert_eq!(block_header.get_global_parents(), transaction_block.get_global_parents());
//     assert_eq!(block_header.get_prop_root(), transaction_block.get_prop_root());
//     assert_eq!(block_header.get_avai_root(), transaction_block.get_avai_root());
//     assert_eq!(block_header.get_cmt_root(), transaction_block.get_cmt_root());
//     assert_eq!(block_header.get_timestamp(), transaction_block.get_timestamp());
//     assert_eq!(block_header.get_info_hash(), transaction_block.get_info_hash());
//     assert_eq!(nonce, transaction_block.get_nonce());
// }

// #[test]
// fn test_block_content() {
//     let prop_tx_set: Vec<TransactionBlock> = (0..7)
//         .map(|_| TransactionBlock::random())
//         .collect();

//     let avai_tx_set: Vec<TransactionBlock> = (0..7)
//         .map(|_| TransactionBlock::random())
//         .collect();

//     let block_size = 99;
//     let txs: Vec<Transaction> = (0..block_size)
//         .map(|_| Transaction::random())
//         .collect();
//     let merkle_txs = MerkleTree::<Transaction>::new(&txs);

//     let block_content = BlockContent::create(
//         MerkleTree::<TransactionBlock>::new(&prop_tx_set),
//         MerkleTree::<TransactionBlock>::new(&avai_tx_set),
//         merkle_txs
//     );

//     let mut rng = rand::thread_rng();
//     let prop_index: usize = rng.gen_range(0..7);
//     let avai_index: usize = rng.gen_range(0..7);
//     let tx_index: usize = rng.gen_range(0..block_size);

//     let prop_proof = block_content.get_prop_merkle_proof(prop_index);
//     let avai_proof = block_content.get_avai_merkle_proof(avai_index);
//     let tx_proof = block_content.get_tx_merkle_proof(tx_index);

//     let prop_datum = prop_tx_set.get(prop_index).unwrap().hash();
//     assert!(block_content.prop_merkle_prove(&prop_datum, &prop_proof, prop_index));

//     let avai_datum = avai_tx_set.get(avai_index).unwrap().hash();
//     assert!(block_content.avai_merkle_prove(&avai_datum, &avai_proof, avai_index));

//     let tx_datum = txs.get(tx_index).unwrap().hash();
//     assert!(block_content.tx_merkle_prove(&tx_datum, &tx_proof, tx_index));

// }

#[test]
fn test_block_size() {
    let block_size = 2048;
    let symbol_size = 64;
    let txs: Vec<Vec<Transaction>> = (0..(block_size / symbol_size))
        .into_iter()
        .map(|_| {
            let sym: Vec<Transaction> = (0..symbol_size)
                .into_iter()
                .map(|_| Transaction::random())
                .collect();
            sym
        }).collect();
    let prop_tx_set: Vec<TransactionBlock> = (0..3)
        .into_iter()
        .map(|i| {
            let block_header = BlockHeader::random();
            let tx_block = TransactionBlock::new(
                block_header,
                0,
            );
            tx_block
        }).collect();
    let avai_tx_set: Vec<TransactionBlock> = (0..3)
        .into_iter()
        .map(|i| {
            let block_header = BlockHeader::random();
            let tx_block = TransactionBlock::new(
                block_header,
                0,
            );
            tx_block
        }).collect();
    let confirmed_avai_tx_set: Vec<(H256, u32)> = vec![
        (H256::default(), 0),
        (H256::default(), 1),
        (H256::default(), 2),
        (H256::default(), 3),
    ];
    let hybrid_block = Block::construct(
        0,
        H256::default(),
        H256::default(),
        vec![(H256::default(), 0), (H256::default(), 1), (H256::default(), 2), (H256::default(), 3)],
        H256::default(),        
        prop_tx_set,
        avai_tx_set,
        confirmed_avai_tx_set,
        txs,
    );

    let block_header = hybrid_block.get_header();
    let block_content = hybrid_block.get_content();

    let prop_merkle_tree = block_content.get_prop_merkle_tree();
    let avai_merkle_tree = block_content.get_avai_merkle_tree();

    let prop_block = ProposerBlock::new(
        block_header.clone(),
        0,
        prop_merkle_tree.clone(),
    );
    let avai_block = AvailabilityBlock::new(
        block_header.clone(),
        0,
        avai_merkle_tree.clone(),
    );

    let serialized_hybrid_block = bincode::serialize(&hybrid_block).unwrap();
    let serialized_prop_block = bincode::serialize(&prop_block).unwrap();
    let serialized_avai_block = bincode::serialize(&avai_block).unwrap();

    println!("Size of serialized hybrid block: {} bytes", serialized_hybrid_block.len());
    println!("Size of serialized proposer block: {} bytes", serialized_prop_block.len());
    println!("Size of serialized availability block: {} bytes", serialized_avai_block.len());


    
}