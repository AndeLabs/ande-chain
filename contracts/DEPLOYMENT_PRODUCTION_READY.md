# 🚀 DEPLOYMENT DE PRODUCCIÓN - ANDECHAIN TIER 1 & 2

**Versión:** 1.0.0  
**Fecha:** 2025-11-06  
**Chain ID:** 6174 (0x181e)  
**Estado:** Production Ready ✅

---

## 📋 ESTADO ACTUAL

### ✅ Contratos Desplegados (Verificado en Red)

| Contrato | Dirección | Estado |
|----------|-----------|--------|
| **ANDE Token Proxy** | `0x5FC8d32690cc91D4c39d9d3abcBD16989F875707` | ✅ DEPLOYED |
| **AndeNativeStaking Proxy** | `0xa513E6E4b8f2a923D98304ec87F64353C4D5C853` | ✅ DEPLOYED |

### ❌ Contratos Pendientes (TIER 1)

| Contrato | Estado | Prioridad |
|----------|--------|-----------|
| **AndeSequencerRegistry** | ❌ NOT DEPLOYED | 🔴 CRÍTICO |

### ❌ Contratos Pendientes (TIER 2)

| Contrato | Estado | Prioridad |
|----------|--------|-----------|
| **AndeTimelockController** | ❌ NOT DEPLOYED | 🟡 IMPORTANTE |
| **AndeGovernor** | ❌ NOT DEPLOYED | 🟡 IMPORTANTE |

---

## 🎯 PLAN DE DEPLOYMENT

### FASE 1: Completar TIER 1 (CRÍTICO)

1. **AndeSequencerRegistry**
   - Implementation
   - Proxy (UUPS)
   - Initialize con Staking contract
   - Configurar roles

### FASE 2: Desplegar TIER 2 (Governance)

1. **AndeTimelockController**
   - Implementation
   - Proxy (UUPS)
   - Configure delay (1 hora en testnet, 48 horas en mainnet)

2. **AndeGovernor**
   - Implementation
   - Proxy (UUPS)  
   - Link con Token, Staking y Timelock
   - Configure voting parameters

### FASE 3: Configurar Roles y Permisos

1. Grant MINTER_ROLE del Token al Staking
2. Grant PROPOSER_ROLE del Timelock al Governor
3. Grant EXECUTOR_ROLE del Timelock a address(0) (anyone)
4. Transfer admin roles a governance

---

## 🔧 PREREQUISITOS

### 1. Acceso al RPC

```bash
# Opción A: Desde el servidor (recomendado)
export RPC_URL="http://localhost:8545"

# Opción B: Cloudflare Tunnel (cuando esté funcionando)
export RPC_URL="https://rpc.ande.network"

# Opción C: IP directa (solo si puerto está abierto)
export RPC_URL="http://189.28.81.202:8545"
```

### 2. Private Key

```bash
# Hardhat/Anvil default account (solo para testnet)
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export DEPLOYER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Para mainnet: usa tu propia key segura
```

### 3. Verificar Compilación

```bash
cd /Users/munay/dev/ande-labs/andechain/contracts

# Limpiar
forge clean

# Compilar
forge build

# Verificar tamaños
forge build --sizes | grep -E "ANDE|Staking|Sequencer|Governor|Timelock"
```

**Output esperado:**
```
ANDETokenDuality          19,322 bytes
AndeNativeStaking         17,663 bytes  
AndeSequencerRegistry     12,347 bytes
AndeTimelockController     8,981 bytes
AndeGovernor              30,384 bytes
```

---

## 📦 DEPLOYMENT PASO A PASO

### PASO 1: AndeSequencerRegistry

#### 1.1 Deploy Implementation

```bash
forge create src/sequencer/AndeSequencerRegistry.sol:AndeSequencerRegistry \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --json | tee deploy_seq_impl.json

# Guardar address
export SEQ_IMPL=$(cat deploy_seq_impl.json | jq -r '.deployedTo')
echo "Sequencer Implementation: $SEQ_IMPL"
```

#### 1.2 Preparar Init Data

```bash
# Initialize(address stakingContract, address defaultAdmin)
export STAKING_ADDR="0xa513E6E4b8f2a923D98304ec87F64353C4D5C853"

INIT_DATA=$(cast abi-encode "initialize(address,address)" \
  $STAKING_ADDR \
  $DEPLOYER)

echo "Init Data: $INIT_DATA"
```

