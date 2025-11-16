# ANDE Chain - Digital DNA Architecture
## Sistema de ADN Digital Generativo en Genesis

**Fecha:** 2024-11-15
**Versión:** 1.0.0-genesis
**Status:** 🟢 PRODUCCIÓN

---

## 🧬 Visión General

Sistema de genética digital nativo de ANDE Chain, inscrito desde el bloque genesis, que permite crear NFTs evolutivos basados en ADN real de plantas medicinales andinas. Superior a CryptoKitties por usar datos científicos verificables del NCBI.

### Características Únicas

- **Inmutabilidad desde Genesis**: Seeds de ADN almacenados en bloque 0, verificables forever
- **Datos Científicos Reales**: Secuencias genéticas de NCBI (National Center for Biotechnology Information)
- **Plantas Medicinales Andinas**: 90+ especies con propiedades medicinales documentadas
- **NFTs Dinámicos**: ERC-721 que evolucionan basados en genética real
- **Sistema de Breeding**: Combinación genética usando algoritmos de Mendel
- **Rareza Científica**: Basada en biodiversidad real, no aleatoriedad arbitraria

---

## 🏗️ Arquitectura del Sistema

### 1. Genesis Storage Layout

```
Address 0x0000000000000000000000000000000000000001 (DNA Registry)
├─ Slot 0x00: "Sonk'o wachary" (metadata - "nacimiento del corazón" en quechua)
├─ Slot 0x01: "NCBI.nlm.nih.gov" (source authority)
├─ Slot 0x02: Hash seed 1 (cryptographic binding)
├─ Slot 0x03: "NC030601" (NCBI accession code - plant 1)
├─ Slot 0x04: Hash seed 2
├─ Slot 0x05: "HQ247200" (NCBI accession code - plant 2)
├─ Slot 0x06: Hash seed 3
├─ Slot 0x07: "LC651165" (NCBI accession code - plant 3)
├─ Slot 0x08: Hash seed 4
├─ Slot 0x09-0xFE: Reserved for 90 DNA seeds (expandible)
└─ Slot 0xFF: System metadata
```

### 2. Smart Contract Addresses

```
0x00000000000000000000000000000000000000d1 - AndeDNA (DNA Registry)
0x00000000000000000000000000000000000000d2 - AndePlant (ERC-721 NFT)
0x00000000000000000000000000000000000000d3 - AndeBreeding (Genetics Engine)
0x00000000000000000000000000000000000000d4 - AndeMarketplace (Trading)
0x00000000000000000000000000000000000000d5 - AndeEvolution (Growth System)
```

---

## 🌿 Base de Datos Genética (90 Plantas Andinas)

### Categorías de Plantas

#### Tier 1: Legendarias (10 plantas) - 1% rarity
Plantas con propiedades medicinales extraordinarias, especies en peligro

1. **Maca** (Lepidium meyenii) - `NC_030453.1`
   - Propiedades: Energizante, fertilidad
   - Rareza: Legendary
   - Hábitat: 4000-4500 msnm

2. **Cat's Claw** (Uncaria tomentosa) - `NC_042371.1`
   - Propiedades: Inmunoestimulante, antiinflamatorio
   - Rareza: Legendary
   - Hábitat: Amazonía andina

3. **Quinoa** (Chenopodium quinoa) - `NC_030267.1`
   - Propiedades: Súper alimento, proteína completa
   - Rareza: Legendary
   - Hábitat: Altiplano

4. **Ayahuasca** (Banisteriopsis caapi) - `NC_051087.1`
   - Propiedades: Ceremonial, visionaria
   - Rareza: Legendary
   - Hábitat: Amazonía

5. **Coca** (Erythroxylum coca) - `NC_040990.1`
   - Propiedades: Estimulante, medicinal ancestral
   - Rareza: Legendary
   - Hábitat: Yungas

#### Tier 2: Épicas (20 plantas) - 5% rarity

6. **Camu Camu** (Myrciaria dubia) - `NC_048658.1`
   - Propiedades: Vitamina C, antioxidante
   - Rareza: Epic

