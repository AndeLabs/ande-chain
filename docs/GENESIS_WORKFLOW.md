# Genesis Workflow - ANDE Chain

Flujo estándar de Reth para crear e inicializar el genesis block.

## 📁 Arquitectura

```
ande-chain/
├── specs/
│   └── genesis.json          ← Fuente de verdad (archivo estándar de Reth)
├── scripts/
│   └── generate-genesis-plants.py  ← Genera storage slots
└── crates/
    └── ande-node/
        └── src/main.rs       ← Carga genesis.json al iniciar
```

## 🔄 Flujo Completo

### 1. Genesis Estándar (`specs/genesis.json`)

```json
{
  "config": {
    "chainId": 6174,
    "homesteadBlock": 0,
    ...
  },
  "alloc": {
    "0x00000000000000000000000000000000000000fd": {
      "balance": "0x0"
    },
    "0x0000000000000000000000000000000000000001": {
      "balance": "0x0",
      "storage": {
        "0x0000...0000": "0x536f6e6b276f2077616368617279",
        "0x0000...0001": "0x4e4342492e6e6c6d2e6e69682e676f76",
        ...
      }
    }
  },
  "gasLimit": "0x1c9c380",
  ...
}
```

### 2. Node Carga Genesis (`crates/ande-node/src/main.rs`)

```rust
fn build_ande_chain_spec() -> Result<Arc<ChainSpec>> {
    // Cargar desde archivo estándar
    let genesis_path = std::env::var("GENESIS_PATH")
        .unwrap_or_else(|_| "specs/genesis.json".to_string());

    let genesis_content = std::fs::read_to_string(&genesis_path)?;
    let genesis: Genesis = serde_json::from_str(&genesis_content)?;

    // Construir ChainSpec usando el método estándar de Reth
    let spec = ChainSpecBuilder::default()
        .chain(CHAIN_ID.into())
        .genesis(genesis)
        .build();

    Ok(Arc::new(spec))
}
```

### 3. Método de Inicialización

**Opción A: Integrado (actual)**
```bash
cargo run -p ande-node
```
- El node carga `specs/genesis.json` automáticamente
- ChainSpec se construye en memoria

**Opción B: Reth init (alternativo)**
```bash
reth init --datadir data/ --chain specs/genesis.json
```
- Crea el genesis block en la base de datos
- Requiere usar `reth` directamente

## 🌱 Agregar Datos al Genesis

### Método 1: Storage Slots Directos

Usar dirección específica con storage slots:

```json
{
  "alloc": {
    "0x0000000000000000000000000000000000000001": {
      "balance": "0x0",
      "storage": {
        "0x0000000000000000000000000000000000000000000000000000000000000000": "0x...",
        "0x0000000000000000000000000000000000000000000000000000000000000001": "0x..."
      }
    }
  }
}
```

**Ventajas:**
- ✅ Método estándar de Ethereum/Reth
- ✅ Datos disponibles desde bloque 0
- ✅ No requiere transacciones

**Limitaciones:**
- ❌ Limitado por tamaño de genesis file
- ❌ Difícil de mantener muchos datos

### Método 2: Híbrido con Celestia DA

**Seeds on-chain + Metadata off-chain:**

```json
{
  "alloc": {
    "0x0000000000000000000000000000000000000001": {
      "balance": "0x0",
      "storage": {
        "0x00": "0x616e6465706c616e74735f763100000000000000000000000000",  // Celestia namespace
        "0x01": "0x0000000000000000000000000000000000000000000000000000000012d687",  // Start height
        "0x02": "0x0000000000000000000000000000000000000000000000000000000012d6ff",  // End height
        "0x03": "0x000000000000000000000000000000000000000000000000000000000000008c",  // Total blobs (140)
        ...
        "0x100": "0x35a2a6d03ee094cb624310ec6e09d7b1fb176e068dc74b0fb81fe6828c0f6fda",  // Plant 1 seed
        "0x101": "0xa7398481d3624783e8604d18bd94829dd373a767106701e9c79519ed017f028f",  // Plant 2 seed
        ...
      }
    }
  }
}
```

**Ventajas:**
- ✅ Seeds on-chain (verificables)
- ✅ Metadata en Celestia DA (escalable)
- ✅ Genesis file pequeño (~30 KB vs ~10 MB)
- ✅ 85% más económico

## 🛠️ Scripts de Generación

### Python Script (`scripts/generate-genesis-plants.py`)

```python
#!/usr/bin/env python3
import json
from eth_utils import keccak

def generate_storage_slots():
    storage = {}

    # Celestia metadata
    storage["0x00"] = "0x616e6465706c616e74735f763100000000000000000000000000"  # Namespace
    storage["0x01"] = "0x0000000000000000000000000000000000000000000000000000000012d687"  # Height start

    # Plant seeds (520 plants)
    for i in range(520):
        seed = keccak(text=f"Plant{i}:Genesis:ANDE")
        storage[hex(0x100 + i)] = "0x" + seed.hex()

    return storage

# Update genesis.json
with open("specs/genesis.json", "r") as f:
    genesis = json.load(f)

genesis["alloc"]["0x0000000000000000000000000000000000000001"]["storage"] = generate_storage_slots()

with open("specs/genesis.json", "w") as f:
    json.dump(genesis, f, indent=2)
```

### Ejecutar

```bash
python3 scripts/generate-genesis-plants.py
cargo run -p ande-node
```

## 📊 Comparación de Métodos

| Método | Datos on-chain | Costo | Escalabilidad | Estándar |
|--------|---------------|-------|---------------|----------|
| **Pure on-chain** | 100% | $10K-$50K | ❌ | ✅ |
| **Híbrido Celestia** | Seeds (16 KB) | $1.5K-$3K | ✅ | ✅ |
| **Pure off-chain** | Pointers (1 KB) | $100 | ✅ | ⚠️ |

## ✅ Recomendación

**Usar método híbrido:**
1. Seeds (32 bytes × 520) en genesis storage slots
2. Metadata extendida (100 KB × 520 = 70 MB) en Celestia DA
3. Pointers de Celestia (namespace + heights) en genesis

**Beneficios:**
- Sigue estándares de Reth/Ethereum
- Genesis file pequeño y manejable
- Datos verificables on-chain
- Escalable con Celestia DA
- 85% más económico que pure on-chain

## 🔐 Verificación

```rust
// Leer seed de planta desde genesis
let plant_id: u64 = 1;
let storage_slot = U256::from(0x100 + plant_id);
let seed = storage.get(contract_address, storage_slot);

// Recuperar metadata desde Celestia
let celestia_namespace = storage.get(contract_address, U256::from(0x00));
let blob_height = storage.get(contract_address, U256::from(0x01));
let metadata = celestia_client.get_blob(blob_height, celestia_namespace, commitment);
```

## 📝 Notas

- **Archivo genesis.json es la fuente de verdad**
- Node carga genesis automáticamente al iniciar
- Storage slots se indexan desde 0x00
- Seeds usan slots 0x100-0x307 (520 plants)
- Celestia metadata usa slots 0x00-0x0F
- Sin nombres específicos en contratos (genérico)
- Siguiendo 100% estándares de Reth

