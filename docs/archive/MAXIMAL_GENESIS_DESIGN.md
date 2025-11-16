# ANDE Chain - Maximal Genesis Design
## 520 Plantas Medicinales 100% On-Chain desde Block 0

**Fecha:** 2024-11-15
**Status:** 🟢 EN CONSTRUCCIÓN
**Filosofía:** "Si no podemos hacer esto, mejoramos la chain hasta que pueda"

---

## 🎯 Objetivo

Inscribir **520 plantas medicinales psicoactivas** con metadata completa, SVG on-chain, y verificación científica en el genesis block 0 de ANDE Chain.

**Esto será:**
- ✅ La base de datos botánica más grande on-chain en historia
- ✅ Prueba de potencia de ANDE Chain + Celestia DA
- ✅ Statement cultural: Preservación de conocimiento ancestral
- ✅ Legendary status: 100% immutable forever

---

## 📊 Storage Layout Optimization

### Challenge

520 plantas × 6 slots cada una = **3,120 storage slots** en genesis

**Problema de escala:**
```
Ethereum mainnet genesis: ~8,893 accounts
Bitcoin genesis: 1 coinbase tx
ANDE genesis target: 3,120+ storage slots en UN solo account
```

**Solución: Data Packing Ultra-Optimizado**

---

## 🗜️ Data Packing Strategy

### Cada Planta Requiere:

```
1. Seed (32 bytes) ────────────────────────> 1 slot
2. Scientific Name (variable, max 64 chars) > 2 slots
3. Common Names (variable, max 128 chars) ─> 4 slots
4. Genus/Family (variable, max 32 chars) ──> 1 slot
5. Properties (bit-packed) ────────────────> 1 slot
6. NCBI Code (8 chars) + Conservation ────> 1 slot (packed)
7. Traits (8 × uint8) ─────────────────────> 1 slot (packed)

Total per plant (naive): 11 slots
Total 520 plants: 5,720 slots ❌ TOO MUCH
```

### Optimized Packing:

```solidity
// ============================================
// OPTIMIZED: 4 SLOTS PER PLANT
// ============================================

struct PlantData {
    // SLOT 1: Seed + Rarity + Conservation
    bytes32 seed;                    // 256 bits

    // SLOT 2: Names (packed)
    bytes32 scientificName;          // Max 32 chars (Latin names are short)

    // SLOT 3: Properties + NCBI + Traits (packed)
    bytes32 packedData;
    // Breakdown:
    // - bytes 0-7:   NCBI code (8 chars)
    // - bytes 8-11:  Common name hash (4 bytes - reference to mapping)
    // - bytes 12-19: 8 traits (uint8 each)
    // - bytes 20-23: Properties bitfield
    // - bytes 24-27: Reserved
    // - bytes 28-31: Checksum

    // SLOT 4: Metadata pointer + Generation data
    bytes32 metadataHash;           // IPFS backup + SVG template ID
}

// Total: 4 slots × 520 plants = 2,080 slots ✅ ACCEPTABLE
```

---

## 🧬 Storage Layout en Genesis

```
Address: 0x0000000000000000000000000000000000000001
Contract: AndeDNAVault (Precompiled)

Storage Slots:
├─ 0x0000000000000000000000000000000000000000000000000000000000000000
│  └─ "Sonk'o wachary" (Easter egg - "Nacimiento del corazón" en Quechua)
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000001
│  └─ "NCBI.nlm.nih.gov" (Scientific authority)
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000002
│  └─ Total plant count: 520
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000003
│  └─ Merkle root (verification)
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000004
│  └─ Genesis timestamp
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000005
│  └─ Version + Protocol flags
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000010
│  └─ Plant 1 - Slot 1 (seed)
├─ 0x0000000000000000000000000000000000000000000000000000000000000011
│  └─ Plant 1 - Slot 2 (scientific name)
├─ 0x0000000000000000000000000000000000000000000000000000000000000012
│  └─ Plant 1 - Slot 3 (packed data)
├─ 0x0000000000000000000000000000000000000000000000000000000000000013
│  └─ Plant 1 - Slot 4 (metadata hash)
│
├─ 0x0000000000000000000000000000000000000000000000000000000000000014
│  └─ Plant 2 - Slot 1 (seed)
│  ...
│
└─ 0x0000000000000000000000000000000000000000000000000000000000000823
   └─ Plant 520 - Slot 4 (metadata hash)

Total slots used: 6 (header) + (520 × 4) = 2,086 slots
```

