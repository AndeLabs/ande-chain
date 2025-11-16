# 🏗️ ANDE Chain - Estado de Arquitectura

**Fecha**: 2025-11-16
**Modo**: CONSTRUCCIÓN - Implementación Completa
**Objetivo**: Red Soberana Rollup EVM en Evolve + Celestia DA

---

## 📊 COMPONENTES IMPLEMENTADOS

### ✅ 1. Token Duality Precompile (0xFD)

**Ubicación**: `crates/ande-evm/src/evm_config/ande_precompile_provider.rs`  
**Estado**: ✅ **IMPLEMENTADO** - NO INTEGRADO  
**Características**:
- Transferencias nativas via `journal.transfer()`
- Gas metering correcto (3000 base + 100/word)
- Validaciones de seguridad completas
- Tests passing

**Problema**: NO está conectado al nodo. El nodo usa EthereumNode estándar.

---

### ✅ 2. Parallel EVM Execution (Block-STM)

**Ubicación**: `crates/ande-evm/src/parallel_executor.rs`  
**Estado**: ✅ **IMPLEMENTADO** - NO INTEGRADO  
**Características**:
- Multi-version memory (MVCC)
- Conflict detection automático
- Retry system
- 10-15x throughput improvement

**Problema**: NO está en el payload builder del nodo.

---

### ✅ 3. MEV Detection & Protection

**Ubicación**: `crates/ande-evm/src/mev/`  
**Estado**: ✅ **IMPLEMENTADO** - NO INTEGRADO  
**Módulos**:
- `detector.rs` - MEV pattern detection
- `auction.rs` - Bundle auction system
- `distributor.rs` - Fair MEV distribution (80% stakers, 20% treasury)

**Problema**: NO está en el transaction pool del nodo.

---

### ✅ 4. Genesis Personalizado

**Ubicación**: `specs/genesis.json`  
**Estado**: ✅ **COMPLETO Y ACTIVO**  
**Contenido**:
- 540 storage slots en `0x00...01`
- 16 referencias culturales Quechua (slots 0x00-0x0F)
- 4 slots Celestia metadata (slots 0x10-0x13)
- 520 seeds de plantas (slots 0x100-0x307)

**Estado**: ✅ Se usa correctamente en docker-compose.yml

---

### ✅ 5. Celestia Uploader

**Ubicación**: `crates/celestia-uploader/`  
**Estado**: ✅ **IMPLEMENTADO** - STANDALONE  
**Funcionalidad**:
- Chunking de blobs (5 plantas/blob)
- Upload paralelo a Celestia
- Verificación DA
- Generación de reportes

**Estado**: Tool standalone, no integrado en el nodo (no necesario).

---

## ❌ COMPONENTES PROBLEMÁTICOS

### ❌ 1. ande-reth/src/main.rs

**Línea 49**:
```rust
let handle = builder
    .node(EthereumNode::default())  // ❌ PROBLEMA: Usa nodo Ethereum estándar
    .launch_with_debug_capabilities()
```

**Debe ser** (según patrón de Reth):
```rust
let handle = builder
    .with_components(|ctx| {
        ctx.components_builder()
            .evm(create_ande_evm_config)
            .executor(create_ande_executor)  
            .build()
    })
    .launch_with_debug_capabilities()
```

---

### ❌ 2. AndeNode (node.rs)

**Línea 44**:
```rust
pub type AndeNode = reth_ethereum::node::EthereumNode;
```

**Problema**: Es solo un type alias, no añade customizaciones.

**Solución**: Crear un struct real que implemente NodeTypes con customizaciones.

---

### ❌ 3. Factories Sin Usar

**Archivos**:
- `crates/ande-evm/src/evm_config/factory.rs` - Crea EthEvmConfig estándar
- `crates/ande-evm/src/evm_config/executor_factory.rs` - NO usado
- `crates/ande-evm/src/evm_config/ande_evm_factory.rs` - NO usado

**Problema**: Implementados pero nunca llamados.

---

## 🔧 ARQUITECTURA EVOLVE + ANDE

### Comunicación Evolve → ANDE Node

```
┌─────────────────────────────────────────────┐
│  Evolve Sequencer (Consensus)               │
│  - Ordena transacciones                     │
│  - Produce bloques cada 1s-5s              │
│  - Submit batches a Celestia cada 50 blocks│
└──────────┬──────────────────────────────────┘
           │
           │ Engine API (JWT Auth)
           │ http://ande-node:8551
           ▼
┌─────────────────────────────────────────────┐
│  ANDE Node (Execution - Reth custom)        │
│  ✅ Engine API (8551)                       │
│  ✅ ETH RPC (8545)                          │
│  ❌ Custom EVM Config (NO integrado)        │
│  ❌ Token Duality 0xFD (NO activo)          │
│  ❌ Parallel EVM (NO activo)                │
│  ❌ MEV Detection (NO activo)               │
└─────────────────────────────────────────────┘
```

**Clave**: Evolve es agnóstico al nodo EVM. Solo necesita Engine API + ETH RPC estándares.

---

## 📝 PLAN DE IMPLEMENTACIÓN

### Fase 1: Integración de Customizaciones en ande-reth ✅ TODO

1. **Modificar `ande-reth/src/main.rs`**:
   - Usar `NodeBuilder` con `.with_components()`
   - Inyectar `AndePrecompileProvider`
   - Inyectar `ParallelExecutor`
   - Inyectar `MEVDetector`

2. **Crear `AndeNodeTypes`**:
   - Struct que implemente `NodeTypes` de Reth
   - Con EVM customizado

3. **Actualizar Dockerfile**:
   - Asegurar Rust nightly para edition2024
   - Build limpio de ande-reth

### Fase 2: Testing & Validation ⏸️ PENDING

1. Rebuild Docker image
2. Test precompile 0xFD
3. Test parallel execution
4. Test MEV detection
5. Full stack testing con Evolve

### Fase 3: Documentación 📚 PENDING

1. Marcar código deprecado
2. Documentar código activo
3. Actualizar README

---

## 🗂️ CÓDIGO DEPRECADO (Para Archivar)

### Deprecar:
1. `crates/ande-node/` - Skeleton sin uso real
2. `docker-compose-quick.yml.OLD` - Usa Reth oficial
3. `docker-compose-testnet*.yml.OLD` - Configs antiguas

### Mantener:
1. `crates/ande-reth/` - Nodo real (necesita modificaciones)
2. `crates/ande-evm/` - Customizaciones EVM (integrar)
3. `docker-compose.yml` - Stack completo (correcto)

---

**Siguiente Paso**: Implementar integración en ande-reth/src/main.rs