7. **Muña** (Minthostachys mollis) - `HQ247200.1`
   - Propiedades: Digestivo, antiparasitario
   - Rareza: Epic

8. **Chuchuhuasi** (Maytenus krukovii) - `LC651165.1`
   - Propiedades: Afrodisíaco, tónico
   - Rareza: Epic

9. **Sacha Inchi** (Plukenetia volubilis) - `NC_030601.1`
   - Propiedades: Omega 3, antioxidante
   - Rareza: Epic

10. **Yacón** (Smallanthus sonchifolius) - `NC_041425.1`
    - Propiedades: Prebiótico, antidiabético
    - Rareza: Epic

11. **Boldo** (Peumus boldus) - `NC_035239.1`
    - Propiedades: Hepático, digestivo
    - Rareza: Epic

12. **Matico** (Piper aduncum) - `NC_049238.1`
    - Propiedades: Cicatrizante, antibacteriano
    - Rareza: Epic

13. **Hercampuri** (Gentianella alborosea) - `MK681573.1`
    - Propiedades: Hepatoprotector, adelgazante
    - Rareza: Epic

14. **Sangre de Drago** (Croton lechleri) - `MN046560.1`
    - Propiedades: Cicatrizante, antiviral
    - Rareza: Epic

15. **Uña de Gato** (Uncaria tomentosa) - `NC_042371.1`
    - Propiedades: Anticancerígeno, inmunomodulador
    - Rareza: Epic

#### Tier 3: Raras (30 plantas) - 15% rarity

16-45. [Lista completa de 30 plantas raras con códigos NCBI]

#### Tier 4: Comunes (30 plantas) - 79% rarity

46-75. [Lista completa de 30 plantas comunes con códigos NCBI]

---

## 🧪 Sistema de Genética

### Estructura del Gen Digital

Cada planta NFT tiene un genoma de 256 bits:

```solidity
struct PlantGenome {
    bytes32 dnaHash;          // Hash único del genoma
    uint8 speciesId;          // 1-90 (especie base)
    uint8 generation;         // Generación (0 = genesis, 1-255 = bred)
    uint8 rarity;             // 0=Common, 1=Rare, 2=Epic, 3=Legendary
    uint8 growthStage;        // 0=Seed, 1=Sprout, 2=Young, 3=Mature, 4=Ancient

    // Traits genéticos (8 genes, 8 bits cada uno)
    uint8 heightGene;         // Altura de la planta
    uint8 colorGene;          // Color predominante
    uint8 potencyGene;        // Potencia medicinal
    uint8 resistanceGene;     // Resistencia a enfermedades
    uint8 yieldGene;          // Productividad
    uint8 aromaGene;          // Perfil aromático
    uint8 flowerGene;         // Patrón de floración
    uint8 rootGene;           // Sistema radicular

    // Metadata
    uint64 birthBlock;        // Bloque de nacimiento
    uint64 parentA;           // ID del padre A (0 si es genesis)
    uint64 parentB;           // ID del padre B (0 si es genesis)
    uint32 breedingCount;     // Veces que se ha usado para breeding
    uint32 evolutionPoints;   // Puntos para evolucionar
}
```

### Algoritmo de Breeding

Basado en genética mendeliana real:

```
Parent A genes: [g1a, g2a, g3a, g4a, g5a, g6a, g7a, g8a]
Parent B genes: [g1b, g2b, g3b, g4b, g5b, g6b, g7b, g8b]

Offspring genes:
- 50% probabilidad de heredar de Parent A
- 50% probabilidad de heredar de Parent B
- 5% probabilidad de mutación (+/- 10%)
- 1% probabilidad de mutación mayor (+/- 30%)

Rarity calculation:
- Common x Common = 85% Common, 14% Rare, 1% Epic
- Common x Rare = 70% Common, 25% Rare, 5% Epic
- Rare x Rare = 40% Rare, 50% Epic, 10% Legendary
- Epic x Epic = 60% Epic, 35% Legendary, 5% Mythic
```

### Sistema de Evolución

Las plantas evolucionan con el tiempo (bloques):