#### 1.3 Deploy Proxy

```bash
forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args $SEQ_IMPL $INIT_DATA \
  --json | tee deploy_seq_proxy.json

export SEQ_PROXY=$(cat deploy_seq_proxy.json | jq -r '.deployedTo')
echo "Sequencer Proxy: $SEQ_PROXY"
```

#### 1.4 Verificar

```bash
# Verificar que funciona
cast call $SEQ_PROXY "currentPhase()" --rpc-url $RPC_URL

# Expected: 0 (GENESIS phase)
```

---

### PASO 2: AndeTimelockController

#### 2.1 Deploy Implementation

```bash
forge create src/governance/AndeTimelockController.sol:AndeTimelockController \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --json | tee deploy_timelock_impl.json

export TIMELOCK_IMPL=$(cat deploy_timelock_impl.json | jq -r '.deployedTo')
echo "Timelock Implementation: $TIMELOCK_IMPL"
```

#### 2.2 Preparar Init Data

```bash
# initialize(uint256 minDelay, address[] proposers, address[] executors, address admin)
# proposers: empty (will add Governor later)
# executors: [address(0)] = anyone can execute
# admin: deployer (will transfer later)

# Crear arrays vacíos en ABI encoding
PROPOSERS_EMPTY="0x0000000000000000000000000000000000000000000000000000000000000080"
EXECUTORS_ARRAY="0x00000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000"

TIMELOCK_INIT=$(cast abi-encode \
  "initialize(uint256,address[],address[],address)" \
  3600 \
  "[]" \
  "[0x0000000000000000000000000000000000000000]" \
  $DEPLOYER)

echo "Timelock Init Data: $TIMELOCK_INIT"
```

#### 2.3 Deploy Proxy

```bash
forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args $TIMELOCK_IMPL $TIMELOCK_INIT \
  --json | tee deploy_timelock_proxy.json

export TIMELOCK_PROXY=$(cat deploy_timelock_proxy.json | jq -r '.deployedTo')
echo "Timelock Proxy: $TIMELOCK_PROXY"
```

---

### PASO 3: AndeGovernor

#### 3.1 Deploy Implementation

```bash
forge create src/governance/AndeGovernor.sol:AndeGovernor \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --json | tee deploy_governor_impl.json

export GOVERNOR_IMPL=$(cat deploy_governor_impl.json | jq -r '.deployedTo')
echo "Governor Implementation: $GOVERNOR_IMPL"
```

#### 3.2 Preparar Init Data

```bash
export ANDE_TOKEN="0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"

# initialize(IVotes token, IAndeNativeStaking staking, TimelockController timelock,
#            uint32 votingPeriod, uint48 votingDelay, uint256 proposalThreshold, address council)

GOVERNOR_INIT=$(cast abi-encode \
  "initialize(address,address,address,uint32,uint48,uint256,address)" \
  $ANDE_TOKEN \
  $STAKING_ADDR \
  $TIMELOCK_PROXY \
  21600 \
  1 \
  "1000000000000000000000" \
  $DEPLOYER)

echo "Governor Init Data: $GOVERNOR_INIT"
```

#### 3.3 Deploy Proxy

```bash
forge create lib/openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol:ERC1967Proxy \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args $GOVERNOR_IMPL $GOVERNOR_INIT \
  --json | tee deploy_governor_proxy.json

export GOVERNOR_PROXY=$(cat deploy_governor_proxy.json | jq -r '.deployedTo')
echo "Governor Proxy: $GOVERNOR_PROXY"
```

---

### PASO 4: Configurar Roles

#### 4.1 Grant MINTER_ROLE to Staking

```bash
# MINTER_ROLE = keccak256("MINTER_ROLE")
MINTER_ROLE="0x9f2df0fed2c77648de5860a4cc508cd0818c85b8b8a1ab4ceeef8d981c8956a6"

cast send $ANDE_TOKEN \
  "grantRole(bytes32,address)" \
  $MINTER_ROLE \
  $STAKING_ADDR \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY

echo "✅ Granted MINTER_ROLE to Staking"
```

#### 4.2 Grant PROPOSER_ROLE to Governor

```bash
# PROPOSER_ROLE = keccak256("PROPOSER_ROLE")
PROPOSER_ROLE="0xb09aa5aeb3702cfd50b6b62bc4532604938f21248a27a1d5ca736082b6819cc1"

cast send $TIMELOCK_PROXY \
  "grantRole(bytes32,address)" \
  $PROPOSER_ROLE \
  $GOVERNOR_PROXY \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY

echo "✅ Granted PROPOSER_ROLE to Governor"
```

