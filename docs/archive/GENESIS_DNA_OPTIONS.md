# ANDE Chain - Genesis DNA Implementation Options
## Análisis de Casos de Éxito + Propuesta de Arquitectura

**Fecha:** 2024-11-15
**Status:** 🟡 DECISIÓN PENDIENTE

---

## 📊 Casos de Éxito Investigados

### 1. CryptoKitties (2017 - Pioneer)

**Arquitectura Genética:**
- **Genome Size**: 256 bits (uint256)
- **Traits**: 48 características (cattributes)
- **Gene System**: D (Dominant), R1 (Recessive), R2 (Minor Recessive)
- **Inheritance**:
  - D: 37.5% chance de heredar
  - R1: 9.375% chance
  - R2: 3.125% chance
- **Mutation**: Solo genes recesivos pueden mutar (10% chance)

**Storage:**
- ❌ Metadata OFF-CHAIN (centralizado - si cierran el sitio, pierdes las imágenes)
- ✅ Genome ON-CHAIN (256-bit number)
- ⚠️ **Problema**: Dependencia del frontend para visualización

**Contratos:**
```
KittyCore (main)
├─ KittyBase
├─ KittyOwnership (ERC-721)
├─ KittyBreeding
└─ KittyAuction
```

**Lecciones:**
- ✅ Sistema de genes D/R1/R2 funciona muy bien
- ✅ 256-bit genome es eficiente para almacenar traits
- ❌ Metadata off-chain es riesgoso a largo plazo
- ✅ Breeding cooldown previene spam

---

### 2. Axie Infinity (2018 - Most Successful)

**Arquitectura Genética:**
- **Body Parts**: 6 partes (eyes, ears, mouth, horn, back, tail)
- **Genes per Part**: 3 (D, R1, R2) = 18 genes totales
- **Classes**: 9 classes (beast, plant, aquatic, etc.)
- **Mutation**: 10% chance en genes recesivos

**Breeding:**
- **Cost**: 0.5 AXS + variable SLP (aumenta con cada breed)
- **Max Breeds**: 7 veces por Axie
- **Cooldown**: 5 días para madurez
- **Restrictions**: No siblings, no parent-child

**Economics:**
- ✅ Dual-token economics (AXS governance + SLP utility)
- ✅ Breeding costs escalan para prevenir hiperinflación
- ✅ Max breeding cap mantiene rareza

**Lecciones:**
- ✅ Dual-token economics es sostenible
- ✅ Breeding limits mantienen valor
- ✅ Madurez gradual crea engagement

---

### 3. Bitcoin Genesis Block (2009 - Hidden Messages)

**Easter Egg Legendario:**
```
"The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"
```

**Características:**
- Mensaje oculto en coinbase transaction
- Inmutable forever (bloque 0)
- Verificable por cualquiera
- Se convirtió en parte de la cultura crypto

**Otros Genesis Famosos:**
- **50 BTC reward**: Unspendable (permanece en genesis address)
- **5-day gap**: Mysterio de 5 días antes de bloque 1
- **Unusual hash**: 10 ceros leading (muy raro)

**Lecciones:**
- ✅ Mensajes en genesis se vuelven legendarios
- ✅ Inmutabilidad desde bloque 0 es poderosa
- ✅ Easter eggs generan comunidad y cultura

---

### 4. Modern Best Practices (2024)

**Storage Híbrido:**
```
Genesis Block
└─ DNA Seeds (on-chain) ─┐
                          ├─> Smart Contract (on-chain)
IPFS                      │   └─ Ownership + Logic
└─ Full Metadata ─────────┘
```

**Ventajas:**
- ✅ **Ownership on-chain**: Verificable, transferible, inmutable
- ✅ **Seeds on-chain**: Genesis DNA garantizado desde bloque 0
- ✅ **Metadata IPFS**: Costo reducido, escalable
- ✅ **Hybrid**: Balance perfecto entre descentralización y costo

**Desventajas:**
- ⚠️ **IPFS depende de pinning**: Necesita infraestructura
- ⚠️ **No 100% on-chain**: Metadata puede perderse si IPFS falla

---

## 🧬 Nuestros Datos: 520+ Plantas Psicoactivas

Del archivo `specs/plantas.md`:

**Géneros Destacados:**
- Acacia (9 especies)
- Brugmansia (8 especies - plantas andinas)
- Cannabis (4 variedades)
- Datura (26 especies)
- Psilocybe (11 especies)
- Virola (5 especies)
- ... y 500+ más

