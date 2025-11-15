# Multi-Sequencer Deployment Status

**Date**: November 15, 2024
**Server**: 192.168.0.8
**Phase**: 2 - Pre-deployment Preparation
**Status**: 🟡 IN PROGRESS

---

## ✅ Completed Tasks

### 1. Planning & Documentation
- ✅ Created `MULTI_SEQUENCER_PLAN.md` (468 lines)
- ✅ Created `MONITORING_ACCESS.md` (456 lines)
- ✅ Created `DEPLOYMENT_STATUS.md` (374 lines)
- ✅ All documentation synchronized to GitHub (commit 2a9c814)

### 2. Server Preparation
- ✅ Code synchronized to server (commit 2a9c814)
- ✅ Foundry v1.4.4-stable installed on server
  - forge ✓
  - cast ✓
  - anvil ✓
  - chisel ✓

### 3. Wallet Generation
- ✅ 3 sequencer wallets generated on server
- ✅ `.env.testnet.local` created with credentials (NOT in git)

**Generated Wallets**:
```
Sequencer 1 (Primary):
  Address: 0xDA104C8Bf401F490599e1DA677Eba4D930aCF920

Sequencer 2 (Backup):
  Address: 0xc9a74e92dCF374ACA1010C0ECfF69c93335d4c10

Sequencer 3 (Backup):
  Address: 0x4Fa20fBfe0C757599d11939e1492b2A306d07064
```

---

## 🟡 Current Status

### Blockchain Health
- **Current Block**: 3564 (0xdec)
- **Block Production**: Active
- **Services**: All operational (ande-node, evolve, celestia, prometheus, grafana)
- **RPC Endpoint**: http://192.168.0.8:8545 ✅

### Configuration Files on Server
```
✅ .env.testnet (base configuration)
✅ .env.testnet.local (wallet credentials - secret)
✅ docker-compose-testnet.yml (3-sequencer orchestration)
✅ scripts/deploy-testnet.sh (automated deployment)
✅ MULTI_SEQUENCER_PLAN.md (deployment guide)
```

---

## 🔄 Next Steps

### Immediate Actions Required

#### 1. Fund Sequencer Wallets

Las wallets necesitan ser fondeadas con tokens de testnet para deployment:

**Option A: Celestia Mocha-4 (Recommended)**
```bash
# Para desplegar contracts en Celestia Mocha-4
# Visitar faucet: https://faucet.celestia-mocha.com/

Wallets a fondear:
- 0xDA104C8Bf401F490599e1DA677Eba4D930aCF920 (Sequencer 1 - deployer)
- 0xc9a74e92dCF374ACA1010C0ECfF69c93335d4c10 (Sequencer 2)
- 0x4Fa20fBfe0C757599d11939e1492b2A306d07064 (Sequencer 3)

Monto sugerido: 1 TIA por wallet (suficiente para deployment)
```

**Option B: Ethereum Sepolia**
```bash
# Visitar faucet: https://sepoliafaucet.com/

Mismo proceso, fondear las 3 wallets
Monto sugerido: 0.5 ETH por wallet
```

**Option C: Usar red propia (Desarrollo rápido)**
```bash
# Si queremos testear sin esperar faucets externos
# Podemos desplegar contracts en nuestra propia chain ANDE
# Las wallets ya tienen balance en genesis
```

#### 2. Verificar Balances

Una vez fondeadas las wallets, verificar en el servidor:

```bash
# SSH al servidor
ssh sator@192.168.0.8

cd ande-chain

# Verificar balance sequencer 1
~/.foundry/bin/cast balance 0xDA104C8Bf401F490599e1DA677Eba4D930aCF920 \
  --rpc-url https://rpc.celestia-mocha.com

# O verificar en nuestra propia chain
~/.foundry/bin/cast balance 0xDA104C8Bf401F490599e1DA677Eba4D930aCF920 \
  --rpc-url http://localhost:8545
```

#### 3. Deploy Consensus Contracts

Ejecutar script de deployment automático:

```bash
# En el servidor
cd ande-chain
./scripts/deploy-testnet.sh

# El script hará:
# 1. Verificar que wallets tienen fondos
# 2. Desplegar AndeConsensus.sol
# 3. Desplegar AndeSequencerCoordinator.sol
# 4. Registrar los 3 validadores
# 5. Actualizar .env.testnet.local con addresses
```

#### 4. Deploy 3-Sequencer Stack

```bash
# Detener stack actual (single node)
docker-compose -f docker-compose-quick.yml down

# Desplegar stack de 3 sequencers
docker-compose -f docker-compose-testnet.yml up -d

# Verificar todos los servicios
docker ps
```