---

## 🎨 SVG On-Chain Generation

### Approach: Template-Based with Trait Substitution

```solidity
// ============================================
// SVG TEMPLATES (Stored in separate contract)
// ============================================

contract AndeSVGTemplates {
    // 10 base templates para diferentes tipos de plantas
    mapping(uint8 => string) public baseTemplates;

    constructor() {
        // Template 0: Cactus (Lophophora, Trichocereus, etc.)
        baseTemplates[0] = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600">'
            '<defs>'
                '<radialGradient id="cactusGrad">'
                    '<stop offset="0%" stop-color="#{COLOR1}"/>'
                    '<stop offset="100%" stop-color="#{COLOR2}"/>'
                '</radialGradient>'
            '</defs>'
            '<rect fill="#{BACKGROUND}" width="400" height="600"/>'
            '<ellipse cx="200" cy="400" rx="#{WIDTH}" ry="#{HEIGHT}" fill="url(#cactusGrad)"/>'
            '<text x="20" y="40" font-size="24" fill="white">#{NAME}</text>'
            '<text x="20" y="70" font-size="16" fill="#ccc">#{COMMON}</text>'
            '</svg>';

        // Template 1: Tree (Banisteriopsis, Cannabis, etc.)
        baseTemplates[1] = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600">'
            '<rect fill="#{BACKGROUND}" width="400" height="600"/>'
            '<rect x="180" y="300" width="40" height="200" fill="#{TRUNK_COLOR}"/>'
            '<circle cx="200" cy="250" r="#{CANOPY_SIZE}" fill="#{FOLIAGE_COLOR}"/>'
            '<text x="20" y="40" font-size="24" fill="white">#{NAME}</text>'
            '</svg>';

        // Template 2: Mushroom (Psilocybe, Amanita, etc.)
        baseTemplates[2] = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 600">'
            '<rect fill="#{BACKGROUND}" width="400" height="600"/>'
            '<rect x="180" y="300" width="40" height="150" fill="#{STIPE_COLOR}"/>'
            '<ellipse cx="200" cy="280" rx="#{CAP_WIDTH}" ry="#{CAP_HEIGHT}" fill="#{CAP_COLOR}"/>'
            '<text x="20" y="40" font-size="24" fill="white">#{NAME}</text>'
            '</svg>';

        // ... 7 more templates for different plant types
    }

    function getTemplate(uint8 templateId)
        external
        view
        returns (string memory)
    {
        return baseTemplates[templateId];
    }
}

// ============================================
// SVG GENERATOR (Main logic)
// ============================================

contract AndeSVGGenerator {
    AndeSVGTemplates public templates;
    AndeDNAVault public vault;

    function generateSVG(uint16 plantId)
        public
        view
        returns (string memory)
    {
        // Get plant data
        (
            bytes32 seed,
            bytes32 scientificName,
            bytes32 packedData,
            bytes32 metadataHash
        ) = vault.getPlant(plantId);

        // Extract traits from packed data
        uint8[8] memory traits = unpackTraits(packedData);

        // Determine template based on genus
        uint8 templateId = getTemplateForPlant(plantId);

        // Get base template
        string memory template = templates.getTemplate(templateId);

        // Replace placeholders
        template = replacePlaceholder(template, "#{NAME}", bytes32ToString(scientificName));
        template = replacePlaceholder(template, "#{COLOR1}", traitToColor(traits[1]));
        template = replacePlaceholder(template, "#{COLOR2}", traitToColor(traits[1] / 2));
        template = replacePlaceholder(template, "#{BACKGROUND}", getBackground(traits[0]));
        template = replacePlaceholder(template, "#{WIDTH}", uint2str(100 + traits[2]));
        template = replacePlaceholder(template, "#{HEIGHT}", uint2str(100 + traits[3]));

        return template;
    }

    // ============================================
    // HELPER FUNCTIONS
    // ============================================

    function traitToColor(uint8 trait)
        internal
        pure
        returns (string memory)
    {
        // Convert trait (0-255) to hex color
        bytes memory hexChars = "0123456789abcdef";
        bytes memory color = new bytes(6);

        // Generate color from trait using deterministic algorithm
        uint256 hue = (uint256(trait) * 360) / 256;
        uint256 r, g, b;

        if (hue < 60) {
            r = 255;
            g = (hue * 255) / 60;
            b = 0;
        } else if (hue < 120) {
            r = 255 - ((hue - 60) * 255) / 60;
            g = 255;
            b = 0;
        } // ... continue for all hues

        color[0] = hexChars[r / 16];
        color[1] = hexChars[r % 16];
        color[2] = hexChars[g / 16];
        color[3] = hexChars[g % 16];
        color[4] = hexChars[b / 16];
        color[5] = hexChars[b % 16];

        return string(color);
    }

    function unpackTraits(bytes32 packed)
        internal
        pure
        returns (uint8[8] memory traits)
    {
        // Extract 8 traits from bytes 12-19
        for (uint i = 0; i < 8; i++) {
            traits[i] = uint8(packed[12 + i]);
        }
    }

    function replacePlaceholder(
        string memory template,
        string memory placeholder,
        string memory value
    )
        internal
        pure
        returns (string memory)
    {
        // Simple string replacement
        // In production, use library like OpenZeppelin Strings
        bytes memory templateBytes = bytes(template);
        bytes memory placeholderBytes = bytes(placeholder);
        bytes memory valueBytes = bytes(value);

        // ... replacement logic
        return template; // Simplified
    }
}
```

