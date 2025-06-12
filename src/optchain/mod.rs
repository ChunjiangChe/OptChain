pub mod api;
pub mod block;
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod configuration;
pub mod validator;
pub mod mempool;
pub mod multichain;
pub mod database;
pub mod symbolpool;

use crate::{
    types::hash::{
        H256,
    },
    optchain::{
        configuration::Configuration,
        mempool::Mempool,
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
            verifier::{
                self as Verifier,
            }
        },
    },
};


use crossbeam::channel::{
    unbounded,
    Receiver,
    Sender,
    TryRecvError,
};
use clap::clap_app;
use smol::channel;
use log::{error, info, debug};
use std::{
    net, 
    process, 
    thread, 
    time, 
    sync::{Arc, Mutex},
    num::ParseIntError,
    convert::TryInto,
};
use env_logger::Env;

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn start() {
    // parse command line arguments
    let matches = clap_app!(OptChain =>
     (version: "0.1")
     (about: "OptChain client")
     (@arg verbose:
            -v ... 
            "Increases the verbosity of logging")
     (@arg peer_addr: 
            --p2p [ADDR] 
            default_value("127.0.0.1:6000") 
            "Sets the IP address and the port of the P2P server")
     (@arg api_addr: 
            --api [ADDR] 
            default_value("127.0.0.1:7000") 
            "Sets the IP address and the port of the API server")
     (@arg known_peer: 
            -c --connect ... [PEER] 
            "Sets the peers to connect to at start")
     (@arg p2p_workers: 
            --("p2p-workers") [INT] 
            default_value("4") 
            "Sets the number of worker threads for P2P server")
    (@arg shard_id:
            --shardId [INT]
            "Sets the shard id of the node")
    (@arg node_id:
            --nodeId [INT]
            "Sets the id of the node")
    (@arg exper_number:
            --experNumber [INT]
            "Sets the number of experiment")
    (@arg shard_num:
            --shardNum [INT]
            "Sets the number of shards")
    (@arg shard_size:
            --shardSize [INT]
            "Sets the size of shards")
    (@arg block_size:
            --blockSize [INT]
            "Sets the size of block")
    (@arg confirmation_depth:
            --k [INT]
            "Sets the confirmation_depth")
    (@arg tx_diff:
            --tDiff [STR]
            "Sets the difficulty of mining a transaction block")
    (@arg prop_diff:
            --pDiff [STR]
            "Sets the difficulty of mining a proposer block")
    (@arg avai_diff:
            --aDiff [STR]
            "Sets the difficulty of mining an availability block")
    (@arg in_avai_diff:
            --iDiff [STR]
            "Sets the difficulty of mining an inclusive availability block")
    )
    .get_matches();

    // init logger
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    //let verbosity = matches.occurrences_of("verbose") as usize;
    //stderrlog::new().verbosity(verbosity).init().unwrap();

    // parse p2p server address
    let p2p_addr = matches
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = matches
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let shard_id = matches
        .value_of("shard_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let node_id = matches
        .value_of("node_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the node id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let exper_number = matches
        .value_of("exper_number")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment number: {}", e);
            process::exit(1);
        });
    let shard_num = matches
        .value_of("shard_num")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard_num number: {}", e);
            process::exit(1);
        });
    let shard_size = matches
        .value_of("shard_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let block_size = matches
        .value_of("block_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the block size: {}", e);
            process::exit(1);
        });
    let confirmation_depth = matches
        .value_of("confirmation_depth")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the confirmation depth: {}", e);
            process::exit(1);
        });
    let tx_diff = matches
        .value_of("tx_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let prop_diff = matches
        .value_of("prop_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let avai_diff = matches
        .value_of("avai_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let in_avai_diff = matches
        .value_of("in_avai_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
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
    config.k = confirmation_depth as usize;
    config.shard_id = shard_id as usize;
    config.node_id = node_id as usize;
    config.exper_number = exper_number as usize;
    config.shard_num = shard_num as usize;
    config.shard_size = shard_size as usize;
    let shard_id = format!("{:x}", shard_id);
    info!("configuration: {:?}", config);

    let api_port: u16 = api_addr.port();
    let prop_chain: Arc<Mutex<Blockchain>> = Arc::new(
        Mutex::new(
            Blockchain::new(&config)
        )
    );
    let avai_chains: Vec<Arc<Mutex<Blockchain>>> = (0..config.shard_num)
        .into_iter()
        .map(|i| {
            let blockchain = Blockchain::new(&config);
            Arc::new(Mutex::new(blockchain))
        })
        .collect();
    let chains_ref: Vec<&Arc<Mutex<Blockchain>>> = avai_chains
        .iter()
        .collect();
    let multichain = Multichain::create(&prop_chain, chains_ref, &config);

    let mempool = Arc::new(
        Mutex::new(
            Mempool::new()
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
    let p2p_workers = matches
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
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


    //start the sample monitor
    let verifier_ctx = Verifier::new(
        &multichain, 
        &server, 
        &config,
        &symbolpool,
    );
    verifier_ctx.start();

    
    // connect to known peers
    if let Some(known_peers) = matches.values_of("known_peer") {
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
