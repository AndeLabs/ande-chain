# 🔄 ANDE Chain - Recreation Guide

> Si alguna vez necesitas recrear esta implementación desde cero, sigue esta guía EXACTA

**Versión**: 1.0.0  
**Fecha**: 2025-11-16  
**Tiempo estimado**: 6-8 horas (sabiendo qué hacer)

---

## 📋 Pre-requisitos

```bash
# 1. Rust nightly
rustup toolchain install nightly-2024-10-18
rustup default nightly-2024-10-18

# 2. Clonar Reth v1.8.2 como referencia
git clone https://github.com/paradigmxyz/reth.git /tmp/reth-reference
cd /tmp/reth-reference
git checkout v1.8.2

# 3. Tener acceso a ev-reth oficial de Evolve (para referencia)
```

---

## 📚 Documentos a Leer (EN ORDEN)

### Fase 1: Entender la Arquitectura

**1. Leer primero** (30 min):
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Secciones:
  - Resumen Ejecutivo
  - Arquitectura del Wrapper Pattern
  - Componentes Clave

**2. Código de referencia** (1 hora):
- `/tmp/reth-reference/crates/ethereum/node/src/node.rs`
  - Buscar: `impl NodeTypes for EthereumNode`
  - Buscar: `impl Node<N> for EthereumNode`
  - Buscar: `fn components<Node>()`
  
- `/tmp/reth-reference/crates/ethereum/node/src/node.rs`
  - Buscar: `impl ExecutorBuilder<Node> for EthereumExecutorBuilder`
  - Buscar: `impl ConsensusBuilder<Node> for EthereumConsensusBuilder`

### Fase 2: Entender Problemas Comunes

**3. Leer** (30 min):
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Sección: "Problemas Resueltos"
  - Leer TODOS los 5 problemas
  - Entender soluciones
  - Memorizar puntos críticos

---

## 🔨 Implementación Paso a Paso

### PASO 1: Setup del Proyecto (15 min)

```bash
# 1. Crear estructura base
mkdir -p crates/ande-reth/src
mkdir -p crates/ande-evm/src/evm_config

# 2. Copiar Cargo.toml workspace
# (Asegurar que tiene las dependencias correctas)
```

**Verificar en** `Cargo.toml` (raíz):
```toml
[workspace.dependencies]
alloy-evm = { version = "0.21.0", default-features = false }
reth-node-api = { git = "https://github.com/paradigmxyz/reth.git", tag = "v1.8.2" }
reth-ethereum-consensus = { git = "https://github.com/paradigmxyz/reth.git", tag = "v1.8.2" }
# ... etc

[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2024"
rust-version = "1.88"
```

---

### PASO 2: Implementar AndeEvmFactory (1 hora)

**Archivo**: `crates/ande-evm/src/evm_config/ande_evm_factory.rs`

**Referencia**: 
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Sección "AndeEvmFactory"
- Código actual: `crates/ande-evm/src/evm_config/ande_evm_factory.rs`

**Puntos críticos**:

1. ⚠️ Usar **wrapper pattern** con generic:
```rust
pub struct AndeEvmFactory<F = EthEvmFactory> {
    inner: F,  // ← Este es el wrapper
    spec_id: SpecId,
}
```

2. ⚠️ Import correcto:
```rust
use alloy_evm::EthEvmFactory;  // ← NO from reth_ethereum::evm
```

3. ⚠️ Implementar `EvmFactory` trait:
```rust
impl EvmFactory for AndeEvmFactory<EthEvmFactory> {
    type Evm<DB: Database, I: Inspector<...>> = EthEvm<DB, I, PrecompilesMap>;
    // ... otros associated types
    
    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> ... {
        // Por ahora, delegar al inner
        self.inner.create_evm(db, input)
    }
}
```

**Verificar**:
```bash
cargo check -p ande-evm
```

---

### PASO 3: Implementar AndeExecutorBuilder (1 hora)

**Archivo**: `crates/ande-reth/src/executor.rs`