---

## 💾 Genesis Generation Script

```python
#!/usr/bin/env python3
"""
ANDE Chain - Genesis Plant Data Generator

Generates cryptographic seeds and packed data for 520 plants
to be included in genesis block 0.
"""

import json
import hashlib
from typing import Dict, List, Tuple
from eth_utils import keccak, to_hex
import pandas as pd

# ============================================
# PLANT DATA STRUCTURE
# ============================================

class PlantData:
    def __init__(
        self,
        plant_id: int,
        scientific_name: str,
        common_names: List[str],
        genus: str,
        family: str,
        ncbi_code: str,
        properties: List[str],
        conservation_status: str,
        rarity_tier: int
    ):
        self.plant_id = plant_id
        self.scientific_name = scientific_name
        self.common_names = common_names
        self.genus = genus
        self.family = family
        self.ncbi_code = ncbi_code
        self.properties = properties
        self.conservation_status = conservation_status
        self.rarity_tier = rarity_tier

    def generate_seed(self) -> bytes:
        """Generate cryptographic seed for this plant"""
        data = f"{self.scientific_name}:{self.ncbi_code}:ANDE:Genesis:2024"
        return keccak(data.encode('utf-8'))

    def generate_traits(self) -> List[int]:
        """Generate 8 genetic traits (0-255) from seed"""
        seed = self.generate_seed()
        traits = []

        for i in range(8):
            # Use different parts of seed for each trait
            trait_seed = keccak(seed + bytes([i]))
            trait_value = int.from_bytes(trait_seed[:1], 'big')
            traits.append(trait_value)

        return traits

    def pack_slot2(self) -> bytes:
        """Pack scientific name into 32 bytes"""
        name_bytes = self.scientific_name.encode('utf-8')[:32]
        return name_bytes.ljust(32, b'\x00')

    def pack_slot3(self) -> bytes:
        """Pack: NCBI + CommonNameHash + Traits + Properties"""
        packed = bytearray(32)

        # Bytes 0-7: NCBI code (8 chars)
        ncbi_bytes = self.ncbi_code.encode('utf-8')[:8]
        packed[0:8] = ncbi_bytes.ljust(8, b'\x00')

        # Bytes 8-11: Common names hash (4 bytes)
        common_hash = keccak(','.join(self.common_names).encode('utf-8'))[:4]
        packed[8:12] = common_hash

        # Bytes 12-19: 8 traits (uint8 each)
        traits = self.generate_traits()
        for i, trait in enumerate(traits):
            packed[12 + i] = trait

        # Bytes 20-23: Properties bitfield
        properties_bits = self._encode_properties()
        packed[20:24] = properties_bits.to_bytes(4, 'big')

        # Bytes 24-27: Rarity + Conservation
        packed[24] = self.rarity_tier
        packed[25] = self._encode_conservation()
        packed[26:28] = b'\x00\x00'  # Reserved

        # Bytes 28-31: Checksum (CRC32 of first 28 bytes)
        checksum = self._calculate_checksum(packed[:28])
        packed[28:32] = checksum.to_bytes(4, 'big')

        return bytes(packed)

    def pack_slot4(self) -> bytes:
        """Pack metadata hash (IPFS CID backup + Template ID)"""
        # For now, generate deterministic hash
        # In production, this would be actual IPFS CID
        metadata = {
            'scientificName': self.scientific_name,
            'commonNames': self.common_names,
            'properties': self.properties
        }
        metadata_json = json.dumps(metadata, sort_keys=True)
        metadata_hash = keccak(metadata_json.encode('utf-8'))

        # Combine with template ID (based on genus)
        template_id = self._get_template_id()
        packed = bytearray(metadata_hash)
        packed[0] = template_id  # First byte is template ID

        return bytes(packed)

    def _encode_properties(self) -> int:
        """Encode properties as bitfield"""
        property_map = {
            'psychoactive': 0,
            'medicinal': 1,
            'toxic': 2,
            'entheogenic': 3,
            'stimulant': 4,
            'sedative': 5,
            'hallucinogenic': 6,
            'dissociative': 7,
            'empathogenic': 8,
            'analgesic': 9,
            'anti-inflammatory': 10,
            'antibacterial': 11,
            'antifungal': 12,
            'antiviral': 13,
            'antioxidant': 14,
            'neuroprotective': 15
        }

        bitfield = 0
        for prop in self.properties:
            if prop.lower() in property_map:
                bit_index = property_map[prop.lower()]
                bitfield |= (1 << bit_index)

        return bitfield

    def _encode_conservation(self) -> int:
        """Encode IUCN conservation status"""
        status_map = {
            'EX': 0,   # Extinct
            'EW': 1,   # Extinct in Wild
            'CR': 2,   # Critically Endangered
            'EN': 3,   # Endangered
            'VU': 4,   # Vulnerable
            'NT': 5,   # Near Threatened
            'LC': 6,   # Least Concern
            'DD': 7,   # Data Deficient
            'NE': 8    # Not Evaluated
        }
        return status_map.get(self.conservation_status, 8)

    def _get_template_id(self) -> int:
        """Determine SVG template based on genus"""
        template_map = {
            'Lophophora': 0,      # Cactus template
            'Trichocereus': 0,
            'Echinopsis': 0,
            'Carnegia': 0,

            'Cannabis': 1,        # Tree/bush template
            'Banisteriopsis': 1,
            'Erythroxylum': 1,

            'Psilocybe': 2,       # Mushroom template
            'Amanita': 2,
            'Panaeolus': 2,

            'Brugmansia': 3,      # Flower template
            'Datura': 3,

            # ... more mappings
        }
        return template_map.get(self.genus, 9)  # 9 = generic template

    def _calculate_checksum(self, data: bytes) -> int:
        """Calculate CRC32 checksum"""
        import zlib
        return zlib.crc32(data) & 0xffffffff

    def to_genesis_alloc(self) -> Dict:
        """Convert to genesis.json allocation format"""
        seed = self.generate_seed()
        slot2 = self.pack_slot2()
        slot3 = self.pack_slot3()
        slot4 = self.pack_slot4()

        # Calculate storage slot addresses
        base_slot = 0x10 + (self.plant_id - 1) * 4

        return {
            hex(base_slot + 0): to_hex(seed),
            hex(base_slot + 1): to_hex(slot2),
            hex(base_slot + 2): to_hex(slot3),
            hex(base_slot + 3): to_hex(slot4)
        }

# ============================================
# PLANT DATABASE (Top 20 examples)
# ============================================

LEGENDARY_PLANTS = [
    PlantData(
        plant_id=1,
        scientific_name="Lophophora williamsii",
        common_names=["Peyote", "Hikuri", "Mescal Button"],
        genus="Lophophora",
        family="Cactaceae",
        ncbi_code="NC030453",  # Real NCBI code
        properties=["psychoactive", "entheogenic", "hallucinogenic", "medicinal"],
        conservation_status="VU",
        rarity_tier=3  # 0=Common, 1=Uncommon, 2=Rare, 3=Legendary
    ),
    PlantData(
        plant_id=2,
        scientific_name="Cannabis sativa",
        common_names=["Marijuana", "Hemp", "Cannabis"],
        genus="Cannabis",
        family="Cannabaceae",
        ncbi_code="NC044371",
        properties=["psychoactive", "medicinal", "analgesic", "anti-inflammatory"],
        conservation_status="LC",
        rarity_tier=2
    ),
    PlantData(
        plant_id=3,
        scientific_name="Banisteriopsis caapi",
        common_names=["Ayahuasca", "Yage", "Caapi"],
        genus="Banisteriopsis",
        family="Malpighiaceae",
        ncbi_code="NC051087",
        properties=["psychoactive", "entheogenic", "visionary", "medicinal"],
        conservation_status="LC",
        rarity_tier=3
    ),
    PlantData(
        plant_id=4,
        scientific_name="Erythroxylum coca",
        common_names=["Coca", "Hoja de Coca"],
        genus="Erythroxylum",
        family="Erythroxylaceae",
        ncbi_code="NC040990",
        properties=["stimulant", "medicinal", "traditional"],
        conservation_status="LC",
        rarity_tier=3
    ),
    PlantData(
        plant_id=5,
        scientific_name="Psilocybe cubensis",
        common_names=["Magic Mushroom", "Golden Teacher"],
        genus="Psilocybe",
        family="Hymenogastraceae",
        ncbi_code="JAAVMF01",
        properties=["psychoactive", "entheogenic", "hallucinogenic"],
        conservation_status="LC",
        rarity_tier=2
    ),
    PlantData(
        plant_id=6,
        scientific_name="Salvia divinorum",
        common_names=["Salvia", "Ska Maria Pastora"],
        genus="Salvia",
        family="Lamiaceae",
        ncbi_code="MH286",  # Partial, needs verification
        properties=["psychoactive", "dissociative", "visionary"],
        conservation_status="EN",
        rarity_tier=3
    ),
    PlantData(
        plant_id=7,
        scientific_name="Brugmansia arborea",
        common_names=["Floripondio", "Angel's Trumpet"],
        genus="Brugmansia",
        family="Solanaceae",
        ncbi_code="NC",  # Needs NCBI lookup
        properties=["psychoactive", "toxic", "entheogenic", "medicinal"],
        conservation_status="VU",
        rarity_tier=3
    ),
    PlantData(
        plant_id=8,
        scientific_name="Anadenanthera peregrina",
        common_names=["Yopo", "Cohoba", "Vilca"],
        genus="Anadenanthera",
        family="Fabaceae",
        ncbi_code="NC",  # Needs NCBI lookup
        properties=["psychoactive", "entheogenic", "hallucinogenic", "traditional"],
        conservation_status="LC",
        rarity_tier=3
    ),
    # ... Continue with 512 more plants
]

# ============================================
# GENESIS GENERATOR
# ============================================

def generate_genesis_storage():
    """Generate complete storage layout for genesis"""

    storage = {}

    # Header slots
    storage["0x0"] = to_hex(b"Sonk'o wachary".ljust(32, b'\x00'))
    storage["0x1"] = to_hex(b"NCBI.nlm.nih.gov".ljust(32, b'\x00'))
    storage["0x2"] = to_hex((520).to_bytes(32, 'big'))

    # Calculate Merkle root
    plant_hashes = [plant.generate_seed() for plant in LEGENDARY_PLANTS]
    merkle_root = calculate_merkle_root(plant_hashes)
    storage["0x3"] = to_hex(merkle_root)

    # Genesis timestamp
    storage["0x4"] = to_hex((1700000000).to_bytes(32, 'big'))

    # Version
    storage["0x5"] = to_hex((1).to_bytes(32, 'big'))

    # Add all plants
    for plant in LEGENDARY_PLANTS:
        plant_storage = plant.to_genesis_alloc()
        storage.update(plant_storage)

    return storage

def calculate_merkle_root(leaves: List[bytes]) -> bytes:
    """Calculate Merkle root of plant seeds"""
    if len(leaves) == 0:
        return b'\x00' * 32
    if len(leaves) == 1:
        return leaves[0]

    # Build tree
    while len(leaves) > 1:
        next_level = []
        for i in range(0, len(leaves), 2):
            if i + 1 < len(leaves):
                combined = keccak(leaves[i] + leaves[i + 1])
            else:
                combined = leaves[i]
            next_level.append(combined)
        leaves = next_level

    return leaves[0]

def export_to_genesis_json(output_file: str):
    """Export to genesis.json format"""

    storage = generate_genesis_storage()

    genesis_alloc = {
        "0x0000000000000000000000000000000000000001": {
            "balance": "0x0",
            "storage": storage
        }
    }

    with open(output_file, 'w') as f:
        json.dump(genesis_alloc, f, indent=2)

    print(f"✅ Generated {len(storage)} storage slots")
    print(f"✅ Exported to {output_file}")

if __name__ == "__main__":
    export_to_genesis_json("genesis_plants.json")
```