**Potencial:**
- ✅ **Datos científicos reales**: Nombres latinos, propiedades documentadas
- ✅ **Biodiversidad única**: Plantas andinas y amazónicas
- ✅ **Easter eggs educativos**: Cada planta enseña etnobotánica
- ✅ **Rareza científica**: Basada en conservation status real (IUCN)

---

## 🎯 OPCIÓN 1: Minimalista "CryptoKitties Style"

### Arquitectura

**Genesis Storage (Bloque 0):**
```solidity
Address 0x0000000000000000000000000000000000000001 (DNA Vault)
│
├─ Slot 0x00: "Sonk'o wachary" (Metadata - "nacimiento del corazón")
├─ Slot 0x01: "NCBI.nlm.nih.gov" (Source)
│
├─ Slot 0x02: Plant Seed 1 (keccak256 hash)
├─ Slot 0x03: Plant Seed 2
├─ Slot 0x04: Plant Seed 3
│  ...
└─ Slot 0x20B: Plant Seed 520 (0x02 + 520 = 0x20A)
```

**¿Qué se almacena?**
- Solo **seeds criptográficos** (hashes de 32 bytes)
- Cada seed puede generar genoma completo de manera determinística
- NO almacena nombres, NO almacena metadata

**Genome Generation:**
```solidity
function generateGenome(uint8 speciesId, address minter, uint256 timestamp)
    internal
    view
    returns (uint256 genome)
{
    bytes32 genesisSeed = GENESIS_SEEDS[speciesId]; // From storage slot

    genome = uint256(keccak256(abi.encodePacked(
        genesisSeed,
        minter,
        timestamp,
        block.difficulty
    )));
}
```

**Breeding:**
```solidity
function breed(uint256 parentA, uint256 parentB)
    external
    returns (uint256 offspring)
{
    // Dominant/Recessive logic (CryptoKitties style)
    uint256 genomeA = plants[parentA].genome;
    uint256 genomeB = plants[parentB].genome;

    uint256 childGenome = mixGenes(genomeA, genomeB);
    // 10% mutation chance on recessive genes
    childGenome = applyMutations(childGenome);

    return _mint(msg.sender, childGenome);
}
```

### Ventajas ✅
- **Gas ultra-eficiente**: Solo hashes de 32 bytes
- **Escalable**: 520 seeds = 16,640 bytes (~16 KB en genesis)
- **Determinístico**: Cualquiera puede verificar seeds
- **Simple**: Fácil de implementar y auditar

### Desventajas ❌
- **Metadata off-chain**: Dependencia de backend/IPFS para nombres
- **No easter eggs visuales**: Seeds son solo números
- **Menos engagement**: Los seeds no son humanos-legibles

### Costo Estimado
- **Genesis**: ~520 storage slots = ~100M gas (one-time)
- **Mint**: ~200K gas por planta
- **Breed**: ~250K gas por breeding

---

## 🎯 OPCIÓN 2: Híbrido "Modern Best Practices"

### Arquitectura

**Genesis Storage (Bloque 0):**
```solidity
Address 0x0000000000000000000000000000000000000001 (DNA Vault)
│
├─ Slot 0x00: "Sonk'o wachary" (Easter egg cultural)
├─ Slot 0x01: "NCBI.nlm.nih.gov" (Source authority)
├─ Slot 0x02: Registry Root Hash (Merkle root de 520 plantas)
│
├─ Slot 0x10: Plant 1 Seed + Metadata Pointer
│   ├─ Bytes 0-31: keccak256(seed)
│   └─ Bytes 32-63: IPFS CID hash
│
├─ Slot 0x11: Plant 2 Seed + Metadata Pointer
│  ...
└─ Slot 0x218: Plant 520 Seed + Metadata Pointer
```

**¿Qué se almacena?**
- **Seeds** (32 bytes) - On-chain
- **IPFS CID pointers** (32 bytes) - On-chain
- **Metadata completa** (nombres, propiedades, imágenes) - IPFS

**IPFS Structure:**
```json
// ipfs://QmXxx.../plant-001.json
{
  "speciesId": 1,
  "scientificName": "Lophophora williamsii",
  "commonNames": ["Peyote", "Hikuri"],
  "genus": "Lophophora",
  "family": "Cactaceae",
  "ncbiCode": "NC_xxxxx",
  "properties": {
    "psychoactive": true,
    "medicinal": true,
    "conservation": "Vulnerable",
    "habitat": "Chihuahuan Desert"
  },
  "traits": {
    "heightGene": 120,
    "colorGene": 85,
    "potencyGene": 250,
    "resistanceGene": 180
  },
  "image": "ipfs://QmYyyy.../plant-001.svg",
  "animation": "ipfs://QmZzzz.../plant-001-anim.mp4"
}
```