**Referencia**: 
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Sección "AndeExecutorBuilder"
- Código de Reth: `/tmp/reth-reference/crates/ethereum/node/src/node.rs`

**⚠️ PUNTO MÁS CRÍTICO**: Type signature

```rust
impl<Types, Node> ExecutorBuilder<Node> for AndeExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    // ⚠️⚠️⚠️ CRÍTICO: Usar Types::ChainSpec
    type EVM = EthEvmConfig<Types::ChainSpec, AndeEvmFactory>;
    //                      ^^^^^^^^^^^^^^^^
    //                      NO usar ChainSpec directo!

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let inner_factory = EthEvmFactory::default();
        let ande_factory = AndeEvmFactory::new(inner_factory, SpecId::CANCUN);
        
        let evm_config = EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec().clone(),
            ande_factory,
        );
        
        Ok(evm_config)
    }
}
```

**Verificar**:
```bash
cargo check -p ande-reth
```

**Si falla con error E0308** (type mismatch):
- ✅ Estás usando `ChainSpec` directo → Cambiar a `Types::ChainSpec`

---

### PASO 4: Implementar AndeConsensusBuilder (1 hora)

**Archivo**: `crates/ande-reth/src/consensus.rs`

**Referencia**:
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Sección "AndeConsensusBuilder"
- Código de Reth: `/tmp/reth-reference/crates/ethereum/node/src/node.rs` (línea ~545)

**⚠️ PUNTOS CRÍTICOS**:

1. Imports correctos:
```rust
use reth_chainspec::EthChainSpec;  // ← NO solo ChainSpec
use reth_ethereum_consensus::EthBeaconConsensus;
use reth_ethereum_primitives::EthPrimitives;
use reth_node_api::{FullNodeTypes, NodeTypes};
```

2. Trait bounds precisos:
```rust
impl<Node> ConsensusBuilder<Node> for AndeConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: EthChainSpec + EthereumHardforks,  // ← Ambos
            Primitives = EthPrimitives,
        >,
    >,
{
```

3. ⚠️⚠️⚠️ **MUY CRÍTICO**: Type de retorno con Arc:
```rust
    type Consensus = Arc<EthBeaconConsensus<<Node::Types as NodeTypes>::ChainSpec>>;
    //               ^^^                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //               Arc obligatorio        Usar ChainSpec del Node
```

4. Build method:
```rust
    async fn build_consensus(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(EthBeaconConsensus::new(ctx.chain_spec())))
        // ^^^ Arc::new es necesario
    }
```

**Verificar**:
```bash
cargo check -p ande-reth
```

**Si falla con error E0277** (trait bound not satisfied):
- ✅ Verificar que `ChainSpec: EthChainSpec + EthereumHardforks`
- ✅ Verificar que retornas `Arc<...>`
- ✅ Verificar que usas `<Node::Types as NodeTypes>::ChainSpec`

---

### PASO 5: Implementar AndeNode (1 hora)

**Archivo**: `crates/ande-reth/src/node.rs`

**Referencia**:
- `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Sección "AndeNode"
- Código actual: `crates/ande-reth/src/node.rs`

**Estructura**:

1. NodeTypes impl:
```rust
impl NodeTypes for AndeNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}
```

2. Function components con constraints:
```rust
pub fn components<N>() -> ComponentsBuilder<...>
where
    N: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: Hardforks + EthereumHardforks + EthExecutorSpec,
            Primitives = EthPrimitives,
        >,
    >,
    <N::Types as NodeTypes>::Payload: PayloadTypes<...>,
{
    ComponentsBuilder::default()
        .node_types::<N>()
        .pool(EthereumPoolBuilder::default())
        .executor(AndeExecutorBuilder::default())
        .payload(BasicPayloadServiceBuilder::default())
        .network(EthereumNetworkBuilder::default())
        .consensus(AndeConsensusBuilder::default())
}
```

3. Node impl:
```rust
impl<N> Node<N> for AndeNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<EthereumPayloadBuilder>,
        EthereumNetworkBuilder,
        AndeExecutorBuilder,
        AndeConsensusBuilder,
    >;
    
    type AddOns = AndeAddOns<NodeAdapter<N>>;
    
    fn components_builder(&self) -> Self::ComponentsBuilder {
        Self::components()
    }
}
```

**Verificar**:
```bash
cargo check -p ande-reth
```

---

### PASO 6: Actualizar main.rs (15 min)

**Archivo**: `crates/ande-reth/src/main.rs`

**⚠️ PUNTO CRÍTICO**: Declarar TODOS los módulos

```rust
// Import ANDE custom components
mod node;
mod executor;
mod consensus;  // ← ¡NO OLVIDAR ESTE!

