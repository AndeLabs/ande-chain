#!/usr/bin/env python3
"""
ANDE Chain - Genesis Plant Data Generator

Genera seeds criptográficos y datos empaquetados para 520 plantas
medicinales psicoactivas a ser incluidas en genesis block 0.

Usage:
    python3 scripts/generate-genesis-plants.py

Output:
    - genesis_plants.json (storage layout para genesis.json)
    - plants_manifest.json (human-readable manifest)
    - merkle_proofs.json (proofs para verificación)
"""

import json
import hashlib
import zlib
from typing import Dict, List, Tuple
from dataclasses import dataclass, asdict
from pathlib import Path

try:
    from eth_utils import keccak, to_hex
    from eth_abi import encode
except ImportError:
    print("❌ Required packages not found")
    print("📦 Install with: pip install eth-utils eth-abi")
    exit(1)

# ============================================
# CONFIGURATION
# ============================================

PROJECT_ROOT = Path(__file__).parent.parent
PLANTAS_MD = PROJECT_ROOT / "specs" / "plantas.md"
OUTPUT_DIR = PROJECT_ROOT / "specs" / "generated"
OUTPUT_DIR.mkdir(exist_ok=True)

# ============================================
# DATA STRUCTURES
# ============================================

@dataclass
class PlantData:
    """Estructura de datos para cada planta"""
    plant_id: int
    scientific_name: str
    common_names: List[str]
    genus: str
    family: str
    ncbi_code: str
    properties: List[str]
    conservation_status: str  # IUCN: EX, EW, CR, EN, VU, NT, LC, DD, NE
    rarity_tier: int  # 0=Common, 1=Uncommon, 2=Rare, 3=Legendary

    def generate_seed(self) -> bytes:
        """Genera seed criptográfico único para esta planta"""
        data = f"{self.scientific_name}:{self.ncbi_code}:ANDE:Genesis:2024:Block0"
        return keccak(text=data)

    def generate_traits(self) -> List[int]:
        """Genera 8 traits genéticos (0-255) derivados del seed"""
        seed = self.generate_seed()
        traits = []

        for i in range(8):
            # Usar diferentes partes del seed para cada trait
            trait_seed = keccak(seed + bytes([i]))
            trait_value = int.from_bytes(trait_seed[:1], 'big')
            traits.append(trait_value)

        return traits

    def pack_slot1(self) -> bytes:
        """Slot 1: Seed (32 bytes)"""
        return self.generate_seed()

    def pack_slot2(self) -> bytes:
        """Slot 2: Scientific name (max 32 chars, UTF-8)"""
        name_bytes = self.scientific_name.encode('utf-8')[:32]
        return name_bytes.ljust(32, b'\x00')

    def pack_slot3(self) -> bytes:
        """
        Slot 3: Packed data (32 bytes)
        - Bytes 0-7:   NCBI code (8 chars)
        - Bytes 8-11:  Common names hash (4 bytes)
        - Bytes 12-19: 8 traits (uint8 cada uno)
        - Bytes 20-23: Properties bitfield
        - Byte 24:     Rarity tier
        - Byte 25:     Conservation status
        - Bytes 26-27: Reserved
        - Bytes 28-31: CRC32 checksum
        """
        packed = bytearray(32)

        # Bytes 0-7: NCBI code
        ncbi_bytes = self.ncbi_code.encode('utf-8')[:8]
        packed[0:8] = ncbi_bytes.ljust(8, b'\x00')

        # Bytes 8-11: Hash de common names
        common_text = ','.join(self.common_names)
        common_hash = keccak(text=common_text)[:4]
        packed[8:12] = common_hash

        # Bytes 12-19: 8 genetic traits
        traits = self.generate_traits()
        for i, trait in enumerate(traits):
            packed[12 + i] = trait

        # Bytes 20-23: Properties bitfield (32 bits)
        properties_bits = self._encode_properties()
        packed[20:24] = properties_bits.to_bytes(4, 'big')

        # Byte 24: Rarity tier
        packed[24] = self.rarity_tier

        # Byte 25: Conservation status
        packed[25] = self._encode_conservation()

        # Bytes 26-27: Reserved for future use
        packed[26:28] = b'\x00\x00'

        # Bytes 28-31: CRC32 checksum of bytes 0-27
        checksum = zlib.crc32(packed[:28]) & 0xFFFFFFFF
        packed[28:32] = checksum.to_bytes(4, 'big')

        return bytes(packed)

    def pack_slot4(self) -> bytes:
        """
        Slot 4: Metadata hash + Template ID
        - Byte 0: SVG template ID (0-9)
        - Bytes 1-31: Metadata hash (IPFS backup o deterministic hash)
        """
        # Generar metadata JSON
        metadata = {
            'scientificName': self.scientific_name,
            'commonNames': self.common_names,
            'genus': self.genus,
            'family': self.family,
            'properties': self.properties
        }
        metadata_json = json.dumps(metadata, sort_keys=True)
        metadata_hash = keccak(text=metadata_json)

        # Combinar con template ID
        packed = bytearray(metadata_hash)
        packed[0] = self._get_template_id()

        return bytes(packed)

    def _encode_properties(self) -> int:
        """Encode properties como bitfield (32 bits)"""
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
            'neuroprotective': 15,
            'traditional': 16,
            'visionary': 17,
            'ceremonial': 18,
            'sacred': 19,
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
        """Determina SVG template basado en genus"""
        template_map = {
            # Cactus template
            'Lophophora': 0,
            'Trichocereus': 0,
            'Echinopsis': 0,
            'Carnegia': 0,
            'Ariocarpus': 0,
            'Epithelantha': 0,

            # Tree/bush template
            'Cannabis': 1,
            'Banisteriopsis': 1,
            'Erythroxylum': 1,
            'Mitragyna': 1,

            # Mushroom template
            'Psilocybe': 2,
            'Amanita': 2,
            'Panaeolus': 2,
            'Conocybe': 2,

            # Flower template (Solanaceae)
            'Brugmansia': 3,
            'Datura': 3,
            'Solandra': 3,
            'Brunfelsia': 3,

            # Vine template
            'Ipomoea': 4,
            'Argyreia': 4,
            'Turbina': 4,

            # Grass template
            'Phalaris': 5,

            # Shrub template
            'Salvia': 6,
            'Heimia': 6,

            # Aquatic template
            'Nymphaea': 7,

            # Succulent template
            'Mesembryanthemum': 8,
        }
        return template_map.get(self.genus, 9)  # 9 = generic template

    def to_storage_slots(self) -> Dict[str, str]:
        """Convierte a formato storage para genesis.json"""
        # Calcular base slot (0x10 + (id-1)*4)
        base_slot = 0x10 + (self.plant_id - 1) * 4

        return {
            hex(base_slot + 0): to_hex(self.pack_slot1()),
            hex(base_slot + 1): to_hex(self.pack_slot2()),
            hex(base_slot + 2): to_hex(self.pack_slot3()),
            hex(base_slot + 3): to_hex(self.pack_slot4())
        }