**Smart Contract:**
```solidity
contract AndePlants is ERC721 {
    struct PlantGenome {
        uint256 dna;           // 256-bit genome
        uint8 speciesId;       // 1-520
        uint8 generation;      // 0=genesis, 1+=bred
        bytes32 ipfsHash;      // Metadata CID
    }

    mapping(uint256 => PlantGenome) public plants;

    function tokenURI(uint256 tokenId)
        public
        view
        returns (string memory)
    {
        PlantGenome memory plant = plants[tokenId];

        // Return IPFS URL
        return string(abi.encodePacked(
            "ipfs://",
            ipfsHashToString(plant.ipfsHash)
        ));
    }
}
```

### Ventajas ✅
- **Balance perfecto**: Seeds on-chain + metadata IPFS
- **Costo razonable**: ~32 KB on-chain vs 500+ MB off-chain
- **Verificable**: Merkle root permite probar metadata
- **Escalable**: Fácil agregar más plantas sin hard fork
- **Rich metadata**: JSON completo con todas las propiedades

### Desventajas ❌
- **Dependencia IPFS**: Necesita infraestructura de pinning
- **No 100% on-chain**: Metadata puede perderse teóricamente
- **Complejidad media**: Requiere backend IPFS management

### Costo Estimado
- **Genesis**: ~520 double-slots = ~200M gas (one-time)
- **IPFS Pinning**: ~$50/month (Pinata/Infura)
- **Mint**: ~220K gas por planta
- **Breed**: ~270K gas por breeding

---

## 🎯 OPCIÓN 3: Maximalista "Immutable Forever"

### Arquitectura

**Genesis Storage (Bloque 0):**
```solidity
Address 0x0000000000000000000000000000000000000001 (DNA Vault)
│
├─ Slot 0x00: "Sonk'o wachary" (Easter egg)
├─ Slot 0x01: "NCBI.nlm.nih.gov" (Source)
│
├─ Slot 0x10-0x15: Plant 1 (Full Data - 6 slots)
│   ├─ 0x10: Seed (32 bytes)
│   ├─ 0x11: Scientific Name (packed string)
│   ├─ 0x12: Common Names (packed string)
│   ├─ 0x13: Properties (bit-packed)
│   ├─ 0x14: NCBI Code (packed string)
│   └─ 0x15: Conservation + Habitat (packed)
│
├─ Slot 0x16-0x1B: Plant 2 (Full Data - 6 slots)
│  ...
└─ Slot 0xC8C-0xC91: Plant 520 (Full Data - 6 slots)
```

**¿Qué se almacena?**
- **EVERYTHING** on-chain
- Seeds, nombres científicos, nombres comunes, propiedades, códigos NCBI
- SVG generation on-chain (como Loot NFT)
- Zero dependencias externas

**On-Chain SVG Generation:**
```solidity
function tokenURI(uint256 tokenId)
    public
    view
    returns (string memory)
{
    PlantData memory plant = getPlantData(tokenId);

    // Generate SVG on-chain
    string memory svg = string(abi.encodePacked(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600">',
        '<rect fill="#', colorFromGene(plant.colorGene), '"/>',
        '<text x="20" y="30">', plant.scientificName, '</text>',
        '<text x="20" y="60">', plant.commonNames, '</text>',
        generatePlantVisual(plant.genome),
        '</svg>'
    ));

    string memory json = Base64.encode(bytes(string(abi.encodePacked(
        '{"name":"', plant.scientificName, '",',
        '"image":"data:image/svg+xml;base64,', Base64.encode(bytes(svg)), '"',
        '}'
    ))));

    return string(abi.encodePacked('data:application/json;base64,', json));
}
```

### Ventajas ✅
- **100% on-chain**: Inmutable FOREVER
- **Zero dependencias**: No IPFS, no backend, no nada
- **Legendary status**: Como Loot NFT (100% on-chain = premium)
- **SVG dinámico**: Genera visuales on-chain
- **Cultural impact**: "Todo en genesis" es poderoso mensaje

