# ANDE Chain - Estado de Características y Roadmap

## 📊 Estado Actual (v1.0 - Noviembre 2025)

### ✅ ACTIVO Y FUNCIONAL

#### 1. **Token Duality Precompile** (0xFD)
- **Estado**: ✅ **Completamente implementado y activo**
- **Implementación**: `AndeTokenDualityPrecompile` con `EvmInternals`
- **Ubicación**: `crates/ande-evm/src/evm_config/ande_token_duality.rs`
- **Características**:
  - ✅ Acceso directo al estado EVM via `EvmInternals`
  - ✅ Allowlist basado en storage (slots 0x00-0xFF)
  - ✅ Per-call caps configurables
  - ✅ Per-block caps con tracking
  - ✅ Admin authorization
  - ✅ Manual ABI encoding/decoding
  - ✅ Comprehensive logging y error handling

**Integración**:
```rust
// AndeEvmFactory → AndeExecutorBuilder → AndeNode
AndeEvmFactory::new(spec_id) 
  → crea EVM con precompile en 0xFD
  → usado por AndeExecutorBuilder
  → configurado en AndeNode::components()
```

**Funciones disponibles**:
- `transfer(address to, uint256 amount)` - Selector: 0xbeabacc8
- `balanceOf(address account)` - Selector: 0x70a08231
- `addToAllowlist(address account)` - Selector: 0xe43252d7
- `removeFromAllowlist(address account)` - Selector: 0xb0f10011
- `isAllowlisted(address account)` - Selector: 0x91d14854
- `transferredThisBlock()` - Selector: 0x9507d39a

**Configuración** (variables de entorno):
- `ANDE_ADMIN_ADDRESS` - Admin address para allowlist
- `ANDE_PER_CALL_CAP` - Cap por llamada (default: sin límite)
- `ANDE_PER_BLOCK_CAP` - Cap por bloque (default: sin límite)

#### 2. **Custom Reth Node (AndeNode)**
- **Estado**: ✅ **Activo y compilando**
- **Implementación**: Fork completo de Reth v1.8.2
- **Ubicación**: `crates/ande-reth/src/`
- **Componentes**:
  - ✅ `AndeNode` - Custom node type
  - ✅ `AndeExecutorBuilder` - Custom EVM executor
  - ✅ `AndeEvmFactory` - Wrapper con precompiles
  - ✅ `AndeConsensusBuilder` - Consensus builder

#### 3. **Multi-Sequencer Consensus (CometBFT)**
- **Estado**: ✅ **Código implementado**
- **Ubicación**: `crates/ande-consensus/`
- **Componentes**:
  - ✅ `ConsensusEngine` - Motor de consenso
  - ✅ `ValidatorSet` - Gestión de validadores
  - ✅ `ContractClient` - Cliente para staking contract
  - ✅ `ConsensusMetrics` - Métricas y monitoreo
- **Estado**: ⚠️ **NO INTEGRADO** en el executor principal aún
- **Próximo paso**: Integrar en `AndeExecutorBuilder`

---

## 🚧 IMPLEMENTADO PERO NO ACTIVO

### 4. **Parallel EVM Execution (Block-STM)**
- **Estado**: 🟡 **Código base implementado, NO activo**
- **Ubicación**: `crates/ande-evm/src/parallel_executor.rs`
- **Implementación actual**:
  - ✅ `ParallelExecutor` struct
  - ✅ `MultiVersionMemory` - MVCC para state
  - ✅ `DependencyTracker` - Detección de conflictos
  - ✅ `optimal_worker_count()` - CPU detection
- **Limitaciones**:
  - ❌ NO integrado con AndeExecutorBuilder
  - ❌ Necesita adaptación a Reth v1.8.2 EVM APIs
  - ❌ Falta lazy evaluation para transfers/gas
  - ❌ Sin soporte para ERC-20 lazy updates

