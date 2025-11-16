# Archived Documentation

Esta carpeta contiene documentación antigua que ya no representa el diseño actual de ANDE Chain.

## ⚠️ DEPRECATED - Solo para Referencia Histórica

Los archivos en esta carpeta fueron parte del proceso de diseño pero **NO deben usarse** para implementación actual.

### Archivos Archivados

1. **GENESIS_DNA_OPTIONS.md** (Nov 2024)
   - Opciones de diseño evaluadas inicialmente
   - Comparación: Minimalista vs Maximalista vs Híbrido
   - **Decisión final**: Híbrido con Celestia Matcha v6

2. **MAXIMAL_GENESIS_DESIGN.md** (Nov 2024)
   - Diseño maximalista 100% on-chain
   - 2,080 storage slots por planta
   - **Descartado** por costos ($10K-$50K)

3. **DIGITAL_DNA_ARCHITECTURE.md** (Nov 2024)
   - Arquitectura inicial de NFT gaming
   - Sistema de breeding y evolución
   - **Reemplazado** por diseño simplificado

## ✅ Documentación Actual

Ver `docs/README.md` para la documentación oficial y actualizada.

### Diseño Final Implementado

- **Seeds on-chain** (520 × 32 bytes = ~16 KB)
- **Metadata en Celestia DA** (140 blobs × 500 KB = ~70 MB)
- **Costo total**: ~$1.5K-$3K (85% ahorro vs maximalista)
- **Easter eggs culturales** en storage slots 0x00-0x0F

## 📅 Timeline

- **2024-11**: Investigación y diseño inicial
- **2025-11-15**: Decisión final e implementación
- **Actual**: Usar `docs/GENESIS_WORKFLOW.md`

---

**Nota**: Mantener estos archivos solo para entender el proceso de diseño.
No implementar código basado en estos documentos.