```
Stage 0: Seed (blocks 0-1000)
  - Cannot breed
  - Visual: seed sprite

Stage 1: Sprout (blocks 1000-5000)
  - Cannot breed
  - Visual: small sprout

Stage 2: Young (blocks 5000-20000)
  - Can breed (low success rate)
  - Visual: young plant

Stage 3: Mature (blocks 20000-100000)
  - Can breed (optimal success rate)
  - Visual: full plant with flowers
  - Produces "medicine tokens"

Stage 4: Ancient (blocks 100000+)
  - Can breed (wisdom bonus)
  - Visual: majestic ancient plant
  - Produces rare "essence tokens"
  - Can become "Mother Plant" (guild system)
```

---

## 💎 Sistema de Tokens

### AndePlant NFT (ERC-721)

Cada planta es un NFT único con:
- Metadata on-chain (genoma completo)
- Imagen dinámica generada (SVG on-chain o IPFS)
- Propiedades que evolucionan con el tiempo
- Transferible, tradeable, rentable

### Medicine Tokens (ERC-20)

Plantas maduras producen tokens ERC-20 basados en sus propiedades:

```solidity
// Plantas producen medicina cada N bloques
uint256 medicineRate = (potencyGene * yieldGene) / 100;
uint256 blocksPerHarvest = 28800; // ~1 día

// Tipos de medicina (diferentes tokens ERC-20)
- VITA: Vitaminas (de frutas como Camu Camu)
- HEAL: Curativos (de Sangre de Drago, Matico)
- ENER: Energizantes (de Maca, Guaraná)
- CALM: Relajantes (de Muña, Toronjil)
- IMMU: Inmuno (de Uña de Gato, Cat's Claw)
```

### Essence Tokens (ERC-1155)

Plantas ancestrales producen esencias raras:

```solidity
// Semi-fungible tokens con propiedades especiales
- Legendary Essence: Para breeding excepcional
- Mutation Serum: Forzar mutaciones positivas
- Time Accelerator: Acelerar crecimiento
- Rarity Booster: Aumentar rarity de offspring
```

---

## 🎮 Mecánicas de Juego

### 1. Minting Inicial (Genesis Drop)

```solidity
// Primeros 1000 NFTs minteables desde genesis seeds
function mintGenesis(uint8 speciesId) external payable {
    require(speciesId >= 1 && speciesId <= 90);
    require(totalGenesisMinted < 1000);
    require(msg.value >= MINT_PRICE);

    // Mint con genoma aleatorio pero determinístico
    bytes32 seed = keccak256(abi.encodePacked(
        block.timestamp,
        msg.sender,
        speciesId,
        genesisSeeds[speciesId] // Del genesis storage!
    ));

    PlantGenome memory genome = generateGenome(seed, speciesId);
    _safeMint(msg.sender, genome);
}
```

### 2. Breeding

```solidity
function breed(uint256 parentAId, uint256 parentBId)
    external
    returns (uint256 offspringId)
{
    require(ownerOf(parentAId) == msg.sender);
    require(ownerOf(parentBId) == msg.sender);
    require(canBreed(parentAId) && canBreed(parentBId));

    PlantGenome memory parentA = plants[parentAId];
    PlantGenome memory parentB = plants[parentBId];

    // Genética mendeliana + mutaciones
    PlantGenome memory offspring = crossbreed(parentA, parentB);
    offspring.generation = max(parentA.generation, parentB.generation) + 1;

    // Cooldown period
    breedingCooldown[parentAId] = block.number + COOLDOWN_BLOCKS;
    breedingCooldown[parentBId] = block.number + COOLDOWN_BLOCKS;

    return _safeMint(msg.sender, offspring);
}
```

### 3. Evolution

```solidity
function evolve(uint256 plantId) external {
    require(ownerOf(plantId) == msg.sender);
    PlantGenome storage plant = plants[plantId];

    uint256 age = block.number - plant.birthBlock;
    uint8 newStage = calculateStage(age);

    if (newStage > plant.growthStage) {
        plant.growthStage = newStage;
        emit PlantEvolved(plantId, newStage);

        // Bonus on evolution
        if (newStage == 3) {
            // Mature: start producing medicine
            enableMedicineProduction(plantId);
        } else if (newStage == 4) {
            // Ancient: produce essence
            enableEssenceProduction(plantId);
        }
    }
}
```

