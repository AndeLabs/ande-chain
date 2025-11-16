# Plan de Integración Segura: Token Duality Precompile

**Fecha**: 2025-11-16  
**Versión**: v1.0  
**Estado**: ✅ AUDITORÍA COMPLETA - LISTO PARA INTEGRACIÓN  

---

## 1. Resumen Ejecutivo

Este documento detalla el plan completo para integrar el Token Duality Precompile (0xFD) en ANDE Chain de manera segura y en producción.

**Auditoría de Seguridad**: ✅ **APROBADA** (Ver `SECURITY_AUDIT_PRECOMPILE.md`)  
**Vulnerabilidades Críticas**: **0 (CERO)**  
**Riesgos Residuales**: ⚠️ **BAJOS**  

---

## 2. Arquitectura Actual vs Target

### 2.1 Estado Actual (❌ NO INTEGRADO)

```
AndeNode (main.rs)
    ↓
AndeNode::components()
    ↓
EthereumNode::components()  ← ❌ Usa componentes estándar
    ↓
EthereumExecutorBuilder  ← ❌ Sin custom precompiles
    ↓
EthEvmConfig  ← ❌ Solo precompiles Ethereum estándar
```

**Problema**: El nodo compila con custom features pero NO las usa.

### 2.2 Arquitectura Target (✅ INTEGRACIÓN COMPLETA)

```
AndeNode (main.rs)
    ↓
AndeNode::components()  ← ✅ Custom components
    ↓
ComponentsBuilder
    ├── .executor(AndeExecutorBuilder)  ← ✅ Custom executor
    ├── .pool(EthereumPoolBuilder)
    ├── .network(EthereumNetworkBuilder)
    └── .payload(EthereumPayloadBuilder)
    ↓
AndeExecutorBuilder::build_evm()
    ↓
AndeEvmConfig::new(chain_spec)  ← ✅ Con precompile provider
    ↓
Reth EVM Execution
    ↓
Precompile Run (address == 0xFD)
    ↓
AndePrecompileProvider::run()  ← ✅ Token Duality ACTIVO
```

---

## 3. Pasos de Integración

### Fase 1: Preparación (✅ COMPLETADO)

- [x] Implementar `AndePrecompileProvider` con seguridad
- [x] Crear `AndeEvmConfig` wrapper
- [x] Implementar `AndeExecutorBuilder`
- [x] Auditar seguridad del precompile
- [x] Documentar superficie de ataque

### Fase 2: Integración en AndeNode (⏳ PENDIENTE)

#### 2.1 Modificar `crates/ande-reth/src/node.rs`

**Cambio Requerido**: Reemplazar `EthereumNode::components()` con custom `ComponentsBuilder`.

**Código Actual**:
```rust
pub fn components<Node>() -> ComponentsBuilder<...> {
    // ❌ Delega a Ethereum estándar
    EthereumNode::components()
}
```

**Código Target**:
```rust
use crate::executor::AndeExecutorBuilder;
use reth_node_ethereum::{
    EthereumNetworkBuilder,
    EthereumPayloadBuilder,
    EthereumPoolBuilder,
};

pub fn components<Node>() -> ComponentsBuilder<
    Node,
    EthereumPoolBuilder,
    EthereumPayloadBuilder,
    EthereumNetworkBuilder,
    AndeExecutorBuilder,  // ✅ Custom executor
>
where
    Node: FullNodeTypes<Types = AndeNode>,
    <Node as FullNodeTypes>::Provider: BlockNumReader,
{
    tracing::info!("🔧 Initializing ANDE Chain components");
    
    ComponentsBuilder::default()
        .pool(EthereumPoolBuilder::default())
        .payload(EthereumPayloadBuilder::default())
        .network(EthereumNetworkBuilder::default())
        .executor(AndeExecutorBuilder::default())  // ✅ INTEGRACIÓN AQUÍ
}
```

**Validaciones de Seguridad**:
1. ✅ `AndeExecutorBuilder` retorna `AndeEvmConfig`
2. ✅ `AndeEvmConfig` contiene `AndePrecompileProvider`
3. ✅ Precompile provider tiene auditoría de seguridad aprobada

#### 2.2 Implementar `ConfigureEvm` para `AndeEvmConfig`

**Investigación**: Basado en AlphaNet, necesitamos implementar el trait `ConfigureEvm` para que `AndeEvmConfig` pueda configurar el EVM con custom precompiles.

**Opciones de Implementación**:

**Opción A: Via Deref (Actual)**
```rust
impl std::ops::Deref for AndeEvmConfig {
    type Target = EthEvmConfig;
    fn deref(&self) -> &Self::Target {
        &self.inner  // Delega todo a EthEvmConfig
    }
}
```
❌ **Problema**: Deref bypasses precompile provider integration.

**Opción B: Implementar ConfigureEvm directamente (RECOMENDADO)**
```rust
impl ConfigureEvm for AndeEvmConfig {
    fn evm<DB: Database>(&self, db: DB) -> Evm<'_, (), DB> {
        EvmBuilder::default()
            .with_db(db)
            .append_handler_register(|handler| {
                // ✅ Integrar AndePrecompileProvider aquí
                let provider = self.precompile_provider.clone();
                handler.pre_execution.load_precompiles = Arc::new(move || {
                    // Cargar precompiles Ethereum estándar
                    let mut loaded = ContextPrecompiles::new(
                        PrecompileSpecId::from_spec_id(provider.spec_id())
                    );
                    
                    // ✅ Agregar ANDE precompile
                    loaded.extend([(
                        ANDE_PRECOMPILE_ADDRESS,
                        ContextPrecompile::Ordinary(provider.clone())
                    )]);
                    
                    loaded
                });
            })
            .build()
    }
}
```

**⚠️ IMPORTANTE**: Esta opción requiere verificar la API exacta de Reth v1.8.2 para `ConfigureEvm`.

**Opción C: Via EvmFactory (Alternativa)**

Si `ConfigureEvm` no permite override de precompiles directamente, usar un `EvmFactory` custom:

```rust
#[derive(Clone)]
pub struct AndeEvmFactory {
    precompile_provider: Arc<AndePrecompileProvider>,
}

impl EvmFactory for AndeEvmFactory {
    fn create_evm<DB: Database>(&self, db: DB, spec: SpecId) -> Evm<'_, (), DB> {
        // Similar a Opción B pero en factory pattern
    }
}

// En AndeEvmConfig:
pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
    let spec_id = Self::spec_id_from_chain(&chain_spec);
    let precompile_provider = Arc::new(AndePrecompileProvider::new(spec_id));
    
    let factory = AndeEvmFactory { 
        precompile_provider: precompile_provider.clone() 
    };
    
    Self {
        inner: EthEvmConfig::new_with_factory(chain_spec, factory),
        precompile_provider,
    }
}
```

### Fase 3: Testing (⏳ PENDIENTE)

Ver `SECURITY_AUDIT_PRECOMPILE.md` Sección 5 para test cases completos.

**Tests Críticos** (MUST HAVE antes de mainnet):

```rust
// crates/ande-evm/src/evm_config/tests.rs

#[cfg(test)]
mod integration_tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use revm::{Database, InMemoryDB};
    
    #[test]
    fn test_precompile_integration_e2e() {
        // Setup
        let chain_spec = Arc::new(test_chain_spec());
        let evm_config = AndeEvmConfig::new(chain_spec);
        let db = InMemoryDB::default();
        
        // Configure EVM
        let mut evm = evm_config.evm(db);
        
        // Create transaction calling precompile 0xFD
        let from = address!("0000000000000000000000000000000000000001");
        let to = address!("00000000000000000000000000000000000000FD");
        let value = U256::from(1000);
        
        let tx = TransactionBuilder::default()
            .from(from)
            .to(to)
            .input(abi::encode(&[from, to, value]))
            .gas_limit(10000)
            .build();
        
        // Execute
        let result = evm.transact(tx);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().result, InstructionResult::Return);
    }
    
    #[test]
    fn test_static_call_rejected() {
        // Test que STATICCALL al precompile falla
        let evm_config = AndeEvmConfig::new(test_chain_spec());
        // ... setup ...
        
        let result = evm.transact_static(tx);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(), 
            "Cannot modify state in static call"
        );
    }
    
    #[test]
    fn test_insufficient_gas() {
        // Gas < 3300 debe fallar
    }
    
    #[test]
    fn test_zero_address_rejected() {
        // Transfer a 0x0 debe fallar
    }
}
```

**Tests de Integración con Docker** (SHOULD HAVE):