# ============================================
# LEGENDARY PLANTS DATABASE
# ============================================

LEGENDARY_PLANTS = [
    PlantData(
        plant_id=1,
        scientific_name="Lophophora williamsii",
        common_names=["Peyote", "Hikuri", "Mescal Button"],
        genus="Lophophora",
        family="Cactaceae",
        ncbi_code="NC030453",
        properties=["psychoactive", "entheogenic", "hallucinogenic", "medicinal", "ceremonial"],
        conservation_status="VU",
        rarity_tier=3
    ),
    PlantData(
        plant_id=2,
        scientific_name="Cannabis sativa",
        common_names=["Marijuana", "Hemp", "Cannabis", "Ganja"],
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
        common_names=["Ayahuasca", "Yage", "Caapi", "Vine of the Soul"],
        genus="Banisteriopsis",
        family="Malpighiaceae",
        ncbi_code="NC051087",
        properties=["psychoactive", "entheogenic", "visionary", "medicinal", "ceremonial"],
        conservation_status="LC",
        rarity_tier=3
    ),
    PlantData(
        plant_id=4,
        scientific_name="Erythroxylum coca",
        common_names=["Coca", "Hoja de Coca", "Mama Coca"],
        genus="Erythroxylum",
        family="Erythroxylaceae",
        ncbi_code="NC040990",
        properties=["stimulant", "medicinal", "traditional", "sacred"],
        conservation_status="LC",
        rarity_tier=3
    ),
    PlantData(
        plant_id=5,
        scientific_name="Psilocybe cubensis",
        common_names=["Magic Mushroom", "Golden Teacher", "Teonanacatl"],
        genus="Psilocybe",
        family="Hymenogastraceae",
        ncbi_code="JAAVMF01",
        properties=["psychoactive", "entheogenic", "hallucinogenic", "ceremonial"],
        conservation_status="LC",
        rarity_tier=2
    ),
    PlantData(
        plant_id=6,
        scientific_name="Salvia divinorum",
        common_names=["Salvia", "Ska Maria Pastora", "Diviner's Sage"],
        genus="Salvia",
        family="Lamiaceae",
        ncbi_code="MH286334",
        properties=["psychoactive", "dissociative", "visionary", "ceremonial"],
        conservation_status="EN",
        rarity_tier=3
    ),
    PlantData(
        plant_id=7,
        scientific_name="Brugmansia arborea",
        common_names=["Floripondio", "Angel's Trumpet", "Borrachero"],
        genus="Brugmansia",
        family="Solanaceae",
        ncbi_code="NC",
        properties=["psychoactive", "toxic", "entheogenic", "medicinal", "ceremonial"],
        conservation_status="VU",
        rarity_tier=3
    ),
    PlantData(
        plant_id=8,
        scientific_name="Anadenanthera peregrina",
        common_names=["Yopo", "Cohoba", "Vilca", "Cebil"],
        genus="Anadenanthera",
        family="Fabaceae",
        ncbi_code="NC",
        properties=["psychoactive", "entheogenic", "hallucinogenic", "traditional", "ceremonial"],
        conservation_status="LC",
        rarity_tier=3
    ),
]