#### 5. Verification

```bash
# Verificar RPC endpoints
curl http://localhost:8545 -X POST \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

curl http://localhost:8547 -X POST \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

curl http://localhost:8549 -X POST \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Verificar consensus state
~/.foundry/bin/cast call \
  --rpc-url http://localhost:8545 \
  $CONSENSUS_ADDRESS \
  "getActiveValidators()"
```

---

## 📋 Deployment Checklist

### Pre-Deployment
- [x] Code synchronized (GitHub + Server)
- [x] Foundry installed on server
- [x] Wallets generated (3 sequencers)
- [x] .env.testnet.local created
- [ ] Wallets funded with testnet tokens
- [ ] Balances verified

### Contract Deployment
- [ ] AndeConsensus deployed
- [ ] AndeSequencerCoordinator deployed
- [ ] 3 validators registered
- [ ] Contract addresses updated in .env
- [ ] Contracts verified on explorer

### Infrastructure Deployment
- [ ] Current single-node stopped
- [ ] Blockchain state backed up
- [ ] 3-sequencer stack deployed
- [ ] P2P network configured
- [ ] All containers healthy

### Verification
- [ ] All 3 RPC endpoints responding
- [ ] All nodes synced (within 5 blocks)
- [ ] Consensus rotation working
- [ ] Prometheus collecting metrics from 3 nodes
- [ ] Grafana dashboards showing all data
- [ ] HAProxy load balancing active

---

## 🎯 Success Criteria

### Deployment Success
- All 3 sequencer containers running
- All RPC endpoints responding (8545, 8547, 8549)
- P2P network established (30303-30305)
- Consensus contract deployed and active
- All validators registered

### Operational Success
- Blocks producing consistently (~2s target)
- Proposer rotating every 100 blocks
- All nodes synced (within 5 blocks)
- HAProxy distributing requests
- Prometheus collecting from all nodes

---

## ⚠️ Known Issues

### RPC Connectivity
- Celestia Mocha-4 public RPC (https://rpc.celestia-mocha.com/)
  experiencing connection timeouts from server
- **Workaround**: Use alternative RPC endpoints or deploy to our own chain

### Recommendations
1. Consider deploying contracts to our own ANDE chain first for testing
2. Then migrate to Celestia Mocha-4 once connectivity is stable
3. Or use alternative Celestia RPC endpoints

---

## 📁 Important Files

### On Server (192.168.0.8)
```
~/ande-chain/
├── .env.testnet                    # Base configuration
├── .env.testnet.local              # Wallet secrets (NOT in git)
├── docker-compose-testnet.yml      # 3-sequencer orchestration
├── scripts/
│   ├── deploy-testnet.sh           # Automated deployment
│   ├── backup-testnet.sh           # Backup automation
│   └── health-check-testnet.sh     # Health monitoring
├── MULTI_SEQUENCER_PLAN.md         # Deployment plan
├── MONITORING_ACCESS.md            # Monitoring guide
└── DEPLOYMENT_STATUS.md            # Current status
```

### Configuration Summary
- **Chain ID**: 6174
- **Network**: ANDE Testnet
- **Consensus**: CometBFT (2000+ lines implemented)
- **Data Availability**: Celestia Mocha-4
- **Sequencers**: 3 nodes
- **Rotation**: Every 100 blocks
- **Block Time**: ~2 seconds (target)

---

## 💡 Decision Point

**Decisión requerida**: ¿Qué red usar para deployment de contracts?

### Option 1: Celestia Mocha-4 (Plan original)
**Pros**:
- Red pública de testnet
- Simula producción
- DA layer real

**Cons**:
- RPC público con problemas de conectividad
- Requiere fondeo de wallets externas
- Tiempo de espera para faucets

### Option 2: ANDE Chain Local (Testing rápido)
**Pros**:
- Control total
- Sin dependencias externas
- Testing inmediato
- Wallets ya tienen fondos en genesis

**Cons**:
- No simula producción completa
- DA layer simulado

### Recommendation
1. **Fase 1**: Desplegar en ANDE Chain local para testing rápido
2. **Fase 2**: Migrar a Celestia Mocha-4 una vez probado

---

## 🚀 Ready for Execution

Todos los componentes están listos:
- ✅ Código sincronizado
- ✅ Herramientas instaladas
- ✅ Wallets generadas
- ✅ Configuración creada
- ✅ Scripts de deployment preparados
- ✅ Documentación completa

**Próxima acción**: Decidir red de deployment y fondear wallets

---

**Last Updated**: November 15, 2024 21:20 UTC
**Version**: Phase 2 - Pre-deployment
**Status**: 🟡 Waiting for wallet funding
