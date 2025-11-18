# ANDE Chain Infrastructure Roadmap

> Guía completa para construir una blockchain poderosa, rápida, robusta y escalable

**Última actualización**: 2025-11-18
**Versión**: 1.0
**Estado**: Planning

---

## Tabla de Contenidos

1. [Arquitectura Actual](#arquitectura-actual)
2. [Prioridades de Implementación](#prioridades-de-implementación)
3. [Arquitectura Objetivo](#arquitectura-objetivo)
4. [Endpoints Recomendados](#endpoints-recomendados)
5. [Métricas Target](#métricas-target)
6. [Plan de Implementación por Fases](#plan-de-implementación-por-fases)
7. [Detalles Técnicos por Componente](#detalles-técnicos-por-componente)

---

## Arquitectura Actual

### Endpoints Activos

| Endpoint | Puerto | Servicio | Descripción |
|----------|--------|----------|-------------|
| `rpc.ande.network` | 8545 | Reth | JSON-RPC HTTP para transacciones |
| `ws.ande.network` | 8546 | Reth | WebSocket RPC para subscripciones blockchain |
| `api.ande.network` | 4000 | BlockScout | REST API + WebSocket del explorer |
| `explorer.ande.network` | 4000 | BlockScout | UI del block explorer |
| `faucet.ande.network` | 8080 | Faucet | Testnet faucet |
| `status.ande.network` | 3000 | Grafana | Monitoring dashboard |

### Stack Tecnológico

- **Execution Layer**: Reth v1.8.2 (fork con customizaciones)
- **Consensus Layer**: Evolve (BFT Multi-Validator)
- **Data Availability**: Celestia
- **Explorer**: BlockScout
- **Chain ID**: 6174

---

## Prioridades de Implementación

### Prioridad 1: Rendimiento y Velocidad (Crítico)

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **Parallel EVM** | Procesar múltiples transacciones simultáneamente | 10x throughput (2,000+ TPS) | Alta |
| **Mempool Streaming** | WebSocket para transacciones pendientes en tiempo real | Trading bots, MEV protection | Media |
| **Archive Node** | Nodo con todo el historial de estados | Analytics, debugging, compliance | Media |
| **Geographic Distribution** | Nodos en múltiples regiones (Frankfurt, Singapore, US-East) | Latencia < 50ms global | Alta |
| **Load Balancer** | Round-robin o least-connections para RPC | Alta disponibilidad 99.95% | Baja |

**Referencia**: Sonic (ex-Fantom) alcanza 2,000+ TPS con finality < 700ms usando DAG + parallel processing.

### Prioridad 2: Experiencia de Usuario (Alto Impacto)

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **Account Abstraction (ERC-4337)** | Smart contract wallets | Gasless transactions, social recovery | Media |
| **Bundler Service** | Agrupa UserOperations | Infraestructura AA | Media |
| **Paymaster** | Patrocinio de gas | Onboarding sin gas, pago con ERC-20 | Media |
| **Faucet Mejorado** | Rate limiting por IP/wallet, captcha | Previene abuso | Baja |

**Componentes ERC-4337**:
- **UserOperation**: Pseudo-transacción que representa la intención del usuario
- **Bundler**: Nodo que agrupa UserOps y las envía al EntryPoint
- **EntryPoint**: Contrato singleton que verifica y ejecuta bundles
- **Paymaster**: Contrato que puede pagar gas por otros usuarios

### Prioridad 3: Interoperabilidad (Expansión)

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **LayerZero** | Mensajería cross-chain | Comunicación con 30+ chains | Alta |
| **Wormhole** | Bridge para tokens y mensajes | Conexión EVM + Solana | Alta |
| **Hyperlane** | Bridge permissionless | Fácil integración | Media |
| **Native Bridge** | ANDE ↔ Ethereum | Depositar/retirar assets | Alta |

**Comparativa de Bridges**:
- **LayerZero**: Ultra-Light Nodes, rápido, oracle-relayer pairs
- **Wormhole**: Guardian network, alta confianza, 14+ blockchains
- **Hyperlane**: Modular, permissionless, delegated PoS

### Prioridad 4: Oracles y Datos Externos

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **Chainlink Price Feeds** | Precios de assets en tiempo real | DeFi, lending, DEXs | Baja |
| **Pyth Network** | Feeds de alta frecuencia (2-3/seg) | Trading de derivados | Baja |
| **Chainlink VRF** | Randomness verificable | Gaming, NFT minting, lotteries | Baja |
| **Chainlink Automation** | Ejecución automática de contratos | Liquidaciones, rebalancing | Media |

**Chainlink vs Pyth**:
- **Chainlink**: 2,000+ price feeds, VRF, Automation, CCIP
- **Pyth**: Pull oracle design, 2-3 updates/segundo, low-latency

### Prioridad 5: MEV y Protección

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **MEV Protection** | Private mempool o Flashbots-style | Protege usuarios de front-running | Alta |
| **MEV Redistribution** | Ya implementado (80/20) | Reward stakers | ✅ Completo |
| **Transaction Ordering** | First-come-first-served o PBS | Fairness | Media |
| **Encrypted Mempool** | Transacciones cifradas hasta ejecución | Máxima protección | Muy Alta |

**Nota**: En 2022, $2B fueron perdidos en hacks de bridges (69% del total robado). La seguridad es crítica.

### Prioridad 6: Developer Experience

| Componente | Descripción | Beneficio | Complejidad |
|------------|-------------|-----------|-------------|
| **Subgraph/Indexer** | The Graph o indexer custom | Query eficiente de datos | Media |
| **Event Streaming** | gRPC streams para eventos | Real-time dApps | Media |
| **Debug/Trace APIs** | debug_traceCall, trace_block | Debugging avanzado | Baja |
| **Gas Estimation API** | Estimación precisa de gas | UX mejorado | Baja |
| **SDK Oficial** | TypeScript SDK para ANDE | Fácil integración | Media |

---

## Arquitectura Objetivo

```
                         ┌─────────────────────────────────────────┐
                         │           Global Load Balancer          │
                         │    (Cloudflare/AWS/Geographic routing)  │
                         └────────────────┬────────────────────────┘
                                          │
              ┌───────────────────────────┼───────────────────────────┐
              │                           │                           │
     ┌────────▼─────────┐       ┌─────────▼──────────┐      ┌────────▼────────┐
     │   US-East Node   │       │   Frankfurt Node   │      │  Singapore Node │
     │   (Primary)      │       │   (Replica)        │      │   (Replica)     │
     └────────┬─────────┘       └─────────┬──────────┘      └────────┬────────┘
              │                           │                           │
              └───────────────────────────┼───────────────────────────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
           ┌────────▼────────┐   ┌────────▼────────┐   ┌───────▼───────┐
           │   RPC Layer     │   │  Archive Node   │   │   Indexer     │
           │ (8545, 8546)    │   │ (Historical)    │   │ (BlockScout)  │
           └────────┬────────┘   └────────┬────────┘   └───────┬───────┘
                    │                     │                     │
                    └─────────────────────┼─────────────────────┘
                                          │
              ┌───────────────────────────┼───────────────────────────┐
              │                           │                           │
     ┌────────▼─────────┐       ┌─────────▼──────────┐      ┌────────▼────────┐
     │   AA Bundler     │       │    Paymaster       │      │   MEV Relay     │
     │   (ERC-4337)     │       │  (Gas Sponsor)     │      │  (Protection)   │
     └──────────────────┘       └────────────────────┘      └─────────────────┘
```

### Flujo de Datos

```
Usuario → Load Balancer → Nodo Regional → Sequencer → Consenso → Celestia DA
                ↓
         BlockScout (indexing) → API/WebSocket → Explorer/dApps
```

---

## Endpoints Recomendados

### Core Blockchain
```bash
rpc.ande.network          # JSON-RPC HTTP (público)
ws.ande.network           # WebSocket RPC (subscriptions blockchain)
archive.ande.network      # Archive node (historical queries)
mempool.ande.network      # Mempool streaming API
```

### Explorer & API
```bash
explorer.ande.network     # BlockScout UI
api.ande.network          # BlockScout API + WebSocket
```

### Account Abstraction
```bash
bundler.ande.network      # ERC-4337 Bundler
paymaster.ande.network    # Gas sponsoring API
```

### Cross-chain
```bash
bridge.ande.network       # Native bridge UI
gateway.ande.network      # LayerZero/Wormhole gateway
```

### Developer Tools
```bash
faucet.ande.network       # Testnet faucet
status.ande.network       # Network health dashboard
docs.ande.network         # API documentation
graph.ande.network        # Subgraph/Indexer queries
```

---

## Métricas Target

### Comparativa con Competidores

| Métrica | ANDE Actual | ANDE Target | Sonic | Base | Arbitrum |
|---------|-------------|-------------|-------|------|----------|
| TPS | ~200 | 2,000+ | 10,000 | 2,000 | 4,000 |
| Finality | ~5s | < 1s | 700ms | 2s | ~1s |
| RPC Latency (p95) | ~300ms | < 100ms | < 50ms | < 100ms | < 100ms |
| Uptime | ? | 99.95% | 99.99% | 99.95% | 99.95% |
| Block Time | 5s | < 1s | 1s | 2s | 250ms |
| WebSocket Latency | N/A | < 50ms | < 30ms | < 50ms | < 50ms |

### SLAs Recomendados (L2 Best Practices)

**RPC Layer**:
- Uptime: 99.5%
- Read operations: < 300ms (p95)
- Write operations: < 500ms (p95)
- Throughput: 5,000 req/s sustained
- Indexer lag: ≤ 1 block

**Recovery**:
- RPO (Recovery Point Objective): ≤ 15 minutos
- RTO (Recovery Time Objective): 30-60 minutos
- Fresh node bootstrap: ≤ 15 minutos

---

## Plan de Implementación por Fases

### Fase 1: Fundamentos (1-2 semanas)

**Objetivo**: Estabilizar infraestructura base y WebSocket

- [ ] **1.1** Arreglar WebSocket de BlockScout
  - Actualizar nginx en servidor (configuración ya lista)
  - Verificar conexión desde frontend
  - Monitorear latencia y reconexiones

- [ ] **1.2** Configurar Archive Node
  - Deploy nodo adicional con `--gcmode=archive`
  - Endpoint separado: `archive.ande.network`
  - Backup y sincronización

- [ ] **1.3** Implementar Load Balancer
  - Configurar Cloudflare Load Balancing o nginx upstream
  - Health checks automáticos
  - Failover automático

- [ ] **1.4** Añadir Mempool streaming API
  - Endpoint WebSocket para pending transactions
  - Filtros por address/contract
  - Rate limiting

### Fase 2: Account Abstraction (2-3 semanas)

**Objetivo**: Habilitar transacciones gasless y smart wallets

- [ ] **2.1** Deploy EntryPoint contract
  - Usar contrato oficial de ERC-4337
  - Verificar en explorer

- [ ] **2.2** Configurar Bundler service
  - Deploy bundler (Stackup, Alchemy, o custom)
  - Endpoint: `bundler.ande.network`
  - Monitoreo de bundles

- [ ] **2.3** Implementar Paymaster
  - Paymaster para gas sponsoring
  - Soporte para pago con ERC-20
  - Dashboard de uso

- [ ] **2.4** Integrar con explorer
  - Mostrar UserOperations
  - Decodificar paymaster data
  - Link bundler → transaction

### Fase 3: Oracles & DeFi (2-3 semanas)

**Objetivo**: Habilitar ecosistema DeFi

- [ ] **3.1** Integrar Chainlink Price Feeds
  - Deploy price feed proxies
  - ETH/USD, BTC/USD, ANDE/USD
  - Documentación para devs

- [ ] **3.2** Integrar Chainlink VRF
  - Deploy VRF Coordinator
  - Ejemplo de uso (lottery contract)
  - Funding de subscriptions

- [ ] **3.3** Integrar Pyth Network
  - Deploy Pyth contracts
  - Configurar price feeds
  - Ejemplo de uso

- [ ] **3.4** Deploy contratos de ejemplo
  - Simple DEX usando Chainlink
  - NFT mint con VRF
  - Lending protocol básico

### Fase 4: Interoperabilidad (3-4 semanas)

**Objetivo**: Conectar ANDE con otras blockchains

- [ ] **4.1** Implementar Native Bridge
  - Smart contracts en ANDE y Ethereum
  - Relayer service
  - UI de bridge

- [ ] **4.2** Integrar LayerZero
  - Deploy LayerZero endpoints
  - Configurar OApp
  - Testing cross-chain

- [ ] **4.3** Añadir soporte para Wormhole
  - Deploy Wormhole contracts
  - Configurar Guardian
  - Testing transfers

- [ ] **4.4** UI de bridge unificada
  - Frontend para todos los bridges
  - Comparador de fees/tiempo
  - Historial de transfers

### Fase 5: Performance Optimization (Ongoing)

**Objetivo**: Alcanzar métricas de Sonic

- [ ] **5.1** Parallel EVM
  - Investigar implementación de Monad/Sei
  - Fork de REVM con parallel execution
  - Testing exhaustivo

- [ ] **5.2** State compression
  - Implementar state pruning
  - Optimizar storage
  - Reducir sync time

- [ ] **5.3** Transaction batching
  - Optimizar mempool ordering
  - Batch similar transactions
  - Reducir overhead

---

## Detalles Técnicos por Componente

### Account Abstraction (ERC-4337)

**Contratos Principales**:
```solidity
// EntryPoint - Singleton global
address constant ENTRY_POINT = 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789;

// UserOperation structure
struct UserOperation {
    address sender;
    uint256 nonce;
    bytes initCode;
    bytes callData;
    uint256 callGasLimit;
    uint256 verificationGasLimit;
    uint256 preVerificationGas;
    uint256 maxFeePerGas;
    uint256 maxPriorityFeePerGas;
    bytes paymasterAndData;
    bytes signature;
}
```

**Bundler API**:
```bash
POST https://bundler.ande.network/rpc
{
  "jsonrpc": "2.0",
  "method": "eth_sendUserOperation",
  "params": [userOp, entryPoint],
  "id": 1
}
```

### Chainlink Integration

**Price Feeds**:
```solidity
import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";

contract PriceConsumer {
    AggregatorV3Interface internal priceFeed;

    constructor() {
        // ETH/USD en ANDE Chain
        priceFeed = AggregatorV3Interface(0x...);
    }

    function getLatestPrice() public view returns (int) {
        (, int price,,,) = priceFeed.latestRoundData();
        return price;
    }
}
```

**VRF (Randomness)**:
```solidity
import "@chainlink/contracts/src/v0.8/VRFConsumerBaseV2.sol";

contract RandomNFT is VRFConsumerBaseV2 {
    function requestRandomWords() external returns (uint256 requestId) {
        requestId = COORDINATOR.requestRandomWords(
            keyHash,
            subscriptionId,
            requestConfirmations,
            callbackGasLimit,
            numWords
        );
    }

    function fulfillRandomWords(uint256, uint256[] memory randomWords) internal override {
        // Mint NFT with random traits
    }
}
```

### Cross-Chain Messaging (LayerZero)

```solidity
import "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/OApp.sol";

contract CrossChainToken is OApp {
    function send(
        uint32 _dstEid,
        bytes memory _message,
        bytes memory _options
    ) external payable {
        _lzSend(_dstEid, _message, _options, MessagingFee(msg.value, 0), msg.sender);
    }

    function _lzReceive(
        Origin calldata,
        bytes32,
        bytes calldata _message,
        address,
        bytes calldata
    ) internal override {
        // Handle incoming message
    }
}
```

### Nginx WebSocket Configuration

```nginx
# WebSocket para BlockScout Phoenix Channels
location /socket {
    proxy_pass http://blockscout;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    # Long-lived connections (7 días)
    proxy_connect_timeout 7d;
    proxy_send_timeout 7d;
    proxy_read_timeout 7d;

    # Sin buffering para real-time
    proxy_buffering off;
}
```

---

## Referencias

### Documentación Oficial
- [ERC-4337 Specification](https://eips.ethereum.org/EIPS/eip-4337)
- [Chainlink Documentation](https://docs.chain.link/)
- [LayerZero V2 Docs](https://docs.layerzero.network/)
- [Wormhole Docs](https://docs.wormhole.com/)
- [Pyth Network Docs](https://docs.pyth.network/)

### Investigación
- [Sonic Chain Architecture](https://docs.soniclabs.com/)
- [L2 SLA Best Practices](https://witnesschain.com/blogs/l2-blockchain-slas-2025)
- [MEV Protection Strategies](https://blog.shutter.network/)

### Herramientas
- [Stackup Bundler](https://github.com/stackup-wallet/stackup-bundler)
- [Alchemy AA SDK](https://www.alchemy.com/account-abstraction)
- [The Graph](https://thegraph.com/)

---

## Changelog

### v1.0 (2025-11-18)
- Documento inicial
- Definición de arquitectura objetivo
- Plan de implementación por fases
- Detalles técnicos de componentes principales

---

## Contacto

- **RPC**: rpc.ande.network
- **Explorer**: explorer.ande.network
- **Server**: 192.168.0.8
- **GitHub**: github.com/AndeLabs

---

*Este documento es una guía viva y debe actualizarse conforme se implementen los componentes.*