```bash
#!/bin/bash
# test-precompile-integration.sh

set -e

echo "🧪 Testing Token Duality Precompile Integration"

# 1. Deploy test contract
echo "1. Deploying test contract..."
forge create \
    --rpc-url http://192.168.0.8:8545 \
    --private-key $PK \
    TestPrecompile

# 2. Call precompile via contract
echo "2. Testing precompile call..."
cast send \
    --rpc-url http://192.168.0.8:8545 \
    --private-key $PK \
    $CONTRACT_ADDR \
    "testTokenDuality(address,uint256)" \
    "0x..." \
    "1000000000000000000"

# 3. Verify balances changed
echo "3. Verifying balances..."
BALANCE=$(cast balance 0x... --rpc-url http://192.168.0.8:8545)
echo "Balance: $BALANCE"

# 4. Test static call rejection
echo "4. Testing static call protection..."
# Should fail
cast call \
    --rpc-url http://192.168.0.8:8545 \
    0x00..FD \
    "transfer(address,address,uint256)" \
    && echo "❌ FAIL: Static call should be rejected" \
    || echo "✅ PASS: Static call rejected correctly"

echo "✅ All tests passed!"
```

### Fase 4: Deployment (⏳ PENDIENTE)

#### 4.1 Build Docker Image

```bash
cd /Users/munay/dev/ande-labs/ande-chain

# Build locally primero (test)
cargo build --release --features "ande-reth/default"

# Si compila, build Docker
docker build -t ande-reth:v1.0-precompile .

# Tag
docker tag ande-reth:v1.0-precompile ghcr.io/andelabs/ande-reth:v1.0-precompile

# Push
docker push ghcr.io/andelabs/ande-reth:v1.0-precompile
```

#### 4.2 Update docker-compose.yml

```yaml
services:
  ande-node:
    # image: ghcr.io/paradigmxyz/reth:v1.8.2  ← OLD
    image: ghcr.io/andelabs/ande-reth:v1.0-precompile  # ← NEW
    restart: unless-stopped
    # ... resto igual
```

#### 4.3 Deploy to Server

```bash
# SSH al servidor
ssh sator@192.168.0.8

cd /path/to/ande-chain

# Pull nueva imagen
docker-compose pull ande-node

# ⚠️ IMPORTANTE: Clean state para fresh start
docker-compose down -v

# Restart
docker-compose up -d

# Monitor logs
docker logs -f ande-chain-ande-node-1 | grep "Token Duality"
```

**Expected Logs**:
```
🔧 Building ANDE EVM with custom precompiles
✅ ANDE EVM configured:
   • Chain ID: 41455
   • Token Duality Precompile: 0x00..FD (ACTIVE)
   • Spec: CANCUN
```

---

## 4. Verificación Post-Deploy

### 4.1 Verificar Precompile Activo

```bash
# Test 1: Call directo al precompile
cast call \
    --rpc-url http://192.168.0.8:8545 \
    0x00000000000000000000000000000000000000FD \
    "balanceOf(address)" \
    "0x..."

# Test 2: Via smart contract
# Deploy contract que use el precompile y verificar funciona
```

### 4.2 Monitor de Seguridad

```bash
# Filtrar logs de precompile
docker logs ande-chain-ande-node-1 2>&1 | grep "ANDE native transfer"

# Buscar errores
docker logs ande-chain-ande-node-1 2>&1 | grep -i "static call"
docker logs ande-chain-ande-node-1 2>&1 | grep -i "insufficient gas"
```

### 4.3 Alertas Prometheus (Futuro)

```yaml
# prometheus-alerts.yml
groups:
  - name: ande-precompile
    rules:
      - alert: PrecompileStaticCallAttempt
        expr: rate(ande_precompile_static_call_errors[5m]) > 0
        annotations:
          summary: "Static call to precompile attempted"
          description: "Potential attack or misconfigured contract"
      
      - alert: PrecompileHighUsage
        expr: rate(ande_precompile_calls[1m]) > 1000
        annotations:
          summary: "Precompile usage spike"
          description: "Investigate potential spam or DoS"
```

---

## 5. Rollback Plan

Si hay problemas post-deploy:

### 5.1 Rollback Rápido

```bash
# 1. Revertir imagen de Docker
docker-compose down

# Edit docker-compose.yml
sed -i 's/v1.0-precompile/v1.8.2/' docker-compose.yml

# 2. Restart con Reth estándar
docker-compose up -d

# 3. Verificar
cast block-number --rpc-url http://192.168.0.8:8545
```

### 5.2 Análisis Post-Mortem

1. Recopilar logs completos:
```bash
docker logs ande-chain-ande-node-1 > rollback-logs.txt
docker logs ande-chain-evolve-1 >> rollback-logs.txt
```

