# 🎯 Plan de Activación del Consenso Descentralizado - ANDE Chain

**Fecha:** 15 Noviembre 2025
**Objetivo:** Activar sistema completo de consenso descentralizado con sequencers rotativos
**Duración Estimada:** 8-12 semanas
**Prioridad:** ALTA - Fundamental para mainnet

---

## 📊 RESUMEN EJECUTIVO

### Estado Actual
- ✅ **Código 95% completo** - Contratos y arquitectura listos
- ✅ **Tests passing** - 304/315 tests (96.5%)
- ❌ **NO activado** - Corriendo con 1 solo sequencer
- ❌ **Integración Rust incompleta** - Consensus client stub

### Objetivo Final
```
┌─────────────────────────────────────────────────────────────┐
│         ANDE Chain - Consenso Descentralizado               │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Sequencer A (100 bloques) ──→ Rotate                       │
│       ↓                                                       │
│  Sequencer B (100 bloques) ──→ Rotate                       │
│       ↓                                                       │
│  Sequencer C (100 bloques) ──→ Rotate                       │
│       ↓                                                       │
│  Back to A...                                                │
│                                                               │
│  Features:                                                    │
│  ✅ Round-Robin automático cada 100 bloques                 │
│  ✅ Timeout detection (10 bloques) → Force rotation        │
│  ✅ Slashing por mala conducta (10-50% stake)              │
│  ✅ BFT finality (2/3+1 attestations)                       │
│  ✅ Force inclusion (anti-censorship)                       │
│  ✅ Emergency fallback sequencer                            │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🗓️ PLAN DE IMPLEMENTACIÓN

### **FASE 0: Preparación y Auditoría** (Semana 1-2)

#### ✅ Objetivos:
- Revisar todo el código de consenso
- Verificar seguridad de contratos
- Preparar entorno de desarrollo

#### 📋 Tareas:

**Semana 1: Auditoría de Código**
- [ ] Revisar `AndeConsensus.sol` línea por línea
- [ ] Revisar `AndeSequencerCoordinator.sol` línea por línea
- [ ] Verificar `AndeSequencerRegistry.sol`
- [ ] Run Slither security analysis
- [ ] Documentar findings críticos

**Semana 2: Setup de Infraestructura**
- [ ] Crear testnet local con 3+ nodos
- [ ] Configurar monitoreo (Prometheus + Grafana)
- [ ] Setup logging avanzado
- [ ] Preparar scripts de deployment

#### 📄 Entregables:
- `SECURITY_AUDIT_REPORT.md` - Findings de Slither
- `LOCAL_TESTNET_SETUP.md` - Guía de setup
- Scripts en `scripts/consensus/`

---

### **FASE 1: Deploy de Contratos (Solo Testnet)** (Semana 3-4)

#### ✅ Objetivos:
- Deployar contratos de consenso a testnet local
- Registrar 3 sequencers de prueba
- Verificar funcionamiento básico

#### 📋 Tareas:

**Semana 3: Deployment**

```solidity
// Script: scripts/DeployConsensus.s.sol

1. Deploy AndeToken (si no existe)
2. Deploy AndeNativeStaking
3. Deploy AndeSequencerRegistry
4. Deploy AndeSequencerCoordinator
5. Deploy AndeConsensus
6. Initialize todos los contratos
7. Grant roles necesarios
8. Verify en explorer
```

**Comandos:**
```bash
# Deploy a testnet local
cd contracts
forge script script/DeployConsensus.s.sol \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --verify

# Verificar deployment
forge script script/VerifyConsensus.s.sol \
  --rpc-url http://localhost:8545
```

**Semana 4: Registro de Sequencers**

```bash
# Registrar 3 sequencers de prueba
cast send $COORDINATOR \
  "registerSequencer(uint256,string,bytes32)" \
  100000000000000000000000 \
  "http://sequencer-a:8545" \
  0x1234...

# Verificar estado
cast call $COORDINATOR "getActiveSequencers()" --rpc-url local
```

#### 📄 Entregables:
- Contratos deployed en testnet local
- Direcciones en `deployments/testnet-local.json`
- 3 sequencers registrados y activos
- Tests de integración pasando

---

### **FASE 2: Integración Rust - Consensus Client** (Semana 5-7)

#### ✅ Objetivos:
- Implementar consensus client en Rust
- Integrar con ande-node
- Sincronizar validator set desde contratos

#### 📋 Tareas:

**Semana 5: Consensus Client Core**

```rust
// File: crates/ande-consensus/src/lib.rs