# ============================================
# MERKLE TREE
# ============================================

def calculate_merkle_root(leaves: List[bytes]) -> bytes:
    """Calcula Merkle root de plant seeds"""
    if len(leaves) == 0:
        return b'\x00' * 32
    if len(leaves) == 1:
        return leaves[0]

    # Build tree level by level
    current_level = leaves[:]

    while len(current_level) > 1:
        next_level = []
        for i in range(0, len(current_level), 2):
            if i + 1 < len(current_level):
                # Hash pair
                combined = keccak(current_level[i] + current_level[i + 1])
            else:
                # Odd one out - promote to next level
                combined = current_level[i]
            next_level.append(combined)
        current_level = next_level

    return current_level[0]

def generate_merkle_proof(leaves: List[bytes], index: int) -> List[str]:
    """Genera Merkle proof para planta en index"""
    if index >= len(leaves):
        return []

    proof = []
    current_index = index
    current_level = leaves[:]

    while len(current_level) > 1:
        next_level = []
        for i in range(0, len(current_level), 2):
            if i + 1 < len(current_level):
                left = current_level[i]
                right = current_level[i + 1]

                # Record sibling in proof
                if current_index == i:
                    proof.append(to_hex(right))
                elif current_index == i + 1:
                    proof.append(to_hex(left))

                # Hash pair
                combined = keccak(left + right)
            else:
                combined = current_level[i]

            next_level.append(combined)

        # Update index for next level
        current_index = current_index // 2
        current_level = next_level

    return proof

# ============================================
# GENESIS GENERATION
# ============================================

def generate_genesis_storage(plants: List[PlantData]) -> Dict[str, str]:
    """Genera complete storage layout para genesis"""

    storage = {}

    # ========== HEADER SLOTS ==========

    # Slot 0x0: Easter egg cultural
    storage["0x0"] = to_hex(b"Sonk'o wachary".ljust(32, b'\x00'))

    # Slot 0x1: Scientific authority
    storage["0x1"] = to_hex(b"NCBI.nlm.nih.gov".ljust(32, b'\x00'))

    # Slot 0x2: Total plant count
    storage["0x2"] = to_hex((len(plants)).to_bytes(32, 'big'))

    # Slot 0x3: Merkle root
    plant_seeds = [plant.generate_seed() for plant in plants]
    merkle_root = calculate_merkle_root(plant_seeds)
    storage["0x3"] = to_hex(merkle_root)

    # Slot 0x4: Genesis timestamp (Nov 15, 2024)
    import time
    genesis_time = int(time.time())
    storage["0x4"] = to_hex(genesis_time.to_bytes(32, 'big'))

    # Slot 0x5: Protocol version + flags
    version = 1
    flags = 0  # Reserved for future use
    version_data = (version << 8) | flags
    storage["0x5"] = to_hex(version_data.to_bytes(32, 'big'))

    # ========== PLANT DATA ==========

    for plant in plants:
        plant_slots = plant.to_storage_slots()
        storage.update(plant_slots)

    return storage