use node::AndeNode;
```

**⚠️ PUNTO CRÍTICO**: Unsafe block para set_var

```rust
fn main() {
    // Enable backtraces
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe {  // ← Rust 2024 requires unsafe
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }
    
    // ... rest of main
    
    // Use AndeNode
    builder.node(AndeNode::new()).launch().await?;
}
```

**Verificar**:
```bash
cargo check -p ande-reth
```

---

### PASO 7: Fix ConsensusIntegration (30 min)

**Archivo**: `crates/ande-node/src/consensus_integration.rs`

**⚠️ PUNTO CRÍTICO**: Usar Option, NO std::mem::zeroed()

```rust
pub struct ConsensusIntegration {
    engine: Option<Arc<RwLock<ConsensusEngine>>>,  // ← Option
    enabled: bool,
    sequencer_address: Address,
}

impl ConsensusIntegration {
    pub fn disabled() -> Self {
        Self {
            engine: None,  // ← NO usar std::mem::zeroed()
            enabled: false,
            sequencer_address: Address::ZERO,
        }
    }
    
    // Actualizar todos los métodos para manejar Option
    pub async fn start(&self) -> Result<()> {
        if let Some(ref engine) = self.engine {
            let engine = engine.read().await;
            engine.start().await?;
        }
        Ok(())
    }
    // ... etc
}
```

**Verificar**:
```bash
cargo check -p ande-node
```

---

### PASO 8: Actualizar lib.rs (5 min)

**Archivo**: `crates/ande-reth/src/lib.rs`

```rust
/// ANDE custom node implementation
pub mod node;

/// ANDE custom executor builder
pub mod executor;

/// ANDE custom consensus builder
pub mod consensus;

/// Re-exports
pub use node::AndeNode;
pub use executor::AndeExecutorBuilder;
pub use consensus::AndeConsensusBuilder;
```

---

### PASO 9: Verificar Dependencies (15 min)

**Archivo**: `crates/ande-reth/Cargo.toml`

Asegurar que tiene:
```toml
[dependencies]
ande-evm = { workspace = true }
reth-ethereum-consensus = { workspace = true }
reth-chainspec = { workspace = true }
alloy-evm = { workspace = true }
# ... etc
```

---

### PASO 10: Compilación Final (10 min)

```bash
# Limpiar
cargo clean

# Compilar
cargo build --release

# Si hay errores, ver troubleshooting en
# docs/CUSTOM_RETH_IMPLEMENTATION.md
```

**Errores comunes y fixes**:
- E0308 (type mismatch) → Usar `Types::ChainSpec`
- E0277 (trait bound) → Verificar `where` clauses
- E0432 (unresolved import) → Declarar módulo en `main.rs`
- E0133 (unsafe) → Agregar `unsafe {  }` block

---

### PASO 11: Testing (30 min)

```bash
# 1. Verificar que compila
cargo build --release

# 2. Verificar binario
ls -lh target/release/ande-node

# 3. Test rápido (debe fallar limpiamente, no panic)
timeout 3 ./target/release/ande-node --version || true

