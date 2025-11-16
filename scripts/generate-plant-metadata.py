#!/usr/bin/env python3
"""
ANDE Chain - Plant Metadata Generator for Celestia Upload

Genera metadata completa de plantas en formato JSON compatible con celestia-uploader.

Input:  specs/generated/seeds_manifest.json
Output: specs/generated/plants_metadata.json (formato PlantMetadata)

Usage:
    python3 scripts/generate-plant-metadata.py
"""

import json
import random
from pathlib import Path
from datetime import datetime

# =============================================================================
# CONFIGURACIÓN
# =============================================================================

PROJECT_ROOT = Path(__file__).parent.parent
SEEDS_MANIFEST = PROJECT_ROOT / "specs" / "generated" / "seeds_manifest.json"
OUTPUT_FILE = PROJECT_ROOT / "specs" / "generated" / "plants_metadata.json"

# Propiedades de plantas (para 520 plantas genéricas)
PROPERTIES_TEMPLATES = [
    {"medicinal": True, "psychoactive": False, "toxic": False, "endangered": False, "ceremonial": False, "cultivated": True},
    {"medicinal": True, "psychoactive": True, "toxic": False, "endangered": False, "ceremonial": True, "cultivated": False},
    {"medicinal": True, "psychoactive": True, "toxic": True, "endangered": True, "ceremonial": True, "cultivated": False},
    {"medicinal": True, "psychoactive": False, "toxic": False, "endangered": True, "ceremonial": False, "cultivated": False},
]

CONSERVATION_STATUS = ["LC", "NT", "VU", "EN", "CR", "EW"]
RARITY_TIERS = [1, 2, 3, 4, 5]

COMPOUND_TEMPLATES = [
    ["Alkaloids", "Flavonoids"],
    ["Terpenoids", "Phenolics"],
    ["Glycosides", "Saponins"],
    ["Essential oils", "Tannins"],
]

REGIONS_TEMPLATES = [
    ["Andes Mountains", "Peru", "Bolivia"],
    ["Amazon Rainforest", "Brazil", "Colombia"],
    ["Patagonia", "Chile", "Argentina"],
    ["Central Andes", "Ecuador", "Peru"],
]

USES_TEMPLATES = [
    ["Respiratory conditions", "Anti-inflammatory"],
    ["Digestive health", "Pain relief"],
    ["Spiritual ceremonies", "Healing rituals"],
    ["Fever reduction", "Wound healing"],
]

# =============================================================================
# GENERACIÓN DE TRAITS
# =============================================================================

def generate_traits(seed: str) -> dict:
    """
    Genera traits determinísticos basados en el seed.

    Usa el seed como fuente de randomness para generar valores consistentes.
    """
    # Usar seed como semilla para random (determinístico)
    random.seed(int(seed, 16) % (2**32))

    return {
        "height": random.randint(50, 200),
        "color": random.randint(0, 255),
        "potency": random.randint(30, 100),
        "resistance": random.randint(40, 100),
        "yield_trait": random.randint(20, 90),
        "aroma": random.randint(10, 100),
        "flower": random.randint(20, 100),
        "root": random.randint(30, 100),
    }

def generate_properties(plant_id: int) -> dict:
    """Genera propiedades de la planta"""
    return PROPERTIES_TEMPLATES[plant_id % len(PROPERTIES_TEMPLATES)]

def generate_metadata(plant_id: int, scientific_name: str) -> dict:
    """Genera metadata extendida de la planta"""
    idx = plant_id % len(COMPOUND_TEMPLATES)

    return {
        "description": f"Medicinal plant species from Andean region. {scientific_name} has been used traditionally for healing purposes.",
        "uses": USES_TEMPLATES[idx],
        "compounds": COMPOUND_TEMPLATES[idx],
        "regions": REGIONS_TEMPLATES[idx],
        "traditional_uses": [
            f"Traditional medicine practice #{plant_id}",
            "Ancestral healing ceremonies"
        ]
    }

# =============================================================================
# CONVERSIÓN A FORMATO PlantMetadata
# =============================================================================

