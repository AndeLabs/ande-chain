# ANDE Chain - Estado Actual de Deployment

**Fecha**: 2025-11-15
**Chain ID**: 6174 (0x181e)
**RPC**: http://192.168.0.8:8545
**Genesis**: ✅ Actualizado con 540 storage slots

---

## 🎯 ESTADO ACTUAL

### Genesis ✅ COMPLETADO
- 540 storage slots en dirección `0x00...01`
- 16 referencias culturales quechua (slots 0x00-0x0F)
- 4 Celestia metadata pointers (slots 0x10-0x13)
- 520 plant seeds criptográficos (slots 0x100-0x307)
- Node compilando y funcionando con Reth v1.8.2

### Smart Contracts ❌ PENDIENTE DE DEPLOYMENT

La red fue reiniciada con el nuevo genesis, por lo tanto **TODOS** los contratos necesitan ser desplegados desde cero.

---

## 📋 CONTRATOS A DESPLEGAR

### TIER 1 - Core Infrastructure (CRÍTICO)

| Contrato | Estado | Prioridad | Notas |
|----------|--------|-----------|-------|
| **ANDETokenDuality** | ❌ NO DESPLEGADO | 🔴 CRÍTICA | Implementation + Proxy UUPS |
| **AndeNativeStaking** | ❌ NO DESPLEGADO | 🔴 CRÍTICA | Implementation + Proxy UUPS |
| **AndeSequencerRegistry** | ❌ NO DESPLEGADO | 🔴 CRÍTICA | Implementation + Proxy UUPS |

### TIER 2 - Governance (IMPORTANTE)

| Contrato | Estado | Prioridad | Notas |
|----------|--------|-----------|-------|
| **AndeTimelockController** | ❌ NO DESPLEGADO | 🟡 IMPORTANTE | Implementation + Proxy UUPS |
| **AndeGovernor** | ❌ NO DESPLEGADO | 🟡 IMPORTANTE | Implementation + Proxy UUPS |

---

## 🚀 PLAN DE DEPLOYMENT

### Paso 1: Verificar Prerequisitos ✅

```bash
# RPC disponible
curl -X POST http://192.168.0.8:8545 \
  -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
# ✅ Chain ID: 0x181e (6174)

# Contratos compilados
cd contracts && forge build
# ✅ Compilación exitosa
```

### Paso 2: Desplegar TIER 1

**2.1. ANDETokenDuality**
```bash
RPC_URL="http://192.168.0.8:8545"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
DEPLOYER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Deploy implementation
forge create src/token/ANDETokenDuality.sol:ANDETokenDuality \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY

# Deploy proxy + initialize
# ...
```

**2.2. AndeNativeStaking**
```bash
# Similar process...
```

**2.3. AndeSequencerRegistry**
```bash
# Similar process...
```

### Paso 3: Desplegar TIER 2

**3.1. AndeTimelockController**
```bash
# Delay: 3600 segundos (1 hora)
```

**3.2. AndeGovernor**
```bash
# Voting period: 21600 blocks
# Voting delay: 1 block
# Proposal threshold: 1000 ANDE
```

### Paso 4: Configurar Roles

```bash
# MINTER_ROLE: Token → Staking
cast send $TOKEN_ADDR "grantRole(bytes32,address)" \
  0x9f2df0fed2c77648de5860a4cc508cd0818c85b8b8a1ab4ceeef8d981c8956a6 \
  $STAKING_ADDR

# PROPOSER_ROLE: Timelock → Governor
cast send $TIMELOCK_ADDR "grantRole(bytes32,address)" \
  0xb09aa5aeb3702cfd50b6b62bc4532604938f21248a27a1d5ca736082b6819cc1 \
  $GOVERNOR_ADDR

# EXECUTOR_ROLE: Timelock → address(0)
cast send $TIMELOCK_ADDR "grantRole(bytes32,address)" \
  0xd8aa0f3194971a2a116679f7c2090f6939c8d4e01a2a8d7e41d55e5351469e63 \
  0x0000000000000000000000000000000000000000
```

---

## 📊 ORDEN RECOMENDADO

1. ✅ **Genesis** - Completado (540 slots on-chain)
2. ⏳ **TIER 1 Deployment** - Pendiente (~20 minutos)
   - ANDETokenDuality
   - AndeNativeStaking
   - AndeSequencerRegistry
3. ⏳ **TIER 2 Deployment** - Pendiente (~15 minutos)
   - AndeTimelockController
   - AndeGovernor
4. ⏳ **Role Configuration** - Pendiente (~5 minutos)
5. ⏳ **Verification** - Pendiente (~5 minutos)

**Tiempo total estimado**: ~45 minutos

---

## 🔄 SIGUIENTES PASOS

### Opción A: Deployment Completo (Recomendado)
Desplegar todos los contratos TIER 1 y TIER 2 para tener la infraestructura completa lista.

**Beneficios**:
- Infraestructura completa operacional
- Governance funcional
- Sequencer registry activo
- Ready for production

### Opción B: Deployment Parcial
Desplegar solo TIER 1 primero, TIER 2 después.

**Beneficios**:
- Más rápido (solo ~20 minutos)
- Core functionality operacional
- Governance puede esperar

### Opción C: Postergar Deployment
Mantener solo el genesis operacional, desplegar contratos después.

**Beneficios**:
- Genesis ya está listo y funcionando
- Node operacional con cultural references on-chain
- Contratos pueden desplegarse cuando se necesiten

---

## 📝 NOTAS IMPORTANTES

1. **Genesis ya está listo**: El node puede funcionar sin contratos desplegados
2. **Red limpia**: La red fue reiniciada con el nuevo genesis
3. **Contratos compilados**: Todos los contratos compilan correctamente
4. **RPC disponible**: El RPC está accesible en `http://192.168.0.8:8545`
5. **Deployer account**: Usar account 0 de Hardhat/Anvil para testnet

---

## ✅ VERIFICACIÓN

Para verificar el estado actual:

```bash
RPC="http://192.168.0.8:8545"

# 1. Verificar Chain ID
cast chain-id --rpc-url $RPC
# Expected: 6174

# 2. Verificar genesis slots
cast storage 0x0000000000000000000000000000000000000001 0x00 --rpc-url $RPC
# Expected: 0x59616368616b00... (Yachak)

# 3. Verificar balance deployer
cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url $RPC
# Expected: > 0 (debe tener fondos del genesis)
```

---

## 🎯 DECISIÓN RECOMENDADA

**Para tener ANDE Chain 100% operacional**: Proceder con **Opción A** - Deployment Completo

Esto dará:
- ✅ Genesis con cultural references on-chain
- ✅ Token system completo
- ✅ Staking mechanism
- ✅ Sequencer registry
- ✅ Governance system
- ✅ Toda la infraestructura core lista

**Tiempo**: ~45 minutos
**Status después**: PRODUCTION READY 🚀

---

**Última actualización**: 2025-11-15
**Responsable**: ANDE Labs Team