---

## 📈 Gas Cost Estimation

```
Genesis Deployment Cost Breakdown:

Header (6 slots):
├─ SSTORE (new): 20,000 gas × 6 = 120,000 gas

Plant Data (520 plants × 4 slots):
├─ SSTORE (new): 20,000 gas × 2,080 = 41,600,000 gas

Total Genesis Storage: ~41,720,000 gas

At different gas prices:
├─ 20 gwei: 0.8344 ETH (~$2,500 USD)
├─ 50 gwei: 2.086 ETH (~$6,250 USD)
├─ 100 gwei: 4.172 ETH (~$12,500 USD)
└─ 200 gwei: 8.344 ETH (~$25,000 USD)

✅ ACCEPTABLE for legendary status
✅ One-time cost, immutable forever
✅ Proves ANDE Chain can handle large state
```

---

## 🚀 Implementation Plan

### Week 1: Data Preparation
- [ ] Parse 520 plants from `specs/plantas.md`
- [ ] Lookup/verify NCBI codes for each
- [ ] Classify by rarity tier (Legendary/Rare/Uncommon/Common)
- [ ] Generate cryptographic seeds
- [ ] Calculate Merkle root

### Week 2: Smart Contracts
- [ ] `AndeDNAVault.sol` (genesis precompile)
- [ ] `AndeSVGTemplates.sol` (10 templates)
- [ ] `AndeSVGGenerator.sol` (rendering logic)
- [ ] `AndePlants.sol` (ERC-721 NFT)
- [ ] Unit tests for all contracts

