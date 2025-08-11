use serde::Serialize;
use crate::{
    optchain::{
        multichain::Multichain,
        miner::Handle as MinerHandle,
        network::{
            server::Handle as NetworkServerHandle,
            message::Message,
        },
        mempool::Mempool,
        // validator::{
        //     Validator,
        // },
        configuration::Configuration,
    },
    // types::{
    //     hash::{
    //         H256,
    //         Hashable,
    //     }
    // },
};

use log::{info};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};
use tiny_http::{
    Header,
    Response,
    Server as HTTPServer,
};
use url::Url;

#[allow(dead_code)]
pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
    network: NetworkServerHandle,
    multichain: Multichain,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
// macro_rules! respond_json {
//     ( $req:expr, $message:expr ) => {{
//         let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
//         let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
//             .with_header(content_type);
//         $req.respond(resp).unwrap();
//     }};
// }

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        network: &NetworkServerHandle,
        multichain: &Multichain,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            network: network.clone(),
            multichain: multichain.clone(),
            mempool: Arc::clone(mempool),
            config: config.clone(),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let network = server.network.clone();
                // let multichain = server.multichain.clone();
                // let mempool = Arc::clone(&server.mempool);
                // let config = server.config.clone();
                // let validator = Validator::new(
                //     &multichain,
                //     &mempool,
                //     &config,
                // );
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/miner/end" => {
                            miner.exit();
                            respond_result!(req, true, "ok");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        // "/blockchain/log" => {
                        //     multichain.log_to_file_with_shard(config.shard_id);
                        //     respond_result!(req, true, "ok");
                        // }
                        // "/blockchain/longest-chain" => {
                        //     let v = multichain.all_blocks_in_longest_chain();
                        //     let v_string: Vec<String> = v
                        //         .into_iter()
                        //         .map(|h| {
                        //             let str = h.to_string();
                        //             let left_slice = &str[0..3];
                        //             let right_slice = &str[61..64];
                        //             format!("{left_slice}..{right_slice}")
                        //         })
                        //         .collect();
                        //     respond_json!(req, v_string);
                        // }
                        // "/blockchain/longest-chain-with-shard" => {
                        //     let params = url.query_pairs();
                        //     let params: HashMap<_, _> = params.into_owned().collect();
                        //     let shard_id = match params.get("shard-id") {
                        //         Some(v) => v,
                        //         None => {
                        //             respond_result!(req, false, "missing shard id");
                        //             return;
                        //         }
                        //     };
                        //     let shard_id = match shard_id.parse::<usize>() {
                        //         Ok(v) => v,
                        //         Err(e) => {
                        //             respond_result!(
                        //                 req, 
                        //                 false, 
                        //                 format!("error parsing shard id: {}", e)
                        //             );
                        //             return;
                        //         }
                        //     };

                        //     let v = multichain.all_blocks_in_longest_chain_with_shard(shard_id);
                        //     let v_string: Vec<String> = v
                        //         .into_iter()
                        //         .map(|h| {
                        //             let str = h.to_string();
                        //             let left_slice = &str[0..3];
                        //             let right_slice = &str[61..64];
                        //             format!("{left_slice}..{right_slice}")
                        //         })
                        //         .collect();
                        //     respond_json!(req, v_string);
                        // }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