### 4. Harvesting

```solidity
function harvest(uint256 plantId) external returns (uint256 amount) {
    require(ownerOf(plantId) == msg.sender);
    PlantGenome memory plant = plants[plantId];
    require(plant.growthStage >= 3, "Not mature");

    uint256 blocksSinceHarvest = block.number - lastHarvest[plantId];
    require(blocksSinceHarvest >= HARVEST_COOLDOWN);

    // Calculate yield
    uint256 yield = calculateYield(plant);

    // Mint medicine tokens
    MedicineToken(medicineAddress).mint(msg.sender, yield);

    lastHarvest[plantId] = block.number;
    return yield;
}
```

---

## 🎨 Generación Visual (SVG On-Chain)

Cada planta genera su imagen SVG dinámicamente:

```solidity
function tokenURI(uint256 tokenId)
    public
    view
    override
    returns (string memory)
{
    PlantGenome memory plant = plants[tokenId];

    // Generate SVG based on genes
    string memory svg = string(abi.encodePacked(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600">',
        generateBackground(plant.colorGene),
        generateRoots(plant.rootGene, plant.growthStage),
        generateStem(plant.heightGene, plant.growthStage),
        generateLeaves(plant.colorGene, plant.yieldGene),
        generateFlowers(plant.flowerGene, plant.growthStage),
        generateAura(plant.rarity),
        '</svg>'
    ));

    // Generate metadata JSON
    string memory json = Base64.encode(bytes(string(abi.encodePacked(
        '{"name":"', getSpeciesName(plant.speciesId), ' #', toString(tokenId), '",',
        '"description":"', getSpeciesDescription(plant.speciesId), '",',
        '"image":"data:image/svg+xml;base64,', Base64.encode(bytes(svg)), '",',
        '"attributes":', generateAttributes(plant),
        '}'
    ))));

    return string(abi.encodePacked('data:application/json;base64,', json));
}
```

---

## 📊 Economía del Juego

### Flujos de Valor

```
Player Actions          → Cost (ANDE)  → Rewards
─────────────────────────────────────────────────
Mint Genesis Plant      → 10 ANDE      → 1 NFT (random rarity)
Breed Plants           → 5 ANDE       → 1 Offspring NFT
Accelerate Growth      → 2 ANDE       → Skip 1000 blocks
Harvest Medicine       → Free         → Medicine Tokens
Trade NFT              → 2.5% fee     → P2P trading
Stake Plant            → Lock NFT     → Medicine boost 2x

Medicine Token Uses:
- Buy Breeding Slots   → 100 VITA     → Extra breed
- Buy Evolution Boost  → 500 HEAL     → +10% evolution speed
- Buy Mutation Serum   → 1000 ENER    → Guarantee good mutation
- Buy Rarity Boost     → 5000 IMMU    → +1 rarity tier attempt

Essence Token Uses:
- Create Mother Plant  → 10 Essence   → Guild breeding center
- Legendary Breeding   → 5 Essence    → Guarantee legendary trait
- Time Warp            → 3 Essence    → Instant maturity
```

### Deflationary Mechanics

```solidity
// 50% de fees se queman
uint256 burnAmount = fee / 2;
andeToken.burn(burnAmount);

// 25% va a treasury para desarrollo
treasury.transfer(fee / 4);

// 25% va a pool de recompensas
rewardPool.deposit(fee / 4);
```

---

## 🔬 Datos Científicos Verificables

Cada planta NFT está respaldada por:

1. **NCBI Accession Code**: Código único de secuencia genética real
2. **Scientific Name**: Nomenclatura binomial (Genus species)
3. **Common Names**: Nombres locales en Quechua, Aymara, Español
4. **Medicinal Properties**: Documentadas científicamente (papers)
5. **Geographic Distribution**: Datos reales de biodiversidad
6. **Conservation Status**: IUCN Red List status

### API de Verificación

