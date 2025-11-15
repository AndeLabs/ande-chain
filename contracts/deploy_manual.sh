#!/bin/bash
set -e

RPC="https://rpc.ande.network"
DEPLOYER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

echo "==================================================="
echo "  DEPLOYMENT MANUAL - ANDECHAIN TIER 1"
echo "==================================================="
echo "RPC: $RPC"
echo "Chain ID: 6174"
echo "Deployer: $DEPLOYER"
echo ""

# Paso 1: Deploy ANDETokenDuality Implementation
echo "[1/6] Deploying ANDETokenDuality Implementation..."
ANDE_IMPL=$(forge create src/ANDETokenDuality.sol:ANDETokenDuality \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --json | jq -r '.deployedTo')
echo "  ✅ Implementation: $ANDE_IMPL"
echo ""

# Paso 2: Deploy ANDE Proxy
echo "[2/6] Deploying ANDE Proxy..."
INIT_DATA=$(cast abi-encode "initialize(address,address,address)" $DEPLOYER $DEPLOYER 0x00000000000000000000000000000000000000FD)
ANDE_PROXY=$(forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --constructor-args $ANDE_IMPL $INIT_DATA \
  --json | jq -r '.deployedTo')
echo "  ✅ Proxy: $ANDE_PROXY"
echo ""

# Paso 3: Deploy AndeNativeStaking Implementation
echo "[3/6] Deploying AndeNativeStaking Implementation..."
STAKING_IMPL=$(forge create src/staking/AndeNativeStaking.sol:AndeNativeStaking \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --json | jq -r '.deployedTo')
echo "  ✅ Implementation: $STAKING_IMPL"
echo ""

# Paso 4: Deploy Staking Proxy
echo "[4/6] Deploying Staking Proxy..."
STAKING_INIT=$(cast abi-encode "initialize(address,address,address,address,address)" $ANDE_PROXY $DEPLOYER $DEPLOYER $DEPLOYER $DEPLOYER)
STAKING_PROXY=$(forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --constructor-args $STAKING_IMPL $STAKING_INIT \
  --json | jq -r '.deployedTo')
echo "  ✅ Proxy: $STAKING_PROXY"
echo ""

# Paso 5: Deploy AndeSequencerRegistry Implementation
echo "[5/6] Deploying AndeSequencerRegistry Implementation..."
SEQ_IMPL=$(forge create src/sequencer/AndeSequencerRegistry.sol:AndeSequencerRegistry \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --json | jq -r '.deployedTo')
echo "  ✅ Implementation: $SEQ_IMPL"
echo ""

# Paso 6: Deploy Sequencer Proxy
echo "[6/6] Deploying Sequencer Proxy..."
SEQ_INIT=$(cast abi-encode "initialize(address,address)" $STAKING_PROXY $DEPLOYER)
SEQ_PROXY=$(forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC \
  --private-key $PRIVATE_KEY \
  --constructor-args $SEQ_IMPL $SEQ_INIT \
  --json | jq -r '.deployedTo')
echo "  ✅ Proxy: $SEQ_PROXY"
echo ""

echo "==================================================="
echo "  TIER 1 DEPLOYMENT COMPLETE"
echo "==================================================="
echo "ANDETokenDuality:"
echo "  Implementation: $ANDE_IMPL"
echo "  Proxy: $ANDE_PROXY"
echo ""
echo "AndeNativeStaking:"
echo "  Implementation: $STAKING_IMPL"
echo "  Proxy: $STAKING_PROXY"
echo ""
echo "AndeSequencerRegistry:"
echo "  Implementation: $SEQ_IMPL"
echo "  Proxy: $SEQ_PROXY"
echo ""
echo "==================================================="

# Guardar direcciones
cat > deployments/tier1-$(date +%s).json << JSON
{
  "network": "AndeChain Testnet",
  "chainId": 6174,
  "timestamp": $(date +%s),
  "deployer": "$DEPLOYER",
  "contracts": {
    "ANDETokenDuality": {
      "implementation": "$ANDE_IMPL",
      "proxy": "$ANDE_PROXY"
    },
    "AndeNativeStaking": {
      "implementation": "$STAKING_IMPL",
      "proxy": "$STAKING_PROXY"
    },
    "AndeSequencerRegistry": {
      "implementation": "$SEQ_IMPL",
      "proxy": "$SEQ_PROXY"
    }
  }
}
JSON

echo "✅ Deployment info saved to deployments/tier1-$(date +%s).json"