pub struct ConsensusEngine {
    config: ConsensusConfig,
    validator_set: ValidatorSet,
    current_proposer: Address,
    contract_client: Arc<ConsensusContractClient>,
}

impl ConsensusEngine {
    // 1. Sync validator set desde AndeConsensus.sol
    pub async fn sync_validator_set(&mut self) -> Result<()> {
        let validators = self.contract_client
            .get_active_validators()
            .await?;

        self.validator_set = ValidatorSet::new(validators);
        Ok(())
    }

    // 2. Get proposer actual desde contrato
    pub async fn get_current_proposer(&self) -> Result<Address> {
        self.contract_client
            .get_current_proposer()
            .await
    }

    // 3. Verificar si bloque es del proposer correcto
    pub fn validate_proposer(
        &self,
        block: &SealedBlock,
        expected_proposer: Address
    ) -> Result<()> {
        let block_proposer = block.recover_signer()?;

        if block_proposer != expected_proposer {
            return Err(ConsensusError::InvalidProposer {
                expected: expected_proposer,
                actual: block_proposer,
            });
        }

        Ok(())
    }

    // 4. Report invalid block al contrato
    pub async fn report_invalid_block(
        &self,
        sequencer: Address,
        block_number: u64,
        reason: &str
    ) -> Result<()> {
        self.contract_client
            .report_invalid_block(sequencer, block_number, reason)
            .await
    }
}
```

**Semana 6: Contract Client (ethers-rs)**

```rust
// File: crates/ande-consensus/src/contract_client.rs

use ethers::prelude::*;

abigen!(
    AndeConsensus,
    "./abi/AndeConsensus.json"
);

