#!/usr/bin/env python3
"""
ANDE Chain - Genesis Seeds Generator

Genera seeds criptográficos para 520 plantas medicinales y actualiza specs/genesis.json
con los storage slots correspondientes.

Arquitectura:
  - Slots 0x00-0x0F: Referencias culturales (quechua)
  - Slots 0x10-0x1F: Metadata de Celestia DA
  - Slots 0x100-0x307: Seeds de 520 plantas (0x100 + 519 = 0x307)

Usage:
    python3 scripts/generate-genesis-seeds.py

Output:
    - specs/genesis.json (actualizado con seeds)
    - specs/generated/seeds_manifest.json (metadata legible)
"""

import json
import hashlib
from pathlib import Path
from typing import List, Dict
from datetime import datetime

try:
    from eth_utils import keccak
except ImportError:
    print("❌ eth-utils no instalado")
    print("📦 Instalar con: pip install eth-utils")
    exit(1)

# =============================================================================
# CONFIGURACIÓN
# =============================================================================

PROJECT_ROOT = Path(__file__).parent.parent
PLANTAS_SPEC = PROJECT_ROOT / "specs" / "plantas.md"
GENESIS_FILE = PROJECT_ROOT / "specs" / "genesis.json"
OUTPUT_DIR = PROJECT_ROOT / "specs" / "generated"
OUTPUT_DIR.mkdir(exist_ok=True)

GENESIS_CONTRACT = "0x0000000000000000000000000000000000000001"
SEED_START_SLOT = 0x100  # Slot inicial para seeds
TOTAL_PLANTS = 520

# =============================================================================
# PARSEO DE PLANTAS DESDE MARKDOWN
# =============================================================================

def parse_plantas_md() -> List[Dict[str, str]]:
    """Parse plantas.md y extrae las especies"""

    print(f"📖 Leyendo {PLANTAS_SPEC}...")

    if not PLANTAS_SPEC.exists():
        print(f"❌ Archivo no encontrado: {PLANTAS_SPEC}")
        exit(1)

    content = PLANTAS_SPEC.read_text()
    plants = []
    current_genus = None
    current_family = None

    for line in content.split('\n'):
        line = line.strip()

        # Detectar familia
        if line.startswith('##') and 'Family:' in line:
            current_family = line.split('Family:')[1].strip()
            continue

        # Detectar género
        if line.startswith('###'):
            current_genus = line.replace('###', '').strip()
            continue

        # Detectar especie (línea con •)
        if line.startswith('•'):
            # Extraer nombre científico (en cursiva *nombre*)
            if '*' in line:
                parts = line.split('*')
                if len(parts) >= 3:
                    scientific_name = parts[1].strip()

                    # Extraer nombres comunes (entre paréntesis)
                    common_names = []
                    if '(' in line and ')' in line:
                        common_part = line.split('(')[1].split(')')[0]
                        common_names = [n.strip() for n in common_part.split(',')]

                    plants.append({
                        'scientific_name': scientific_name,
                        'common_names': common_names,
                        'genus': current_genus or 'Unknown',
                        'family': current_family or 'Unknown'
                    })

    print(f"✅ Encontradas {len(plants)} especies")

    # Si no tenemos suficientes, generar plantas ficticias
    if len(plants) < TOTAL_PLANTS:
        print(f"⚠️  Solo se encontraron {len(plants)} plantas, generando {TOTAL_PLANTS - len(plants)} adicionales...")
        for i in range(len(plants), TOTAL_PLANTS):
            plants.append({
                'scientific_name': f'Planta medicinae sp. {i+1}',
                'common_names': [f'Medicinal Plant {i+1}'],
                'genus': 'Planta',
                'family': 'Medicinae'
            })

    return plants[:TOTAL_PLANTS]

# =============================================================================
# GENERACIÓN DE SEEDS
# =============================================================================

def generate_seed(plant: Dict[str, str], plant_id: int) -> str:
    """
    Genera un seed criptográfico único para una planta.

    Formula: keccak256(scientific_name + ":" + genus + ":ANDE:Genesis:2025:Block0:" + id)

    Returns:
        Hex string de 32 bytes (64 caracteres)
    """
    seed_input = (
        f"{plant['scientific_name']}:"
        f"{plant['genus']}:"
        f"ANDE:Genesis:2025:Block0:"
        f"{plant_id}"
    )

    seed_bytes = keccak(text=seed_input)
    return '0x' + seed_bytes.hex()

def generate_all_seeds(plants: List[Dict[str, str]]) -> Dict[str, Dict]:
    """
    Genera seeds para todas las plantas.

    Returns:
        Diccionario con storage slots {slot_hex: seed_hex}
    """
    print(f"\n🌱 Generando seeds para {len(plants)} plantas...")

    storage_slots = {}
    seeds_metadata = []

    for i, plant in enumerate(plants, start=1):
        plant_id = i
        seed = generate_seed(plant, plant_id)

        # Calcular slot (0x100 + index)
        slot = hex(SEED_START_SLOT + (i - 1))

        storage_slots[slot] = seed

        seeds_metadata.append({
            'id': plant_id,
            'slot': slot,
            'seed': seed,
            'scientific_name': plant['scientific_name'],
            'common_names': plant['common_names'],
            'genus': plant['genus'],
            'family': plant['family']
        })

        if i % 100 == 0:
            print(f"  ✓ {i}/{len(plants)} seeds generados")

    print(f"✅ {len(seeds_metadata)} seeds generados")

    return storage_slots, seeds_metadata

# =============================================================================
# ACTUALIZACIÓN DE GENESIS.JSON
# =============================================================================

