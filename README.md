# OptChain

OptChain is a high-performance, sharding Proof-of-Work (PoW) blockchain protocol implemented in Rust. It utilizes a novel multi-chain architecture—consisting of Proposer, Availability, and Ordering chains—to optimize throughput and data availability verification.

This repository contains the client implementation for running an OptChain node, including networking, mining, and consensus logic.

## Prerequisites

To compile and run OptChain, you need to have the Rust toolchain installed.

- **Rust**: Latest stable version (install via [rustup.rs](https://rustup.rs/))

## Building the Project

Clone the repository and build the release binary using Cargo:

```bash
cargo build --release
```

The executable will be located at `./target/release/Powchain`.

## Usage

OptChain is run via the command-line interface. The binary supports multiple protocols, but this guide focuses on the `optchain` subcommand.

### Basic Syntax

```bash
./target/release/Powchain optchain [FLAGS] [OPTIONS]
```

### Running a Node

Because OptChain is a research prototype, many configuration parameters (such as block sizes, difficulties, and sharding setups) must be explicitly defined via command-line flags. If a required flag is omitted, the client will exit with an error.

#### Example Command

Here is an example of how to start a single node (Shard 0, Node 0) with low difficulty (for testing purposes):

```bash
./target/release/Powchain optchain \
  --p2p 127.0.0.1:6000 \
  --api 127.0.0.1:7000 \
  --shardId 0 \
  --nodeId 0 \
  --shardNum 1 \
  --shardSize 1 \
  --experNumber 1 \
  --experIter 1 \
  --blockSize 1024 \
  --symbolSize 32 \
  --propSize 100 \
  --avaiSize 100 \
  --eReq 5 \
  --iReq 5 \
  --k 6 \
  --tDiff ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
  --pDiff ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
  --aDiff ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
  --iDiff ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
  --oDiff ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
```

### Configuration Options

Below is a detailed list of all available configuration flags for the `optchain` subcommand.

#### Network Configuration

| Flag | Default | Description |
| --- | --- | --- |
| `--p2p [ADDR]` | `127.0.0.1:6000` | The IP address and port for the P2P server to listen on. |
| `--api [ADDR]` | `127.0.0.1:7000` | The IP address and port for the API server (handling RPCs). |
| `-c, --connect [PEER]` | None | A known peer address to connect to at startup. Can be used multiple times. |
| `--p2p-workers [INT]` | `1` | Number of worker threads for the P2P server. |

#### Identity & Sharding

| Flag | Required? | Description |
| --- | --- | --- |
| `--shardId [INT]` | **Yes** | The shard ID this node belongs to. |
| `--nodeId [INT]` | **Yes** | A unique identifier for the node within the experiment. |
| `--shardNum [INT]` | **Yes** | The total number of shards in the network. |
| `--shardSize [INT]` | **Yes** | The number of nodes per shard. |

#### Block & Data Configuration

| Flag | Required? | Description |
| --- | --- | --- |
| `--blockSize [INT]` | **Yes** | The size of the block in bytes. |
| `--symbolSize [INT]` | **Yes** | The size of a single data symbol in bytes. |
| `--propSize [INT]` | **Yes** | The size of `prop_tx_set` for each proposer block. |
| `--avaiSize [INT]` | **Yes** | The size of `avai_tx_set` for each availability block. |
| `--eReq [INT]` | **Yes** | Number of requested symbols for exclusive transaction blocks. |
| `--iReq [INT]` | **Yes** | Number of requested symbols for inclusive transaction blocks. |
| `--k [INT]` | **Yes** | Confirmation depth (security parameter). |

#### Mining Difficulty

Mining difficulties are provided as **32-byte hex strings**. Lower values indicate higher difficulty. For testing, use `ffff...` strings.

| Flag | Description |
| --- | --- |
| `--tDiff [HEX]` | Difficulty target for mining a **Transaction Block**. |
| `--pDiff [HEX]` | Difficulty target for mining a **Proposer Block**. |
| `--aDiff [HEX]` | Difficulty target for mining an **Exclusive Availability Block**. |
| `--iDiff [HEX]` | Difficulty target for mining an **Inclusive Availability Block**. |
| `--oDiff [HEX]` | Difficulty target for mining an **Ordering Block**. |

#### Experiment Logging

| Flag | Description |
| --- | --- |
| `-v` | Increases logging verbosity. |
| `--experNumber [INT]` | Sets the experiment number ID (used for logging/metrics). |
| `--experIter [INT]` | Sets the iteration number of the experiment. |

## Connecting Multiple Nodes

To run a second node that connects to the first one:

1. Change the ports (`--p2p`, `--api`) to avoid conflicts.
2. Change the `--nodeId` (and `--shardId` if testing cross-shard).
3. Use the `--connect` flag to point to the first node.

```bash
./target/release/Powchain optchain \
  --p2p 127.0.0.1:6001 \
  --api 127.0.0.1:7001 \
  --connect 127.0.0.1:6000 \
  --shardId 0 \
  --nodeId 1 \
  # ... (include all other required size/diff flags same as node 0)
```

## Quick Start

To quickly spin up a local test environment, we provide a helper script `quick_start.sh`. This script launches **4 nodes** configured as **2 shards with 2 nodes each**.

1. **Make the script executable:**
```bash
sudo chmod +x quick_start.sh
```
2. **Run the cluster:**

```bash
./quick_start.sh
```