### Desventajas ❌
- **Gas EXTREMO**: 520 plantas × 6 slots = 3,120 slots
- **Costo genesis**: ~600M gas (~$10,000-$50,000 USD depending on gas)
- **Storage caro**: Cada plant requiere múltiples slots
- **Complejidad alta**: SVG on-chain es complejo
- **No multimedia**: No videos, no audio (solo SVG)

### Costo Estimado
- **Genesis**: ~3,120 storage slots = ~600M gas (⚠️ VERY EXPENSIVE)
- **Mint**: ~280K gas por planta
- **Breed**: ~320K gas por breeding
- **One-time cost**: $10,000-$50,000 USD (but forever immutable)

---

## 📊 Comparación de Opciones

| Feature | Opción 1 (Minimal) | Opción 2 (Hybrid) | Opción 3 (Maximal) |
|---------|-------------------|-------------------|-------------------|
| **Genesis Cost** | ~100M gas | ~200M gas | ~600M gas |
| **Genesis USD** | ~$2K-$10K | ~$4K-$20K | ~$10K-$50K |
| **On-Chain Data** | Solo seeds | Seeds + IPFS CIDs | TODO |
| **IPFS Dependency** | ❌ Sí | ⚠️ Sí (con backup) | ✅ No |
| **Immutability** | ⚠️ Partial | ⚠️ Partial | ✅ Total |
| **Metadata Rich** | ❌ No | ✅ Sí | ✅ Sí |
| **SVG On-Chain** | ❌ No | ❌ No | ✅ Sí |
| **Escalabilidad** | ✅ Excellent | ✅ Good | ⚠️ Limited |
| **Complejidad** | ⚠️ Low | ⚠️ Medium | ❌ High |
| **Cultural Impact** | ⚠️ Medium | ✅ Good | ✅ Legendary |
| **Mantenimiento** | ⚠️ Backend needed | ⚠️ IPFS pinning | ✅ Zero |

---

## 🎯 RECOMENDACIÓN BASADA EN CASOS DE ÉXITO

### Opción 2.5: "Híbrido Progresivo" (LO MEJOR DE TODO)

**Fase 1: Genesis (Ahora)**
```solidity
// Almacenar solo lo CRÍTICO en genesis block 0
Address 0x0000000000000000000000000000000000000001
├─ Slot 0x00: "Sonk'o wachary" (Cultural easter egg)
├─ Slot 0x01: "NCBI.nlm.nih.gov" (Authority)
├─ Slot 0x02: Merkle Root (520 plantas)
├─ Slot 0x03-0x0A: Top 8 Genesis Plants (full data on-chain)
│   ├─ Cannabis sativa (icónica)
│   ├─ Lophophora williamsii (peyote)
│   ├─ Banisteriopsis caapi (ayahuasca)
│   ├─ Erythroxylum coca (coca)
│   ├─ Salvia divinorum (salvia)
│   ├─ Psilocybe cubensis (hongos)
│   ├─ Brugmansia arborea (floripondio - ANDINA ✅)
│   └─ Anadenanthera peregrina (yopo - ANDINA ✅)
│
└─ Slot 0x10-0x218: Remaining 512 Seeds + IPFS Pointers
```

**¿Por qué esta opción?**

1. **Top 8 Legendary**: Full data on-chain = collectors premium
2. **512 Common**: Seeds + IPFS = escalable y económico
3. **Merkle Root**: Permite probar autenticidad de las 520
4. **Cultural Impact**: "8 plantas sagradas en genesis" = narrative poderosa
5. **Costo Razonable**: ~250M gas (~$5K-$25K one-time)

**Evolutionary Path:**
```
Block 0 (Genesis)
└─ 8 Legendary (100% on-chain)
└─ 512 Seeds (hybrid)

Block 100,000 (Upgrade 1)
└─ Migrate top 50 plants to on-chain
└─ SVG generation for legendaries

Block 500,000 (Upgrade 2)
└─ Full on-chain migration (optional)
└─ Community vote: keep hybrid vs go full on-chain
```

---

## 🧬 Implementación Técnica Recomendada