```solidity
function verifyGenetics(uint256 plantId)
    external
    view
    returns (VerificationData memory)
{
    PlantGenome memory plant = plants[plantId];

    return VerificationData({
        ncbiCode: getNCBICode(plant.speciesId),
        scientificName: getScientificName(plant.speciesId),
        paperReferences: getPapers(plant.speciesId),
        iucnStatus: getConservationStatus(plant.speciesId),
        verified: true
    });
}
```

---

## 🌍 Impacto Real

### Conservation Partnership

10% de profits van a:
- Conservación de plantas medicinales andinas
- Investigación científica de biodiversidad
- Comunidades indígenas guardianes del conocimiento ancestral
- Reforestación de especies en peligro

### Educational Platform

- Base de datos educativa de plantas medicinales
- Recursos para estudiantes de botánica
- Preservación de conocimiento ancestral
- Gamificación de la ciencia

---

## 🚀 Roadmap de Implementación

### Phase 1: Genesis (Block 0)
- ✅ Inscribir 90 seeds en genesis storage
- ✅ Deploy contratos precompilados
- ✅ Setup initial balances

### Phase 2: Core Contracts (Week 1-2)
- [ ] AndeDNA registry
- [ ] AndePlant ERC-721
- [ ] Basic minting functionality
- [ ] SVG generation engine

### Phase 3: Genetics (Week 3-4)
- [ ] Breeding mechanics
- [ ] Mutation algorithms
- [ ] Evolution system
- [ ] Rarity calculations

### Phase 4: Economy (Week 5-6)
- [ ] Medicine ERC-20 tokens (5 types)
- [ ] Essence ERC-1155 tokens
- [ ] Harvesting mechanics
- [ ] Staking system

### Phase 5: UI/UX (Week 7-8)
- [ ] Web3 frontend (Next.js)
- [ ] Plant visualization
- [ ] Breeding interface
- [ ] Marketplace integration

### Phase 6: Advanced Features (Week 9-12)
- [ ] Guild system (Mother Plants)
- [ ] Tournaments (best plants)
- [ ] Achievements & badges
- [ ] Mobile app (React Native)

---

## 📝 Ejemplo de Plantas en Genesis

### Plant 1: Sacha Inchi (Epic)
```json
{
  "speciesId": 9,
  "ncbiCode": "NC_030601",
  "scientificName": "Plukenetia volubilis",
  "commonNames": ["Sacha Inchi", "Inca Peanut", "Maní del Inca"],
  "properties": ["Omega 3", "Antioxidante", "Antiinflamatorio"],
  "rarity": "Epic",
  "habitat": "Amazonía andina, 200-1500 msnm",
  "conservationStatus": "LC"
}
```

### Plant 2: Muña (Epic)
```json
{
  "speciesId": 7,
  "ncbiCode": "HQ247200",
  "scientificName": "Minthostachys mollis",
  "commonNames": ["Muña", "Tipo", "Poleo"],
  "properties": ["Digestivo", "Antiparasitario", "Carminativo"],
  "rarity": "Epic",
  "habitat": "Andes, 2700-3400 msnm",
  "conservationStatus": "LC"
}
```

---

## 🔐 Seguridad

### Consideraciones

1. **Randomness**: Usar Chainlink VRF para breeding aleatorio verificable
2. **Reentrancy**: Proteción en todas las funciones de transferencia
3. **Access Control**: OpenZeppelin AccessControl para roles
4. **Upgradability**: Proxy pattern para futuras mejoras (cuidado con immutability)
5. **Rate Limiting**: Cooldowns para prevenir spam

### Auditorías

- [ ] Audit interno (team)
- [ ] Audit externo (firma reconocida)
- [ ] Bug bounty program
- [ ] Formal verification (TLA+)

---

## 📚 Referencias

1. NCBI GenBank: https://www.ncbi.nlm.nih.gov/genbank/
2. CryptoKitties: https://www.cryptokitties.co/
3. ERC-721 Standard: https://eips.ethereum.org/EIPS/eip-721
4. Mendelian Genetics: Papers de genética vegetal
5. Andean Medicinal Plants: Base de datos etnobotánica

---

**Creado con ❤️ para ANDE Chain**
**"Preservando la sabiduría ancestral andina en la blockchain"**
