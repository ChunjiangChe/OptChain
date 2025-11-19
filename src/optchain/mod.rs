pub mod api;
pub mod block;
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod configuration;
// pub mod validator;
pub mod mempool;
pub mod multichain;
pub mod symbolpool;

use crate::{
    types::{
        hash::{
            H256,
            Hashable,
        },
        merkle::MerkleTree,
    },
    optchain::{
        configuration::Configuration,
        mempool::Mempool,
        block::{
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            ordering_block::OrderingBlock,
            versa_block::VersaBlock,
            transaction_block::TransactionBlock,
            BlockHeader,
        },
        network::{
            server as NetworkServer,
            worker::Worker as NetworkWorker,
        },
        api::Server as ApiServer,
        miner::{
            self as Miner,
            worker::Worker as MinerWorker,
        },
        blockchain::Blockchain as Blockchain,
        multichain::Multichain,
        symbolpool::{
            SymbolPool,
            // verifier::{
            //     self as Verifier,
            // }
        },
    },
};


// use crossbeam::channel::{
//     unbounded,
//     Receiver,
//     Sender,
//     TryRecvError,
// };
// use clap::clap_app;
use smol::channel;
use log::{error, info};
use std::{
    net, 
    process, 
    thread, 
    time, 
    sync::{Arc, Mutex},
    num::ParseIntError,
    convert::TryInto,
};
// use env_logger::Env;

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn start(sub_com: &clap::ArgMatches) {

    // parse p2p server address
    let p2p_addr = sub_com
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = sub_com
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let shard_id = sub_com
        .value_of("shard_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let node_id = sub_com
        .value_of("node_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the node id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let exper_number = sub_com
        .value_of("exper_number")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment number: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let exper_iter = sub_com
        .value_of("exper_iter")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment iter: {}", e);
            process::exit(1);
        });
    let shard_num = sub_com
        .value_of("shard_num")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard_num number: {}", e);
            process::exit(1);
        });
    let shard_size = sub_com
        .value_of("shard_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let block_size = sub_com
        .value_of("block_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the block size: {}", e);
            process::exit(1);
        });
    let symbol_size = sub_com
        .value_of("symbol_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the symbol size: {}", e);
            process::exit(1);
        });
    let prop_size = sub_com
        .value_of("prop_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the prop size: {}", e);
            process::exit(1);
        });
    let avai_size = sub_com
        .value_of("avai_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the avai size: {}", e);
            process::exit(1);
        });
    let ex_req_num = sub_com
        .value_of("ex_req_num")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the ex_req_num: {}", e);
            process::exit(1);
        });
    let in_req_num = sub_com
        .value_of("in_req_num")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the in_req_num: {}", e);
            process::exit(1);
        });
    let confirmation_depth = sub_com
        .value_of("confirmation_depth")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the confirmation depth: {}", e);
            process::exit(1);
        });
    let tx_diff = sub_com
        .value_of("tx_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let prop_diff = sub_com
        .value_of("prop_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let avai_diff = sub_com
        .value_of("avai_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let in_avai_diff = sub_com
        .value_of("in_avai_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let p2p_workers = sub_com
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
    
    
    let mut config = Configuration::new();
    let tx_diff_bytes: [u8; 32] = decode_hex(tx_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let prop_diff_bytes: [u8; 32] = decode_hex(prop_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let avai_diff_bytes: [u8; 32] = decode_hex(avai_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let in_avai_diff_bytes: [u8; 32] = decode_hex(in_avai_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let tx_diff_hash: H256 = tx_diff_bytes.into();
    let prop_diff_hash: H256 = prop_diff_bytes.into();
    let avai_diff_hash: H256 = avai_diff_bytes.into();
    let in_avai_diff_hash: H256 = in_avai_diff_bytes.into();
    config.tx_diff = tx_diff_hash;
    config.prop_diff = prop_diff_hash;
    config.avai_diff = avai_diff_hash;
    config.in_avai_diff = in_avai_diff_hash;
    config.block_size = block_size as usize;
    config.symbol_size = symbol_size as usize;
    assert!(block_size % symbol_size == 0);
    config.num_symbol_per_block = block_size / symbol_size;
    config.prop_size = prop_size as usize;
    config.avai_size = avai_size as usize;
    config.ex_req_num = ex_req_num as usize;
    config.in_req_num = in_req_num as usize;
    config.k = confirmation_depth as usize;
    config.shard_id = shard_id as usize;
    config.node_id = node_id as usize;
    config.exper_number = exper_number as usize;
    config.exper_iter = exper_iter as usize;
    config.shard_num = shard_num as usize;
    config.shard_size = shard_size as usize;
    // let shard_id = format!("{:x}", shard_id);
    info!("configuration: {:?}", config);

    // let api_port: u16 = api_addr.port();
    let prop_genesis_block = VersaBlock::PropBlock(ProposerBlock::default());
    let prop_chain = Blockchain::new(prop_genesis_block, &config);

    let mut genesis_avai_set: Vec<(H256, u32)> = vec![];
    let avai_chains: Vec<Blockchain> = (0..config.shard_num)
        .into_iter()
        .map(|i| {
            let mut header = BlockHeader::default();
            header.set_shard_id(i);
            let avai_block = AvailabilityBlock::new(
                header,
                0,
                MerkleTree::<TransactionBlock>::new((vec![]).as_slice())
            );
            let avai_genesis_block = VersaBlock::ExAvaiBlock(avai_block);
            genesis_avai_set.push((avai_genesis_block.hash(), i as u32));
            Blockchain::new(avai_genesis_block, &config)
        })
        .collect();
    // let chains_ref: Vec<&Arc<Mutex<Blockchain>>> = avai_chains
    //     .iter()
    //     .collect();
    let ordering_genesis_block = VersaBlock::OrderBlock(OrderingBlock::new(
        BlockHeader::default(),
        0,
        genesis_avai_set.clone(),
    ));
    let ordering_chain = Blockchain::new(ordering_genesis_block, &config);
    let multichain = Arc::new(
        Mutex::new(
            Multichain::new(
                prop_chain, 
                avai_chains, 
                ordering_chain, 
                &config
            )
        )
    );

    let mempool = Arc::new(
        Mutex::new(
            Mempool::new(&config)
        )
    );

    let symbolpool = Arc::new(
        Mutex::new(
            SymbolPool::new(&config)
        )
    );

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);

    // start the p2p server
    let (server_ctx, server) = NetworkServer::new(p2p_addr, msg_tx, config.shard_id).unwrap();
    server_ctx.start().unwrap();
    
    // start the worker
    let worker_ctx = NetworkWorker::new(
        p2p_workers,
        msg_rx,
        &server,
        &multichain,
        &mempool,
        &symbolpool,
        &config,
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner, finished_block_chan) = Miner::new(&multichain, &mempool, &config);
    let miner_worker_ctx = MinerWorker::new(
        &server, 
        finished_block_chan, 
        &multichain,
        &mempool,
        &symbolpool,
        &config,
    );
    miner_ctx.start();
    miner_worker_ctx.start();


    // //start the sample monitor
    // let verifier_ctx = Verifier::new(
    //     &multichain, 
    //     &server, 
    //     &config,
    //     &symbolpool,
    // );
    // verifier_ctx.start();

    
    // connect to known peers
    if let Some(known_peers) = sub_com.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });

    }

    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &multichain,
        &mempool,
        &config,
    );

    loop {
        std::thread::park();
    }
}