def convert_to_plant_metadata(seeds_data: dict) -> list:
    """
    Convierte seeds_manifest.json al formato PlantMetadata completo.

    Estructura esperada por celestia-uploader:
    {
        "id": u16,
        "seed": String,
        "scientific_name": String,
        "common_names": Vec<String>,
        "genus": String,
        "family": String,
        "ncbi_code": String,
        "conservation": String,
        "rarity": u8,
        "properties": {...},
        "traits": {...},
        "metadata": {...}
    }
    """
    plants = []

    for plant_data in seeds_data['plants']:
        plant_id = plant_data['id']
        seed = plant_data['seed']

        # Generar NCBI code ficticio
        ncbi_code = f"NC{plant_id:06d}"

        # Seleccionar conservation status y rarity
        conservation = CONSERVATION_STATUS[plant_id % len(CONSERVATION_STATUS)]
        rarity = RARITY_TIERS[(plant_id - 1) % len(RARITY_TIERS)]

        # Generar traits determinísticos
        traits = generate_traits(seed)

        # Generar properties
        properties = generate_properties(plant_id)

        # Generar metadata
        metadata = generate_metadata(plant_id, plant_data['scientific_name'])

        plant_metadata = {
            "id": plant_id,
            "seed": seed,
            "scientific_name": plant_data['scientific_name'],
            "common_names": plant_data['common_names'],
            "genus": plant_data['genus'],
            "family": plant_data['family'],
            "ncbi_code": ncbi_code,
            "conservation": conservation,
            "rarity": rarity,
            "properties": properties,
            "traits": traits,
            "metadata": metadata
        }

        plants.append(plant_metadata)

    return plants

# =============================================================================
# MAIN
# =============================================================================

def main():
    print("=" * 60)
    print(" ANDE Chain - Plant Metadata Generator")
    print("=" * 60)
    print()

    # 1. Cargar seeds manifest
    print(f"📖 Cargando {SEEDS_MANIFEST}...")

    if not SEEDS_MANIFEST.exists():
        print(f"❌ Archivo no encontrado: {SEEDS_MANIFEST}")
        print("⚠️  Ejecutar primero: python3 scripts/generate-genesis-seeds.py")
        exit(1)

    with open(SEEDS_MANIFEST, 'r') as f:
        seeds_data = json.load(f)

    total_plants = seeds_data['total_plants']
    print(f"✅ Cargadas {total_plants} plantas")

    # 2. Convertir a formato PlantMetadata
    print(f"\n🌿 Generando metadata completa...")
    plants_metadata = convert_to_plant_metadata(seeds_data)
    print(f"✅ Metadata generada para {len(plants_metadata)} plantas")

    # 3. Calcular tamaño estimado
    metadata_json = json.dumps(plants_metadata, indent=2)
    size_bytes = len(metadata_json.encode('utf-8'))
    size_mb = size_bytes / (1024 * 1024)

    print(f"\n📊 Estadísticas:")
    print(f"  • Total plantas: {len(plants_metadata)}")
    print(f"  • Tamaño total: {size_mb:.2f} MB ({size_bytes:,} bytes)")
    print(f"  • Tamaño promedio/planta: {size_bytes // len(plants_metadata):,} bytes")

    # 4. Guardar
    print(f"\n💾 Guardando en {OUTPUT_FILE}...")
    with open(OUTPUT_FILE, 'w') as f:
        json.dump(plants_metadata, f, indent=2)

    print(f"✅ Metadata guardada exitosamente")

    # 5. Estimación de blobs para Celestia
    PLANTS_PER_BLOB = 5
    MAX_BLOB_SIZE = 500_000  # 500 KB

    total_blobs = (len(plants_metadata) + PLANTS_PER_BLOB - 1) // PLANTS_PER_BLOB
    estimated_size_per_blob = size_bytes / total_blobs

    print(f"\n📡 Estimación para Celestia Upload:")
    print(f"  • Plantas por blob: {PLANTS_PER_BLOB}")
    print(f"  • Total blobs necesarios: {total_blobs}")
    print(f"  • Tamaño promedio/blob: {estimated_size_per_blob / 1024:.2f} KB")

    if estimated_size_per_blob > MAX_BLOB_SIZE:
        print(f"  ⚠️  WARNING: Blobs exceden 500 KB recomendado")
        print(f"  💡 Considerar reducir PLANTS_PER_BLOB")
    else:
        print(f"  ✅ Tamaño de blobs dentro del límite recomendado")

    # 6. Estimación de costos
    GAS_PER_BLOB = 4_161_000
    FEE_PER_BLOB_TIA = 0.016644

    total_gas = total_blobs * GAS_PER_BLOB
    total_fee_tia = total_blobs * FEE_PER_BLOB_TIA
    total_fee_usd = total_fee_tia * 5  # Asumiendo $5/TIA

    print(f"\n💰 Estimación de Costos:")
    print(f"  • Gas total: {total_gas:,}")
    print(f"  • Fee total: {total_fee_tia:.4f} TIA")
    print(f"  • Costo estimado: ${total_fee_usd:.2f} USD")

    print()
    print("=" * 60)
    print(" ✨ Generación completada")
    print("=" * 60)
    print()
    print("Próximo paso:")
    print("  cargo run --bin upload-genesis -- \\")
    print(f"    --plants {OUTPUT_FILE} \\")
    print("    --config config.json \\")
    print("    --dry-run")
    print()

if __name__ == "__main__":
    main()
