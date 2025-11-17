pub mod api;
pub mod block;
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod configuration;
pub mod validator;
pub mod testimony;
pub mod mempool;
pub mod multichain;
pub mod fraudproof;
pub mod confirmation;
pub mod verifier;

use crate::{
    manifoldchain::{
        configuration::Configuration,
        mempool::Mempool,
        transaction::{
            generator::{
                self as Generator,
            },
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
        verifier::{
            self as Verifier,
        },
        confirmation::Confirmation,
    },
    types::{
        hash::H256,
        // random::Random,
    },
};

// use crossbeam::channel::{
//     unbounded,
//     Receiver,
//     Sender,
//     TryRecvError,
// };
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
    let exper_number = sub_com
        .value_of("exper_number")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment number: {}", e);
            process::exit(1);
        });
    let exper_iter = sub_com
        .value_of("exper_iter")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment iteration: {}", e);
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
    let confirmation_depth = sub_com
        .value_of("confirmation_depth")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the confirmation depth: {}", e);
            process::exit(1);
        });
    let exclusive_diff = sub_com
        .value_of("exclusive_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let inclusive_diff = sub_com
        .value_of("inclusive_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let domestic_ratio = sub_com
        .value_of("domestic_ratio")
        .unwrap()
        .parse::<f64>()
        .unwrap_or_else(|e| {
            error!("Error parsing the domestic ratio: {}", e);
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
    let exclusive_diff_bytes: [u8; 32] = decode_hex(exclusive_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let inclusive_diff_bytes: [u8; 32] = decode_hex(inclusive_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let exclusive_diff_hash: H256 = exclusive_diff_bytes.into();
    let inclusive_diff_hash: H256 = inclusive_diff_bytes.into();
    config.difficulty = exclusive_diff_hash;
    config.thredshold = inclusive_diff_hash;
    config.shard_id = shard_id as usize;
    config.node_id = node_id as usize;
    config.exper_number = exper_number as usize;
    config.exper_iter = exper_iter as usize;
    config.shard_num = shard_num as usize;
    config.shard_size = shard_size as usize;
    config.block_size = block_size as usize;
    config.k = confirmation_depth as usize;
    config.domestic_tx_ratio = domestic_ratio as f64;
    // let shard_id = format!("{:x}", shard_id);
    info!("configuration: {:?}", config);

    let api_port: u16 = api_addr.port();
    let chains: Vec<Arc<Mutex<Blockchain>>> = (0..config.shard_num)
        .into_iter()
        .map(|i| {
            let blockchain = Blockchain::new(&config, i);
            Arc::new(Mutex::new(blockchain))
        })
        .collect();
    let chains_ref: Vec<&Arc<Mutex<Blockchain>>> = chains
        .iter()
        .collect();
    let multichain = Multichain::create(chains_ref, &config);

    let mempool = Arc::new(
        Mutex::new(
            Mempool::new()
        )
    );

    let confirmation = Arc::new(
        Mutex::new(
            Confirmation::new(&multichain, &config)
        )
    );

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);


    //create the channel for tx generators
    let (tx_generator_sender, tx_generator_receiver) = Generator::create_channel();

    let tx_generator_handle = Generator::new_handle(&tx_generator_sender);

    // start the p2p server
    let (server_ctx, server) = NetworkServer::new(p2p_addr, msg_tx, &tx_generator_handle, config.shard_id).unwrap();
    server_ctx.start().unwrap();
    
    // start the worker
    let worker_ctx = NetworkWorker::new(
        p2p_workers,
        msg_rx,
        &server,
        &multichain,
        &mempool,
        &config,
        &confirmation,
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner, finished_block_chan) = Miner::new(&multichain, &mempool, &config);
    let miner_worker_ctx = MinerWorker::new(
        &server, 
        finished_block_chan, 
        &multichain,
        &confirmation,
        &config,
    );
    miner_ctx.start();
    miner_worker_ctx.start();


    //start the sample monitor
    let verifier_ctx = Verifier::new(&multichain, &server, &config);
    verifier_ctx.start();

    
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

    //start the transaction generator
    let generator_ctx = Generator::new_ctx(
        &tx_generator_receiver, 
        &server,
        &mempool, 
        &config, 
        api_port
    );
    generator_ctx.start();

    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &multichain,
        &tx_generator_handle,
        &mempool,
        &config,
    );

    loop {
        std::thread::park();
    }
}