def export_genesis_json(plants: List[PlantData], output_file: Path):
    """Export a genesis.json format"""

    storage = generate_genesis_storage(plants)

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

    return genesis_alloc

def export_manifest(plants: List[PlantData], output_file: Path):
    """Export human-readable manifest"""

    manifest = {
        "version": "1.0.0",
        "totalPlants": len(plants),
        "generatedAt": int(time.time()),
        "merkleRoot": to_hex(calculate_merkle_root([p.generate_seed() for p in plants])),
        "plants": []
    }

    for plant in plants:
        plant_dict = asdict(plant)
        plant_dict['seed'] = to_hex(plant.generate_seed())
        plant_dict['traits'] = plant.generate_traits()
        manifest['plants'].append(plant_dict)

    with open(output_file, 'w') as f:
        json.dump(manifest, f, indent=2)

    print(f"✅ Manifest exported to {output_file}")

def export_merkle_proofs(plants: List[PlantData], output_file: Path):
    """Export Merkle proofs para verificación"""

    plant_seeds = [p.generate_seed() for p in plants]
    proofs = {}

    for i, plant in enumerate(plants):
        proof = generate_merkle_proof(plant_seeds, i)
        proofs[str(plant.plant_id)] = {
            "scientificName": plant.scientific_name,
            "seed": to_hex(plant.generate_seed()),
            "proof": proof
        }

    with open(output_file, 'w') as f:
        json.dump(proofs, f, indent=2)

    print(f"✅ Merkle proofs exported to {output_file}")

# ============================================
# MAIN
# ============================================

def main():
    print("=" * 60)
    print(" ANDE Chain - Genesis Plant Data Generator")
    print("=" * 60)
    print()

    # Use legendary plants for now (8 plants)
    plants = LEGENDARY_PLANTS

    print(f"📊 Processing {len(plants)} plants...")
    print()

    # Export genesis storage
    genesis_file = OUTPUT_DIR / "genesis_plants.json"
    export_genesis_json(plants, genesis_file)
    print()

    # Export manifest
    manifest_file = OUTPUT_DIR / "plants_manifest.json"
    export_manifest(plants, manifest_file)
    print()

    # Export Merkle proofs
    proofs_file = OUTPUT_DIR / "merkle_proofs.json"
    export_merkle_proofs(plants, proofs_file)
    print()

    # Statistics
    print("=" * 60)
    print(" Statistics")
    print("=" * 60)
    print(f"Total plants:        {len(plants)}")
    print(f"Storage slots used:  {6 + len(plants) * 4}")
    print(f"Legendary tier:      {sum(1 for p in plants if p.rarity_tier == 3)}")
    print(f"Rare tier:           {sum(1 for p in plants if p.rarity_tier == 2)}")
    print()

    # Gas estimation
    slots = 6 + len(plants) * 4
    gas_per_slot = 20_000
    total_gas = slots * gas_per_slot

    print("=" * 60)
    print(" Gas Cost Estimation")
    print("=" * 60)
    print(f"Total gas:           {total_gas:,}")
    print(f"At 20 gwei:          {total_gas * 20 / 1e9:.4f} ETH")
    print(f"At 50 gwei:          {total_gas * 50 / 1e9:.4f} ETH")
    print(f"At 100 gwei:         {total_gas * 100 / 1e9:.4f} ETH")
    print()

    print("✨ Generation complete!")
    print()
    print("Next steps:")
    print("1. Review generated files in specs/generated/")
    print("2. Update specs/genesis.json with genesis_plants.json")
    print("3. Deploy smart contracts")
    print("4. Test genesis loading in node")

if __name__ == "__main__":
    import time
    main()