2. Identificar causa raíz
3. Fix en branch separado
4. Re-test localmente
5. Re-deploy cuando esté fixed

---

## 6. Roadmap de Features

### v1.0 (Current) - Token Duality Precompile
- ✅ Precompile implementado y auditado
- ⏳ Integración en node
- ⏳ Tests exhaustivos
- ⏳ Deployment a producción

### v1.1 - Monitoring & Observability
- ⏳ Métricas Prometheus
- ⏳ Grafana dashboards
- ⏳ Alerting

### v2.0 - Parallel EVM (Block-STM)
- ⏳ Implementar multi-version concurrency control
- ⏳ Conflict detection
- ⏳ Benchmarks de throughput

### v3.0 - MEV Detection & Distribution
- ⏳ Bundle execution
- ⏳ MEV profit tracking
- ⏳ Fair distribution (80/20 split)

---

## 7. Checklist Pre-Deploy

**Antes de hacer `docker-compose up` con la nueva imagen**:

- [ ] Código compilado localmente sin errores
- [ ] Tests unitarios pasando (Sección 3, Fase 3)
- [ ] Tests de integración pasando
- [ ] Docker image built y pushed a registry
- [ ] Backup de estado actual del nodo
- [ ] Rollback plan documentado (Sección 5)
- [ ] Monitoring configurado (logs, métricas)
- [ ] Equipo notificado del deploy
- [ ] Ventana de mantenimiento programada
- [ ] Documentación actualizada

**Post-Deploy**:

- [ ] Verificar logs muestran precompile activo
- [ ] Test manual con cast call
- [ ] Verificar métricas (si disponibles)
- [ ] Monitor por 24h para stability
- [ ] Actualizar status docs

---

## 8. Decisiones Pendientes

### 8.1 API de ConfigureEvm en Reth v1.8.2

**Pregunta**: ¿Cuál es la API exacta para override precompiles?

**Opciones**:
1. Implementar trait `ConfigureEvm` directamente
2. Usar `EvmFactory` pattern
3. Override via `append_handler_register`

**Acción**: Investigar en Reth v1.8.2 source code o AlphaNet reference.

**Código a Revisar**:
```bash
# Buscar en Reth source
cd /tmp
git clone https://github.com/paradigmxyz/reth --branch v1.8.2 --depth 1
cd reth
rg "trait ConfigureEvm" -A 20
rg "append_handler_register" -B 5 -A 10
```

### 8.2 Testing Strategy

**Pregunta**: ¿Dónde ejecutar tests de integración?

**Opciones**:
1. Localmente con testnet (devnet)
2. En servidor de staging (192.168.0.8 con flag `--chain dev`)
3. CI/CD pipeline

**Recomendación**: Opción 2 + 3 (staging server + CI)

### 8.3 Monitoring

**Pregunta**: ¿Qué nivel de monitoring implementar en v1.0?

**Opciones**:
1. Solo logs (mínimo viable)
2. Logs + Prometheus metrics
3. Logs + Prometheus + Grafana + Alerting

**Recomendación**: Opción 1 para v1.0, Opción 3 para v1.1

---

## 9. Referencias

**Documentos Internos**:
- `SECURITY_AUDIT_PRECOMPILE.md` - Auditoría completa de seguridad
- `ARCHITECTURE_STATUS.md` - Estado de features personalizadas
- `PRECOMPILE_INTEGRATION_FINDINGS.md` - Hallazgos de integración

**Código Fuente**:
- `crates/ande-evm/src/evm_config/ande_precompile_provider.rs` - Implementación
- `crates/ande-evm/src/evm_config/wrapper.rs` - AndeEvmConfig
- `crates/ande-reth/src/executor.rs` - AndeExecutorBuilder
- `crates/ande-reth/src/node.rs` - AndeNode types

**Referencias Externas**:
- Reth v1.8.2: https://github.com/paradigmxyz/reth/tree/v1.8.2
- AlphaNet (ejemplo custom precompiles): https://github.com/paradigmxyz/alphanet
- REVM precompile docs: https://github.com/bluealloy/revm
- EIP-214 (STATICCALL): https://eips.ethereum.org/EIPS/eip-214

---

**FIN DEL PLAN DE INTEGRACIÓN**

**Siguiente Paso**: Investigar API exacta de `ConfigureEvm` en Reth v1.8.2 (Sección 8.1).