### Genesis Contract (Precompiled at 0x...001)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract AndeDNAVault {
    // ========== EASTER EGGS ==========
    bytes32 public constant EASTER_EGG_1 =
        bytes32("Sonk'o wachary"); // "Nacimiento del corazón"

    bytes32 public constant EASTER_EGG_2 =
        bytes32("NCBI.nlm.nih.gov"); // Scientific authority

    // ========== MERKLE ROOT ==========
    bytes32 public immutable PLANTS_MERKLE_ROOT;

    // ========== LEGENDARY PLANTS (8 full on-chain) ==========
    struct LegendaryPlant {
        bytes32 seed;
        bytes32 scientificName;  // "Cannabis sativa"
        bytes32 commonName;      // "Marijuana"
        uint8 rarity;            // 0-255
        uint8 conservationStatus; // IUCN code
        bytes32 ncbiCode;        // "NC_xxxxx"
        bytes32 properties;      // Bit-packed traits
    }

    mapping(uint8 => LegendaryPlant) public legendaries; // IDs 1-8

    // ========== COMMON PLANTS (512 seeds + IPFS) ==========
    struct CommonPlant {
        bytes32 seed;
        bytes32 ipfsHash; // CIDv1
    }

    mapping(uint16 => CommonPlant) public commons; // IDs 9-520

    // ========== CONSTRUCTOR (Called at genesis) ==========
    constructor(
        bytes32 _merkleRoot,
        LegendaryPlant[8] memory _legendaries,
        CommonPlant[] memory _commons
    ) {
        PLANTS_MERKLE_ROOT = _merkleRoot;

        // Store 8 legendaries
        for (uint8 i = 0; i < 8; i++) {
            legendaries[i + 1] = _legendaries[i];
        }

        // Store 512 commons
        for (uint16 i = 0; i < 512; i++) {
            commons[i + 9] = _commons[i];
        }
    }

    // ========== VERIFICATION ==========
    function verifyPlant(
        uint16 speciesId,
        bytes32[] calldata merkleProof
    ) external view returns (bool) {
        bytes32 leaf;

        if (speciesId <= 8) {
            // Legendary - verify against stored data
            leaf = keccak256(abi.encode(legendaries[uint8(speciesId)]));
        } else {
            // Common - verify against stored seed
            leaf = keccak256(abi.encode(commons[speciesId]));
        }

        return MerkleProof.verify(merkleProof, PLANTS_MERKLE_ROOT, leaf);
    }

    // ========== GETTERS ==========
    function getSeed(uint16 speciesId)
        external
        view
        returns (bytes32)
    {
        if (speciesId <= 8) {
            return legendaries[uint8(speciesId)].seed;
        } else {
            return commons[speciesId].seed;
        }
    }

    function isLegendary(uint16 speciesId)
        external
        pure
        returns (bool)
    {
        return speciesId <= 8;
    }
}
```

---

## 🚀 Roadmap de Implementación

### Week 1: Genesis Preparation
- [ ] Seleccionar las 8 plantas legendarias
- [ ] Obtener códigos NCBI verificados
- [ ] Generar 520 seeds criptográficos
- [ ] Crear Merkle tree de todas las plantas
- [ ] Preparar metadata IPFS (backup)

### Week 2: Smart Contracts
- [ ] AndeDNAVault (genesis precompile)
- [ ] AndePlants (ERC-721 main)
- [ ] AndeBreeding (genetics engine)
- [ ] Deploy to testnet

### Week 3: Genesis Deployment
- [ ] Deploy AndeDNAVault to mainnet genesis
- [ ] Verify all 520 plants accessible
- [ ] Test seed generation
- [ ] Audit contracts

### Week 4: NFT Launch
- [ ] Deploy AndePlants (ERC-721)
- [ ] Genesis mint (primeros 1000 NFTs)
- [ ] Community verification event
- [ ] Explorer integration

---

## 💡 Decisión Final

Te recomiendo **Opción 2.5 (Híbrido Progresivo)** porque:

1. **Costo razonable**: ~$5K-$25K one-time (aceptable para producción)
2. **Cultural impact**: 8 plantas sagradas 100% on-chain = legendary
3. **Escalable**: 512 plantas hybrid = crecimiento futuro
4. **Verificable**: Merkle root permite probar todo desde genesis
5. **Flexible**: Podemos migrar más plantas on-chain gradualmente
6. **Basado en casos de éxito**: Combina lo mejor de CryptoKitties + Axie + Loot

**¿Cuál prefieres?**

A) Opción 1 - Minimalista (solo seeds)
B) Opción 2 - Híbrido estándar (seeds + IPFS para todo)
C) Opción 2.5 - Híbrido progresivo (8 legendaries + 512 hybrid) ⭐ RECOMMENDED
D) Opción 3 - Maximalista (todo on-chain - muy caro)

---

**Creado con ❤️ para ANDE Chain**
**"De la tierra al bloque genesis - preservando sabiduría ancestral"**
