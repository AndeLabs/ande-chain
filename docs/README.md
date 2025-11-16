# ANDE Chain Documentation

Official documentation for ANDE Chain - Sovereign Rollup with Reth.

## 📚 Active Documentation

### Genesis & Chain Initialization

- **[GENESIS_WORKFLOW.md](./GENESIS_WORKFLOW.md)** - ⭐ **START HERE**
  - Flujo estándar de Reth para crear genesis
  - Método híbrido: Seeds on-chain + Celestia DA
  - Comandos y configuración

- **[GENESIS_CULTURAL_REFERENCES.md](./GENESIS_CULTURAL_REFERENCES.md)**
  - Easter eggs culturales en el genesis
  - Términos quechua y su significado
  - Verificación on-chain

### Celestia Integration

- **[CELESTIA_INTEGRATION_ARCHITECTURE.md](./CELESTIA_INTEGRATION_ARCHITECTURE.md)**
  - Integración con Celestia Matcha v6
  - Arquitectura de blob uploader
  - Costos y especificaciones técnicas

## 🗂️ Archived Documentation

Documentación antigua movida a \`docs/archive/\`:

- \`GENESIS_DNA_OPTIONS.md\` - Opciones de diseño evaluadas (DEPRECATED)
- \`MAXIMAL_GENESIS_DESIGN.md\` - Diseño maximalista (DEPRECATED)
- \`DIGITAL_DNA_ARCHITECTURE.md\` - Arquitectura NFT inicial (DEPRECATED)

**⚠️ NO usar estos documentos - solo para referencia histórica**

## 🚀 Quick Start

### 1. Generar Genesis

\`\`\`bash
# Generar seeds de 520 plantas
python3 scripts/generate-genesis-seeds.py

# Verificar genesis.json
cat specs/genesis.json | jq '.alloc["0x0000000000000000000000000000000000000001"].storage | keys | length'
\`\`\`

### 2. Iniciar Node

\`\`\`bash
# Compilar
cargo build --release -p ande-node

# Ejecutar (carga specs/genesis.json automáticamente)
cargo run --release -p ande-node
\`\`\`

---

**Last Updated**: 2025-11-15
**Maintainer**: ANDE Labs Team