**Referencia de implementación**: [risechain/pevm](https://github.com/risechain/pevm)
- pevm logra 22x speedup con 32 CPUs
- 30 Gigagas/s throughput
- Usa Block-STM + lazy evaluation
- Rust implementation (minimal overhead)

**Plan de activación**:
1. Estudiar pevm architecture
2. Adaptar lazy evaluation para gas payments
3. Implementar ERC-20 transfer mocking
4. Integrar con AndeExecutorBuilder
5. Testing exhaustivo con mainnet blocks

### 5. **MEV Detection & Distribution**
- **Estado**: 🟡 **Código base implementado, NO activo**
- **Ubicación**: `crates/ande-evm/src/mev/`
- **Componentes**:
  - ✅ `MevDetector` - Detección de oportunidades MEV
  - ✅ `MevAuctionClient` - Cliente para subastas
  - ✅ `MevDistributorClient` - Distribución de revenue
  - ✅ `MevMetrics` - Métricas
- **Limitaciones**:
  - ❌ NO integrado con executor
  - ❌ Sin conexión a sistema de subastas
  - ❌ Falta implementación de fair distribution (80% stakers, 20% treasury)

**Plan de activación**:
1. Implementar MEV auction smart contract
2. Integrar MevDetector en block building
3. Conectar con validator rewards
4. Deploy distributor contract
5. Testing con bundles reales

---

## 📋 Roadmap de Implementación

### Fase 1: Producción Básica (COMPLETADA ✅)
- [x] Token Duality Precompile funcional
- [x] AndeNode compilando
- [x] Custom executor builder
- [x] EVM factory con precompiles

### Fase 2: Consensus Integration (EN PROGRESO 🚧)
**Prioridad: ALTA**
- [ ] Integrar ConsensusEngine en AndeExecutorBuilder
- [ ] Conectar ValidatorSet con block production
- [ ] Deploy staking contracts
- [ ] Testing multi-validator
- **Estimado**: 1-2 semanas

### Fase 3: Parallel Execution (PENDIENTE ⏳)
**Prioridad: MEDIA-ALTA**
- [ ] Estudiar pevm implementation en detalle
- [ ] Implementar lazy gas payment updates
- [ ] Implementar lazy ETH transfer mocking
- [ ] Adaptar a Reth v1.8.2 APIs
- [ ] Integrar con AndeExecutorBuilder
- [ ] Benchmarking con mainnet blocks
- **Estimado**: 3-4 semanas

### Fase 4: MEV Protection (PENDIENTE ⏳)
**Prioridad**: MEDIA
- [ ] Deploy MEV auction contract
- [ ] Integrar MevDetector
- [ ] Implementar fair distribution
- [ ] Testing con bundles
- **Estimado**: 2-3 semanas

### Fase 5: Optimizaciones Avanzadas (FUTURO 🔮)
- [ ] Parallel sparse trie (state root computation)
- [ ] Shred broadcasting para pending states
- [ ] Resource-aware scheduler
- [ ] Advanced ERC-20 lazy updates
- **Estimado**: 4-6 semanas

---

## 🔍 Referencias Técnicas

### Block-STM y Parallel EVM
1. **Aptos Block-STM Paper**: https://arxiv.org/abs/2203.06871
2. **RISE pevm**: https://github.com/risechain/pevm
   - Rust implementation
   - 22x speedup demostrado
   - Lazy evaluation pattern
3. **Reth Parallel EVM Roadmap**: https://www.paradigm.xyz/2024/04/reth-perf
   - Target: 1 Gigagas/s
   - OPStack integration
4. **Sei Research**: 64.85% de Ethereum txs son paralelizables

### MEV y Fair Distribution
1. **Flashbots**: MEV auction mechanism
2. **MEV-Boost**: Proposer-Builder separation
3. **Fair MEV Distribution**: Staker rewards models

### Consensus (CometBFT)
1. **CometBFT Docs**: https://docs.cometbft.com/
2. **Cosmos SDK Integration**: Validator set management
3. **ABCI++ Spec**: Application Blockchain Interface

---

## 📊 Métricas de Éxito

### Token Duality Precompile ✅
- [x] Compila sin errores
- [x] Tests unitarios pasando
- [ ] E2E tests con Foundry
- [ ] Gas benchmarks
- [ ] Auditoría de seguridad

### Parallel Execution (Targets)
- [ ] 5x speedup en blocks promedio (conservador)
- [ ] 15x speedup en blocks independientes (ej: Uniswap)
- [ ] <100ms latency para tx #1000 en block
- [ ] 10+ Gigagas/s throughput

### MEV Protection (Targets)
- [ ] 80% revenue a stakers
- [ ] 20% a treasury
- [ ] <5% extracción por searchers
- [ ] 100% bundles válidos procesados

### Consensus (Targets)
- [ ] <1s block time
- [ ] Byzantine fault tolerance (BFT)
- [ ] 100+ validator support
- [ ] <10% downtime anual

---

## 🚀 Próximos Pasos Inmediatos

### Esta semana:
1. ✅ Completar Token Duality Precompile
2. ✅ Compilar y sincronizar código
3. [ ] **Rebuild Docker y testing E2E**
4. [ ] Documentar estado actual

### Próxima semana:
1. [ ] Integrar ConsensusEngine
2. [ ] Estudiar pevm en profundidad
3. [ ] Plan detallado para Parallel EVM
4. [ ] Setup testing infrastructure

---

**Última actualización**: 2025-11-16
**Versión**: v1.0.0
**Autor**: ANDE Labs Team