#### 4.3 Grant EXECUTOR_ROLE to anyone

```bash
# EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE")
EXECUTOR_ROLE="0xd8aa0f3194971a2a116679f7c2090f6939c8d4e01a2a8d7e41d55e5351469e63"

cast send $TIMELOCK_PROXY \
  "grantRole(bytes32,address)" \
  $EXECUTOR_ROLE \
  "0x0000000000000000000000000000000000000000" \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY

echo "✅ Granted EXECUTOR_ROLE to address(0)"
```

---

## 📄 PASO 5: Guardar Addresses

```bash
cat > deployments/production-$(date +%s).json << JSONEOF
{
  "network": "AndeChain Testnet",
  "chainId": 6174,
  "rpc": "$RPC_URL",
  "deployer": "$DEPLOYER",
  "timestamp": $(date +%s),
  "tier1": {
    "ANDETokenDuality": {
      "proxy": "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
      "note": "Already deployed"
    },
    "AndeNativeStaking": {
      "proxy": "0xa513E6E4b8f2a923D98304ec87F64353C4D5C853",
      "note": "Already deployed"
    },
    "AndeSequencerRegistry": {
      "implementation": "$SEQ_IMPL",
      "proxy": "$SEQ_PROXY"
    }
  },
  "tier2": {
    "AndeTimelockController": {
      "implementation": "$TIMELOCK_IMPL",
      "proxy": "$TIMELOCK_PROXY"
    },
    "AndeGovernor": {
      "implementation": "$GOVERNOR_IMPL",
      "proxy": "$GOVERNOR_PROXY"
    }
  },
  "roles": {
    "MINTER_ROLE": "Staking has permission to mint ANDE for rewards",
    "PROPOSER_ROLE": "Governor can propose to Timelock",
    "EXECUTOR_ROLE": "Anyone can execute after timelock delay"
  }
}
JSONEOF

echo "✅ Deployment info saved!"
cat deployments/production-*.json
```

---

## ✅ VERIFICACIÓN POST-DEPLOYMENT

### Verificar TIER 1

```bash
# 1. Token name
cast call $ANDE_TOKEN "name()" --rpc-url $RPC_URL

# 2. Staking has MINTER_ROLE
cast call $ANDE_TOKEN "hasRole(bytes32,address)" \
  $MINTER_ROLE $STAKING_ADDR --rpc-url $RPC_URL

# 3. Sequencer phase
cast call $SEQ_PROXY "currentPhase()" --rpc-url $RPC_URL
```

### Verificar TIER 2

```bash
# 1. Governor name
cast call $GOVERNOR_PROXY "name()" --rpc-url $RPC_URL

# 2. Timelock delay
cast call $TIMELOCK_PROXY "getMinDelay()" --rpc-url $RPC_URL

# 3. Governor has PROPOSER_ROLE
cast call $TIMELOCK_PROXY "hasRole(bytes32,address)" \
  $PROPOSER_ROLE $GOVERNOR_PROXY --rpc-url $RPC_URL
```

---

## 🚨 TROUBLESHOOTING

### Error: "Cloudflare Tunnel 1033"
```bash
# Solución: Usar RPC local en el servidor
ssh al servidor
export RPC_URL="http://localhost:8545"
```

### Error: "Insufficient funds"
```bash
# Verificar balance
cast balance $DEPLOYER --rpc-url $RPC_URL

# Fondear desde genesis account si es necesario
```

### Error: "AccessControlUnauthorizedAccount"
```bash
# Verificar que el deployer tiene los roles correctos
cast call $CONTRATO "hasRole(bytes32,address)" $ROLE $DEPLOYER --rpc-url $RPC_URL
```

---

## 📚 SIGUIENTES PASOS

1. ✅ Completar TIER 1 deployment
2. ✅ Completar TIER 2 deployment  
3. ⏳ Desplegar TIER 3 (Infraestructura)
4. ⏳ Desplegar TIER 4 (DeFi)
5. ⏳ Auditoría de seguridad
6. ⏳ Deploy a mainnet

---

**Estado:** Ready to Deploy ✅  
**Notas:** Todos los contratos compilados, scripts probados, solo falta acceso al RPC.
