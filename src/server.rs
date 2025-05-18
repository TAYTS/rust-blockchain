use std::{
    error::Error,
    io::{BufReader, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use data_encoding::HEXLOWER;
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::{
    block::Block,
    blockchain::Blockchain,
    config::GLOBAL_CONFIG,
    memory_pool::{BlockInTransit, MemoryPool},
    node::Nodes,
    transaction::Transaction,
    utxo_set::UTXOSet,
};

const NODE_VERSION: usize = 1;
pub const CENTRAL_NODE: &str = "127.0.0.1:2001";

pub const TRANSACTION_THRESHOLD: usize = 2;

static GLOBAL_NODES: Lazy<Nodes> = Lazy::new(|| {
    let nodes = Nodes::new();
    nodes.add_node(String::from(CENTRAL_NODE));
    return nodes;
});

static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool::new());

static GLOBAL_BLOCKS_IN_TRANSIT: Lazy<BlockInTransit> = Lazy::new(|| BlockInTransit::new());

const TCP_WRITE_TIMEOUT: u64 = 1000;

pub struct Server {
    blockchain: Blockchain,
}

impl Server {
    pub fn new(blockchain: Blockchain) -> Server {
        Server { blockchain }
    }

    pub fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).unwrap();
        if addr.eq(CENTRAL_NODE) == false {
            let best_height = self.blockchain.get_best_height();
            send_version(CENTRAL_NODE, best_height);
        }

        for stream in listener.incoming() {
            let blockchain = self.blockchain.clone();
            thread::spawn(|| match stream {
                Ok(stream) => {}
                Err(e) => {}
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OpType {
    Tx,
    Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Package {
    Block {
        addr_from: String,
        block: Vec<u8>,
    },
    GetBlocks {
        addr_from: String,
    },
    GetData {
        addr_from: String,
        op_type: OpType,
        id: Vec<u8>,
    },
    Inv {
        addr_from: String,
        op_type: OpType,
        items: Vec<Vec<u8>>,
    },
    Tx {
        addr_from: String,
        transaction: Vec<u8>,
    },
    Version {
        addr_from: String,
        version: usize,
        best_height: usize,
    },
}

fn send_get_data(addr: &str, op_type: OpType, id: &[u8]) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetData {
            addr_from: node_addr,
            op_type,
            id: id.to_vec(),
        },
    )
}

/// Sending inventory information to the specified address
fn send_inv(addr: &str, op_type: OpType, blocks: &[Vec<u8>]) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Inv {
            addr_from: node_addr,
            op_type,
            items: blocks.to_vec(),
        },
    )
}

fn send_block(addr: &str, block: &Block) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Block {
            addr_from: node_addr,
            block: block.serialize(),
        },
    )
}

pub fn send_tx(addr: &str, tx: &Transaction) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Tx {
            addr_from: node_addr,
            transaction: tx.serialize(),
        },
    );
}

fn send_version(addr: &str, height: usize) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Version {
            addr_from: node_addr,
            version: NODE_VERSION,
            best_height: height,
        },
    )
}

fn send_get_blocks(addr: &str) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetBlocks {
            addr_from: node_addr,
        },
    )
}

fn send_data(addr: SocketAddr, pkg: Package) {
    info!("send package: {:?}", pkg);
    let stream = TcpStream::connect(addr);
    if stream.is_err() {
        error!("The {} is not valid", addr);
        GLOBAL_NODES.evict_node(addr.to_string().as_str());
        return;
    }
    let mut stream = stream.unwrap();
    let _ = stream.set_write_timeout(Option::from(Duration::from_millis(TCP_WRITE_TIMEOUT)));
    let _ = serde_json::to_writer(&stream, &pkg);
    let _ = stream.flush();
}

fn serve(blockchain: Blockchain, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    let reader = BufReader::new(&stream);
    let pkg_reader = Deserializer::from_reader(reader).into_iter::<Package>();
    for pkg in pkg_reader {
        let pkg = pkg?;
        info!("Receive request from {}: {:?}", peer_addr, pkg);
        match pkg {
            Package::Block { addr_from, block } => {
                let block = Block::deserialize(block.as_slice());
                blockchain.add_block(&block);
                info!("Added block {}", block.get_hash());

                if GLOBAL_BLOCKS_IN_TRANSIT.len() > 0 {
                    let block_hash = GLOBAL_BLOCKS_IN_TRANSIT.first().unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, &block_hash);

                    GLOBAL_BLOCKS_IN_TRANSIT.remove(&block_hash);
                } else {
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                }
            }
            Package::GetBlocks { addr_from } => {
                let blocks = blockchain.get_block_hashes();
                send_inv(&addr_from.as_str(), OpType::Block, &blocks);
            }
            Package::GetData {
                addr_from,
                op_type,
                id,
            } => match op_type {
                OpType::Block => {
                    if let Some(block) = blockchain.get_block(id.as_slice()) {
                        send_block(addr_from.as_str(), &block);
                    }
                }
                OpType::Tx => {
                    let txid_hex = HEXLOWER.encode(id.as_slice());
                    if let Some(tx) = GLOBAL_MEMORY_POOL.get(txid_hex.as_str()) {
                        send_tx(addr_from.as_str(), &tx);
                    }
                }
            },
        }
    }

    let _ = stream.shutdown(Shutdown::Both);
    Ok(())
}