pub struct ConsensusContractClient {
    consensus: AndeConsensus<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl ConsensusContractClient {
    pub async fn get_active_validators(&self) -> Result<Vec<Address>> {
        let validators = self.consensus
            .get_active_validators()
            .call()
            .await?;

        Ok(validators)
    }

    pub async fn get_current_proposer(&self) -> Result<Address> {
        let proposer = self.consensus
            .get_current_proposer()
            .call()
            .await?;

        Ok(proposer)
    }

    pub async fn is_validator(&self, addr: Address) -> Result<bool> {
        let is_val = self.consensus
            .is_validator(addr)
            .call()
            .await?;

        Ok(is_val)
    }
}
```

**Semana 7: Integración con ande-node**

```rust
// File: crates/ande-node/src/consensus_integration.rs

pub async fn run_node_with_consensus(
    config: NodeConfig,
    consensus_config: ConsensusConfig,
) -> Result<()> {
    // 1. Initialize consensus engine
    let consensus_engine = ConsensusEngine::new(consensus_config).await?;

    // 2. Sync validator set
    consensus_engine.sync_validator_set().await?;

    // 3. Start block production loop
    loop {
        // Check si soy el proposer actual
        let current_proposer = consensus_engine.get_current_proposer().await?;

        if current_proposer == config.sequencer_address {
            // Soy el proposer, producir bloque
            produce_block(&consensus_engine).await?;
        } else {
            // No soy proposer, esperar y validar
            validate_incoming_blocks(&consensus_engine).await?;
        }

        // Check for rotation
        check_rotation(&consensus_engine).await?;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

#### 📄 Entregables:
- `crates/ande-consensus/` completamente implementado
- Tests unitarios de consensus engine
- Integration tests con contratos
- Documentación de arquitectura

---

### **FASE 3: Multi-Sequencer Setup** (Semana 8-9)

#### ✅ Objetivos:
- Configurar 3+ nodos sequencers
- Implementar rotación automática
- Verificar timeout y slashing

#### 📋 Tareas:

**Semana 8: Docker Compose Multi-Sequencer**

```yaml
# File: docker-compose-multi-sequencer.yml

services:
  # ============================================
  # Sequencer A (Foundation)
  # ============================================
  sequencer-a:
    image: ghcr.io/ande-labs/ande-node:latest
    ports:
      - "8545:8545"   # RPC
      - "8551:8551"   # Engine API
      - "7331:7331"   # Consensus P2P
    environment:
      - SEQUENCER_ADDRESS=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
      - SEQUENCER_PRIVATE_KEY=${SEQ_A_KEY}
      - CONSENSUS_CONTRACT=0x... # AndeConsensus address
      - COORDINATOR_CONTRACT=0x... # AndeSequencerCoordinator
      - ROTATION_BLOCKS=100
      - ENABLE_CONSENSUS=true
    volumes:
      - sequencer-a-data:/data
    command:
      - node
      - --sequencer-mode
      - --consensus-enabled
      - --rotation-aware

  # ============================================
  # Sequencer B
  # ============================================
  sequencer-b:
    image: ghcr.io/ande-labs/ande-node:latest
    ports:
      - "8546:8545"
      - "8552:8551"
      - "7332:7331"
    environment:
      - SEQUENCER_ADDRESS=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
      - SEQUENCER_PRIVATE_KEY=${SEQ_B_KEY}
      - CONSENSUS_CONTRACT=0x...
      - COORDINATOR_CONTRACT=0x...
      - ROTATION_BLOCKS=100
      - ENABLE_CONSENSUS=true
    volumes:
      - sequencer-b-data:/data

  # ============================================
  # Sequencer C
  # ============================================
  sequencer-c:
    image: ghcr.io/ande-labs/ande-node:latest
    ports:
      - "8547:8545"
      - "8553:8551"
      - "7333:7331"
    environment:
      - SEQUENCER_ADDRESS=0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
      - SEQUENCER_PRIVATE_KEY=${SEQ_C_KEY}
      - CONSENSUS_CONTRACT=0x...
      - COORDINATOR_CONTRACT=0x...
      - ROTATION_BLOCKS=100
      - ENABLE_CONSENSUS=true
    volumes:
      - sequencer-c-data:/data

  # ============================================
  # Celestia DA (shared)
  # ============================================
  celestia:
    # ... (mismo de docker-compose-quick.yml)

  # ============================================
  # Monitoring
  # ============================================
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./infra/config/prometheus-consensus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    volumes:
      - ./infra/grafana/dashboards/consensus-dashboard.json:/etc/grafana/dashboards/
```

**Semana 9: Testing de Rotación**

```bash
# 1. Start all sequencers
docker-compose -f docker-compose-multi-sequencer.yml up -d

# 2. Registrar sequencers en contratos
./scripts/consensus/register-sequencers.sh

# 3. Monitor rotación
watch -n 1 'cast call $COORDINATOR "getCurrentLeader()" --rpc-url local'

# 4. Simulate timeout
docker stop sequencer-a
# Esperar 10 bloques...
# Verificar que sequencer-b toma control automáticamente

# 5. Verificar slashing
cast call $COORDINATOR "getSequencerInfo(address)" $SEQ_A_ADDR --rpc-url local
# Debe mostrar stake reducido por timeout

# 6. Restart sequencer-a
docker start sequencer-a
# Verificar que re-entra en rotación
```

#### 📄 Entregables:
- `docker-compose-multi-sequencer.yml` funcionando
- 3 sequencers rotando correctamente
- Timeout detection verificado
- Slashing funcionando
- Dashboard de monitoreo

---

### **FASE 4: Activación de Features Avanzadas** (Semana 10-11)

#### ✅ Objetivos:
- Activar ANDE precompile
- Implementar force inclusion
- Integrar tBTC (opcional para esta fase)
- BFT finality

#### 📋 Tareas:

**Semana 10: ANDE Precompile Activation**

```rust
// File: crates/ande-node/src/precompile_activation.rs

use ande_evm::evm_config::{AndePrecompileConfig, AndePrecompileInspector};

pub fn activate_ande_precompile(evm_builder: EvmBuilder) -> Result<EvmBuilder> {
    // 1. Load config from environment
    let config = AndePrecompileConfig::from_env()?;

    // 2. Create inspector
    let inspector = AndePrecompileInspector::new(config);

    // 3. Add to EVM
    let evm = evm_builder
        .with_inspector(inspector)
        .build()?;

    Ok(evm)
}
```

**Environment Variables:**
```bash
# .env.precompile
ANDE_PRECOMPILE_ADDRESS=0x00000000000000000000000000000000000000fd
ANDE_TOKEN_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
ANDE_ALLOW_LIST=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266,0x70997970C51812dc3A010C7d01b50e0d17dc79C8
ANDE_PER_CALL_CAP=1000000000000000000000000  # 1M ANDE
ANDE_PER_BLOCK_CAP=10000000000000000000000000 # 10M ANDE
ANDE_STRICT_VALIDATION=true
```

**Semana 11: Force Inclusion & BFT Finality**

```rust
// File: crates/ande-consensus/src/force_inclusion.rs

pub struct ForceInclusionMonitor {
    coordinator_contract: AndeSequencerCoordinator,
    pending_requests: Vec<ForceInclusionRequest>,
}

impl ForceInclusionMonitor {
    pub async fn check_pending_requests(&mut self) -> Result<()> {
        let pending = self.coordinator_contract
            .get_pending_force_inclusions()
            .await?;

        for request in pending {
            if self.should_include(&request).await? {
                // Force include TX in next block
                self.include_transaction(&request).await?;
            }
        }

        Ok(())
    }
}
```

```rust
// File: crates/ande-consensus/src/bft_finality.rs

pub struct BFTFinalityChecker {
    consensus_contract: AndeConsensus,
}

impl BFTFinalityChecker {
    pub async fn is_block_finalized(&self, block_hash: H256) -> Result<bool> {
        let finalized = self.consensus_contract
            .is_block_finalized(block_hash)
            .await?;

        Ok(finalized)
    }

    pub async fn wait_for_finality(&self, block_hash: H256) -> Result<()> {
        loop {
            if self.is_block_finalized(block_hash).await? {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

#### 📄 Entregables:
- ANDE precompile activado y funcionando
- Force inclusion working
- BFT finality implementado
- Tests E2E completos

---

### **FASE 5: Testnet Público** (Semana 12-14)

#### ✅ Objetivos:
- Deploy a testnet público
- Permitir que otros registren sequencers
- Monitoring público
- Stress testing

#### 📋 Tareas:

**Semana 12: Testnet Deployment**

```bash
# Deploy contracts to testnet
forge script script/DeployConsensus.s.sol \
  --rpc-url https://testnet-rpc.ande.network \
  --broadcast \
  --verify \
  --etherscan-api-key $EXPLORER_API_KEY

# Publish addresses
echo "CONSENSUS_CONTRACT=$CONSENSUS_ADDR" > deployments/testnet.env
echo "COORDINATOR_CONTRACT=$COORDINATOR_ADDR" >> deployments/testnet.env
echo "REGISTRY_CONTRACT=$REGISTRY_ADDR" >> deployments/testnet.env
```

**Semana 13: Community Sequencer Program**

```markdown
# ANDE Testnet Sequencer Program

## Requirements:
- Minimum stake: 100,000 ANDE (testnet)
- Server: 4 CPU, 16GB RAM, 500GB SSD
- Network: Static IP, 100 Mbps uptime

## Rewards:
- Testnet ANDE rewards
- Early mainnet sequencer spots
- NFT badge

## How to Join:
1. Get testnet ANDE from faucet
2. Stake via contracts
3. Run ande-node
4. Monitor performance

## Documentation:
https://docs.ande.network/sequencer-program
```

**Semana 14: Stress Testing**

```bash
# Load testing script
./scripts/stress-test/flood-testnet.sh \
  --transactions 10000 \
  --duration 1h \
  --sequencers 5

# Monitor:
# - Block production rate
# - Rotation smoothness
# - Timeout handling
# - Slashing events
# - Force inclusion success rate
```

#### 📄 Entregables:
- Testnet público running con 5+ sequencers
- Documentation completa
- Stress test report
- Community feedback

---

### **FASE 6: Mainnet Preparation** (Semana 15-16)

#### ✅ Objetivos:
- Security audit externo
- Bug fixes de testnet
- Mainnet deployment plan
- Genesis sequencers selection

#### 📋 Tareas:

**Semana 15: Security Audit**

```markdown
# Audit Scope:
- AndeConsensus.sol (833 lines)
- AndeSequencerCoordinator.sol (780 lines)
- AndeSequencerRegistry.sol
- Rust consensus implementation

# Audit Firms (recomendados):
- Trail of Bits ($50-100K, 4 weeks)
- OpenZeppelin ($40-80K, 3 weeks)
- Consensys Diligence ($45-90K, 4 weeks)

# Critical Areas:
1. Slashing logic (double spend prevention)
2. Rotation mechanism (no dead-locks)
3. Emergency mode (admin powers)
4. Economic attacks (stake manipulation)
```

**Semana 16: Mainnet Genesis**

```solidity
// script/MainnetGenesis.s.sol

contract MainnetGenesis is Script {
    function run() external {
        vm.startBroadcast();

        // 1. Deploy contracts
        AndeConsensus consensus = new AndeConsensus();
        AndeSequencerCoordinator coordinator = new AndeSequencerCoordinator();

        // 2. Initialize with genesis sequencers
        address[] memory genesisSequencers = new address[](3);
        genesisSequencers[0] = FOUNDATION_SEQUENCER;
        genesisSequencers[1] = PARTNER_1_SEQUENCER;
        genesisSequencers[2] = PARTNER_2_SEQUENCER;

        coordinator.initialize(
            ADMIN,
            address(andeToken),
            EMERGENCY_SEQUENCER
        );

        // 3. Register genesis sequencers (no stake required)
        for (uint i = 0; i < genesisSequencers.length; i++) {
            coordinator.registerGenesisSequencer(
                genesisSequencers[i],
                endpoints[i],
                nodeIds[i]
            );
        }

        vm.stopBroadcast();
    }
}
```

#### 📄 Entregables:
- Security audit report
- All critical findings fixed
- Mainnet deployment scripts
- Genesis sequencers confirmed
- GO/NO-GO decision for mainnet

---

## 🎯 MILESTONES Y MÉTRICAS DE ÉXITO

### Milestone 1: Contratos Deployed (Semana 4)
✅ **Success Criteria:**
- [ ] Todos los contratos deployed a testnet local
- [ ] 3 sequencers registrados
- [ ] Rotación manual funcionando
- [ ] Tests E2E pasando

### Milestone 2: Rust Integration (Semana 7)
✅ **Success Criteria:**
- [ ] Consensus client conectado a contratos
- [ ] Sync de validator set funcionando
- [ ] Proposer verification working
- [ ] Invalid block reporting working

### Milestone 3: Auto-Rotation (Semana 9)
✅ **Success Criteria:**
- [ ] Rotación automática cada 100 bloques
- [ ] Timeout detection en <15 segundos
- [ ] Slashing aplicado correctamente
- [ ] Zero downtime durante rotación

### Milestone 4: Features Completos (Semana 11)
✅ **Success Criteria:**
- [ ] ANDE precompile activado
- [ ] Force inclusion funcionando
- [ ] BFT finality < 10 segundos
- [ ] All tests passing

### Milestone 5: Testnet Público (Semana 14)
✅ **Success Criteria:**
- [ ] 5+ sequencers externos running
- [ ] 1M+ transactions processed
- [ ] <0.1% block production failures
- [ ] Positive community feedback

### Milestone 6: Mainnet Ready (Semana 16)
✅ **Success Criteria:**
- [ ] Security audit passed
- [ ] Zero critical bugs
- [ ] Genesis sequencers committed
- [ ] Marketing ready

---

## 📊 RECURSOS NECESARIOS

### Team
- **1 Senior Rust Engineer** - Consensus implementation (8 semanas)
- **1 Senior Solidity Engineer** - Contracts review (2 semanas)
- **1 DevOps Engineer** - Infrastructure (4 semanas)
- **1 QA Engineer** - Testing (6 semanas)
- **1 Technical Writer** - Documentation (3 semanas)

### Costos Estimados
```
Desarrollo:
  - Senior Rust Engineer: $120K/year × (8/52) = $18,500
  - Senior Solidity: $110K/year × (2/52) = $4,200
  - DevOps: $100K/year × (4/52) = $7,700
  - QA: $80K/year × (6/52) = $9,200
  - Tech Writer: $70K/year × (3/52) = $4,000
  Subtotal: $43,600

Infraestructura:
  - Testnet nodes (3 servers): $500/month × 3 = $1,500
  - Monitoring (Grafana Cloud): $200/month
  - Domain & SSL: $100
  Subtotal: $1,800

Auditoría:
  - Security Audit (Trail of Bits): $70,000

TOTAL ESTIMADO: $115,400
```

### Timeline Risk Mitigation
- **Buffer de 20%** en cada fase
- **Weekly checkpoints** para detectar delays
- **Parallel work** donde sea posible
- **Fallback plan** si audit encuentra critical issues

---

## 🚨 RIESGOS Y MITIGACIONES

### Riesgo 1: Security Vulnerabilities
**Probabilidad:** Media
**Impacto:** Crítico
**Mitigación:**
- Auditoría externa obligatoria
- Bug bounty program ($100K pool)
- Testnet extensivo (4+ semanas)
- Emergency pause mechanism

### Riesgo 2: Performance Issues
**Probabilidad:** Media
**Impacto:** Alto
**Mitigación:**
- Benchmark cada feature
- Load testing continuo
- Optimize critical paths
- Lazy block intervals configurables

### Riesgo 3: Coordination Failures
**Probabilidad:** Baja
**Impacto:** Alto
**Mitigación:**
- Extensive logging
- P2P gossip protocol robusto
- Fallback a emergency sequencer
- Clear rotation signals

### Riesgo 4: Economic Attacks
**Probabilidad:** Media
**Impacto:** Alto
**Mitigación:**
- Minimum stake 100K ANDE
- Slashing penalties 10-50%
- Jail mechanism
- Community governance

---

## ✅ CHECKLIST FINAL ANTES DE MAINNET

### Contratos
- [ ] Security audit passed (zero critical, zero high)
- [ ] All tests passing (100% coverage core logic)
- [ ] Contracts verified on explorer
- [ ] Emergency pause tested
- [ ] Upgrade mechanism tested
- [ ] Economic parameters reviewed by tokenomics team

### Infraestructura
- [ ] 3+ genesis sequencers committed (legal agreements)
- [ ] Monitoring dashboard public
- [ ] Block explorer integrated
- [ ] RPC load balancers configured
- [ ] Backup/recovery procedures documented
- [ ] DDoS protection active

### Código Rust
- [ ] Consensus engine benchmarked (<100ms overhead)
- [ ] All integration tests passing
- [ ] P2P networking stable (tested 24h+)
- [ ] Memory leaks checked (valgrind)
- [ ] Fuzz testing passed (1M+ iterations)
- [ ] Docker images published & signed

### Documentación
- [ ] Sequencer setup guide
- [ ] API documentation
- [ ] Troubleshooting guide
- [ ] Economic whitepaper updated
- [ ] Security disclosure policy
- [ ] Community FAQ

### Comunicación
- [ ] Mainnet announcement blog post
- [ ] Twitter campaign scheduled
- [ ] Community AMA scheduled
- [ ] Partner announcements coordinated
- [ ] Press kit prepared

---

## 📚 REFERENCIAS

### Implementaciones Similares
- **CometBFT** - Cosmos consensus (weighted round-robin)
- **Arbitrum** - Sequencer rotation mechanism
- **Optimism** - Decentralized sequencing roadmap
- **Polygon zkEVM** - Proof aggregation

### Standards
- **ERC-4337** - Account abstraction (paymaster integration)
- **EIP-1559** - Base fee mechanism
- **BFT Consensus** - Byzantine fault tolerance

### Tools
- **Foundry** - Smart contract development
- **ethers-rs** - Rust Ethereum library
- **libp2p** - P2P networking
- **Prometheus** - Metrics collection
- **Grafana** - Dashboards

---

**Prepared by:** Claude (Anthropic)
**Date:** 15 Noviembre 2025
**Version:** 1.0
**Status:** DRAFT - READY FOR REVIEW

---

## 🚀 NEXT STEPS INMEDIATOS

1. **Aprobación de este plan** por el equipo
2. **Asignación de recursos** (team + budget)
3. **Kick-off meeting** - Semana próxima
4. **Start Fase 0** - Security audit y setup

**¿Proceder con implementación?** 🚀
