#!/bin/bash

# ==============================================================================
# OptChain Local Cluster Launcher
# Runs 4 nodes: 
#   - Shard 0: Node 0, Node 1
#   - Shard 1: Node 0, Node 1
#   - Automatically starts mining
#   - Interactive monitor to view chain state
# ==============================================================================

# --- Configuration ---
BIN="./target/release/powchain optchain"
# Difficulty: ffff... (Max target = easiest difficulty)
TX_DIFF="00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
PROP_DIFF="00afffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
EX_AVAI_DIFF="005fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
IN_AVAI_DIFF="001fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
ORDER_DIFF="007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"

# Common Args: 2 Shards, 2 Nodes/Shard, Block Size 1024
COMMON_ARGS="--shardNum 2 --shardSize 2 --experNumber 1 --experIter 1 --blockSize 1024 --symbolSize 32 --propSize 2 --avaiSize 2 --eReq 4 --iReq 4 --k 6 --tDiff $TX_DIFF --pDiff $PROP_DIFF --aDiff $EX_AVAI_DIFF --iDiff $IN_AVAI_DIFF --oDiff $ORDER_DIFF"

# --- Helper Functions ---

cleanup() {
    echo -e "\nShutting down cluster..."
    pkill -P $$
    exit 0
}
trap cleanup SIGINT

print_json() {
    # Try to pretty-print JSON using Python, otherwise plain cat
    if command -v python3 &> /dev/null; then
        python3 -m json.tool
    else
        cat
    fi
}

show_chains() {
    echo "======================================================================"
    echo "                       CLUSTER MINING STATUS                          "
    echo "======================================================================"
    
    # Iterate through all 4 nodes
    # Shard 0: Ports 7000, 7001
    # Shard 1: Ports 7002, 7003
    PORTS=(7000 7001 7002 7003)
    
    for PORT in "${PORTS[@]}"; do
        # Determine Shard/Node ID based on port for display
        if [ "$PORT" -eq 7000 ]; then INFO="Shard 0, Node 0 (Bootnode)"; fi
        if [ "$PORT" -eq 7001 ]; then INFO="Shard 0, Node 1"; fi
        if [ "$PORT" -eq 7002 ]; then INFO="Shard 1, Node 0"; fi
        if [ "$PORT" -eq 7003 ]; then INFO="Shard 1, Node 1"; fi

        echo "----------------------------------------------------------------------"
        echo "Node: $INFO (API: $PORT)"
        echo "----------------------------------------------------------------------"
        
        # 1. Proposer Chain
        echo "  [Proposer Chain] (Global view of proposer blocks)"
        curl -s "http://127.0.0.1:$PORT/blockchain/proposer-chain" | print_json | head -n 20
        echo "  ... (truncated)"
        echo ""

        # 2. Ordering Chain
        echo "  [Ordering Chain] (Global view of ordering blocks)"
        curl -s "http://127.0.0.1:$PORT/blockchain/ordering-chain" | print_json | head -n 20
        echo "  ... (truncated)"
        echo ""

        # 3. Availability Chain (Local Shard)
        echo "  [Availability Chain] (Local shard blocks)"
        curl -s "http://127.0.0.1:$PORT/blockchain/availability-chain" | print_json | head -n 20
        echo "  ... (truncated)"
        echo ""
    done
    echo "======================================================================"
}

# --- Main Execution ---

# 1. Build
echo "Building OptChain..."
cargo build --release || { echo "Build failed."; exit 1; }

# 2. Start Nodes
echo "Starting Local Cluster..."

# Shard 0
$BIN --p2p 127.0.0.1:6000 --api 127.0.0.1:7000 --shardId 0 --nodeId 0 $COMMON_ARGS &
sleep 2
$BIN --p2p 127.0.0.1:6001 --api 127.0.0.1:7001 --shardId 0 --nodeId 1 --connect 127.0.0.1:6000 $COMMON_ARGS &

# Shard 1
$BIN --p2p 127.0.0.1:6002 --api 127.0.0.1:7002 --shardId 1 --nodeId 0 --connect 127.0.0.1:6000 --connect 127.0.0.1:6001 $COMMON_ARGS &
$BIN --p2p 127.0.0.1:6003 --api 127.0.0.1:7003 --shardId 1 --nodeId 1 --connect 127.0.0.1:6000 --connect 127.0.0.1:6001 --connect 127.0.0.1:6002 $COMMON_ARGS &

# 3. Wait & Mining
echo "Nodes started. Waiting 5s for P2P stabilization..."
sleep 5

echo "Triggering mining on all nodes..."
for PORT in 7000 7001 7002 7003; do
    curl -s "http://127.0.0.1:$PORT/miner/start?lambda=0" > /dev/null
done
echo "Mining active."

# 4. Interactive Loop
while true; do
    echo -e "\n[RUNNING] Press [ENTER] to inspect chains, or Ctrl+C to stop."
    read -r
    show_chains
done