### Week 3: Genesis Integration
- [ ] Run Python generator script
- [ ] Update `specs/genesis.json`
- [ ] Update `infra/stacks/single-sequencer/genesis.json`
- [ ] Test genesis loading in node
- [ ] Verify all 520 plants accessible

### Week 4: Testing & Deployment
- [ ] Testnet deployment
- [ ] Gas optimization passes
- [ ] Load testing (can node handle 2080 slots?)
- [ ] Mainnet genesis deployment
- [ ] Public verification event

---

## ✅ Success Criteria

1. **All 520 plants accessible** from genesis block 0
2. **SVG generation works** for all plant types
3. **Gas costs acceptable** (< $30K USD total)
4. **Node handles load** without performance issues
5. **100% verifiable** via Merkle proofs
6. **Community celebrates** legendary status

---

## 🎯 Cultural Impact Target

**Headlines we're aiming for:**

> "ANDE Chain Inscribes Largest Botanical Database On-Chain"
> "520 Medicinal Plants Preserved Forever in Genesis Block"
> "Blockchain Meets Ethnobotany: ANDE's Cultural Statement"
> "First L2 to Store Complete Scientific Database in Genesis"

**Community narrative:**

"While other chains put JPEGs of apes on IPFS, ANDE Chain immortalized 520 medicinal plants with full scientific metadata directly in the genesis block. This is what building for humanity looks like."

---

**Status:** 🟢 Ready to implement
**Next:** Run genesis generator script + Update contracts