def update_genesis_json(storage_slots: Dict[str, str]):
    """Actualiza specs/genesis.json con los seeds generados"""

    print(f"\n📝 Actualizando {GENESIS_FILE}...")

    if not GENESIS_FILE.exists():
        print(f"❌ Genesis file no encontrado: {GENESIS_FILE}")
        exit(1)

    # Cargar genesis actual
    with open(GENESIS_FILE, 'r') as f:
        genesis = json.load(f)

    # Verificar estructura
    if 'alloc' not in genesis:
        genesis['alloc'] = {}

    if GENESIS_CONTRACT not in genesis['alloc']:
        genesis['alloc'][GENESIS_CONTRACT] = {
            'balance': '0x0',
            'storage': {}
        }

    # Obtener storage actual (preservar cultural refs y Celestia metadata)
    current_storage = genesis['alloc'][GENESIS_CONTRACT].get('storage', {})

    # Contar slots existentes
    existing_cultural = sum(1 for k in current_storage.keys()
                           if int(k, 16) <= 0x0F)
    existing_celestia = sum(1 for k in current_storage.keys()
                           if 0x10 <= int(k, 16) <= 0x1F)

    print(f"  📚 Cultural refs existentes: {existing_cultural}")
    print(f"  📡 Celestia metadata existente: {existing_celestia}")

    # Agregar seeds (reemplazar cualquier seed anterior)
    print(f"  🌱 Agregando {len(storage_slots)} seeds...")

    for slot, seed in storage_slots.items():
        # Formatear slot con padding correcto (64 caracteres hex = 32 bytes)
        slot_int = int(slot, 16)
        slot_padded = f"0x{slot_int:064x}"
        current_storage[slot_padded] = seed

    # Actualizar genesis
    genesis['alloc'][GENESIS_CONTRACT]['storage'] = current_storage

    # Guardar
    with open(GENESIS_FILE, 'w') as f:
        json.dump(genesis, f, indent=2)

    total_slots = len(current_storage)
    print(f"✅ Genesis actualizado: {total_slots} storage slots totales")

# =============================================================================
# MANIFESTO DE SEEDS
# =============================================================================

def save_manifest(seeds_metadata: List[Dict]):
    """Guarda manifest legible con toda la metadata de seeds"""

    manifest_file = OUTPUT_DIR / "seeds_manifest.json"

    print(f"\n📄 Guardando manifest en {manifest_file}...")

    manifest = {
        'version': '1.0.0',
        'generated_at': datetime.utcnow().isoformat() + 'Z',
        'total_plants': len(seeds_metadata),
        'seed_start_slot': hex(SEED_START_SLOT),
        'seed_end_slot': hex(SEED_START_SLOT + len(seeds_metadata) - 1),
        'plants': seeds_metadata
    }

    with open(manifest_file, 'w') as f:
        json.dump(manifest, f, indent=2)

    print(f"✅ Manifest guardado: {len(seeds_metadata)} plantas")

# =============================================================================
# VERIFICACIÓN
# =============================================================================

def verify_genesis():
    """Verifica que el genesis.json esté correcto"""

    print(f"\n🔍 Verificando genesis...")

    with open(GENESIS_FILE, 'r') as f:
        genesis = json.load(f)

    storage = genesis['alloc'][GENESIS_CONTRACT]['storage']

    # Contar slots por tipo
    cultural = [k for k in storage.keys() if int(k, 16) <= 0x0F]
    celestia = [k for k in storage.keys() if 0x10 <= int(k, 16) <= 0x1F]
    seeds = [k for k in storage.keys() if int(k, 16) >= SEED_START_SLOT]

    print(f"  ✓ Slots culturales (0x00-0x0F): {len(cultural)}")
    print(f"  ✓ Slots Celestia (0x10-0x1F): {len(celestia)}")
    print(f"  ✓ Slots seeds (0x{SEED_START_SLOT:x}+): {len(seeds)}")
    print(f"  ✓ Total slots: {len(storage)}")

    # Verificar que todos los seeds estén presentes
    expected_seeds = TOTAL_PLANTS
    if len(seeds) != expected_seeds:
        print(f"⚠️  WARNING: Se esperaban {expected_seeds} seeds, encontrados {len(seeds)}")
    else:
        print(f"  ✅ Todos los {expected_seeds} seeds están presentes")

    # Verificar formato de seeds
    invalid_seeds = []
    for slot in seeds:
        seed_value = storage[slot]
        if not seed_value.startswith('0x') or len(seed_value) != 66:  # 0x + 64 hex chars
            invalid_seeds.append(slot)

    if invalid_seeds:
        print(f"  ⚠️  {len(invalid_seeds)} seeds con formato inválido")
    else:
        print(f"  ✅ Todos los seeds tienen formato válido (32 bytes)")

# =============================================================================
# MAIN
# =============================================================================

def main():
    print("=" * 60)
    print(" ANDE Chain - Genesis Seeds Generator")
    print("=" * 60)
    print()

    # 1. Parsear plantas
    plants = parse_plantas_md()

    # 2. Generar seeds
    storage_slots, seeds_metadata = generate_all_seeds(plants)

    # 3. Actualizar genesis.json
    update_genesis_json(storage_slots)

    # 4. Guardar manifest
    save_manifest(seeds_metadata)

    # 5. Verificar
    verify_genesis()

    print()
    print("=" * 60)
    print(" ✨ Generación completada exitosamente")
    print("=" * 60)
    print()
    print("Archivos generados:")
    print(f"  • {GENESIS_FILE}")
    print(f"  • {OUTPUT_DIR / 'seeds_manifest.json'}")
    print()
    print("Próximos pasos:")
    print("  1. Revisar specs/genesis.json")
    print("  2. Compilar node: cargo build -p ande-node")
    print("  3. Ejecutar node: cargo run -p ande-node")
    print()

if __name__ == "__main__":
    main()