# 4. Verificar que no hay panic de ConsensusEngine
# Debe ver logs de inicialización, NO panic
```

---

## ✅ Checklist de Verificación

Antes de considerar completo:

### Compilación
- [ ] `cargo check` sin errores
- [ ] `cargo build --release` exitoso
- [ ] 0 errores (warnings ok)
- [ ] Binario generado: `target/release/ande-node`

### Código
- [ ] AndeEvmFactory usa wrapper pattern con `F` generic
- [ ] AndeExecutorBuilder usa `Types::ChainSpec` (no `ChainSpec`)
- [ ] AndeConsensusBuilder retorna `Arc<EthBeaconConsensus<...>>`
- [ ] Todos los módulos declarados en `main.rs`
- [ ] ConsensusIntegration usa `Option`, no `zeroed()`
- [ ] `unsafe` block en `set_var`

### Imports
- [ ] `alloy_evm::EthEvmFactory` (no de reth_ethereum)
- [ ] `reth_chainspec::EthChainSpec` (no solo ChainSpec)
- [ ] Todos los imports resuelven correctamente

### Runtime
- [ ] Binario ejecuta sin panic
- [ ] Logs de inicialización aparecen
- [ ] Consensus integration en modo disabled funciona

---

## 🎯 Tiempo Total Esperado

| Fase | Tiempo |
|------|--------|
| Lectura y entendimiento | 2 horas |
| Setup proyecto | 15 min |
| AndeEvmFactory | 1 hora |
| AndeExecutorBuilder | 1 hora |
| AndeConsensusBuilder | 1 hora |
| AndeNode | 1 hora |
| main.rs y lib.rs | 20 min |
| ConsensusIntegration fix | 30 min |
| Compilación y debugging | 1 hora |
| Testing | 30 min |
| **Total** | **~8 horas** |

---

## 🆘 Si Te Atascas

### Paso 1: Identificar Error

```bash
cargo build --release 2>&1 | grep '^error'
```

### Paso 2: Buscar en Documentación

Revisar `docs/CUSTOM_RETH_IMPLEMENTATION.md` sección "Problemas Resueltos"

### Paso 3: Comparar con Código Actual

```bash
diff tu_archivo.rs crates/ande-reth/src/archivo.rs
```

### Paso 4: Verificar Imports

```bash
grep "^use" crates/ande-reth/src/archivo.rs
```

### Paso 5: Último Recurso

Copiar el archivo completo del código actual y entender línea por línea.

---

## 📚 Referencias Durante Implementación

**Siempre tener abiertas**:
1. `docs/CUSTOM_RETH_IMPLEMENTATION.md` - Para arquitectura
2. `/tmp/reth-reference/crates/ethereum/node/src/node.rs` - Para patterns oficiales
3. `docs/DEVELOPMENT_GUIDE.md` - Para troubleshooting
4. `QUICK_REFERENCE.md` - Para comandos

**Código actual como referencia**:
- `crates/ande-reth/src/*.rs` - Implementación correcta
- `crates/ande-evm/src/evm_config/ande_evm_factory.rs` - Wrapper pattern

---

## 🎉 Éxito Confirmado

Sabrás que tuviste éxito cuando:

```bash
$ cargo build --release
   Compiling ande-reth v0.1.0
   Compiling ande-node v0.1.0
    Finished `release` profile [optimized] target(s) in 1.5m

$ ./target/release/ande-node --version
# Ve logs de inicialización
# NO ve panic
# Proceso termina limpiamente
```

---

## 📝 Notas Finales

1. **No saltarse pasos**: Cada paso es crítico
2. **Leer errores completos**: El compilador de Rust es muy descriptivo
3. **Comparar con referencia**: Si algo no funciona, comparar con Reth oficial
4. **Documentar cambios**: Si haces algo diferente, documentar por qué

---

**Creado**: 2025-11-16  
**Versión**: 1.0.0  
**Tested**: ✅ Funcionó para implementación original

**Con esta guía y la documentación completa, recrear ANDE Chain custom Reth debería tomar ~8 horas en lugar de días de debugging.**

🚀 **Good luck!**
