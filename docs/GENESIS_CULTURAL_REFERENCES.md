# Genesis Cultural References - Conocimiento Ancestral

Referencias culturales andinas incluidas en el genesis block de ANDE Chain.

## 📜 Storage Slots 0x00 - 0x0F: Sabiduría Ancestral

Palabras quechua/aymara relacionadas con plantas medicinales y conocimiento ancestral:

### Slot 0x00: `Yachak`
```
0x59616368616b0000000000000000000000000000000000000000000000000000
```
**Significado**: Sabio, curandero, chamán
**Contexto**: Personas con conocimiento profundo de plantas medicinales

### Slot 0x01: `Hampi koka`
```
0x48616d7069206b6f6b610000000000000000000000000000000000000000000000
```
**Significado**: Planta medicinal sagrada
**Contexto**: Medicina ancestral, hoja de coca

### Slot 0x02: `Hampi kamayoc`
```
0x48616d7069206b616d61796f630000000000000000000000000000000000000000
```
**Significado**: Maestro de la medicina
**Contexto**: Sanador especializado en plantas

### Slot 0x03: `Kaya kaya`
```
0x4b617961206b6179610000000000000000000000000000000000000000000000
```
**Significado**: Planta amarga medicinal
**Contexto**: Plantas de poder para sanación

### Slot 0x04: `Quyllur`
```
0x5175796c6c75720000000000000000000000000000000000000000000000000000
```
**Significado**: Estrella
**Contexto**: Conexión cósmica con la naturaleza

### Slot 0x05: `Hatu munay`
```
0x48617475206d756e6179000000000000000000000000000000000000000000000
```
**Significado**: Gran amor/voluntad
**Contexto**: Respeto y amor por las plantas

### Slot 0x06: `Kawasay yachay`
```
0x4b61776173617920796163686179000000000000000000000000000000000000
```
**Significado**: Conocimiento de la vida
**Contexto**: Sabiduría sobre el ciclo de vida de las plantas

### Slot 0x07: `Pachamama`
```
0x50616368616d616d610000000000000000000000000000000000000000000000
```
**Significado**: Madre Tierra
**Contexto**: Tierra que provee todas las plantas medicinales

### Slot 0x08: `Sonk'o wachary`
```
0x536f6e6b276f2077616368617279000000000000000000000000000000000000
```
**Significado**: Corazón naciendo
**Contexto**: Nuevo comienzo, renacimiento

### Slot 0x09: `Apu yachay`
```
0x4170752079616368617900000000000000000000000000000000000000000000
```
**Significado**: Sabiduría del Apu (espíritu de la montaña)
**Contexto**: Conocimiento sagrado de las alturas

### Slot 0x0A: `Hampi yachay`
```
0x48616d7069207961636861790000000000000000000000000000000000000000
```
**Significado**: Conocimiento medicinal
**Contexto**: Ciencia ancestral de curación

### Slot 0x0B: `Allin curay`
```
0x416c6c696e2063757261790000000000000000000000000000000000000000000
```
**Significado**: Buena curación
**Contexto**: Sanación efectiva y respetuosa

### Slot 0x0C: `Miti miti`
```
0x4d697469206d697469000000000000000000000000000000000000000000000000
```
**Significado**: Crecimiento progresivo
**Contexto**: Desarrollo gradual de las plantas

### Slot 0x0D: `Taita`
```
0x54616974610000000000000000000000000000000000000000000000000000000
```
**Significado**: Padre, abuelo sabio
**Contexto**: Respeto a los ancianos con conocimiento

### Slot 0x0E: `Mama quchua`
```
0x4d616d61207175636875610000000000000000000000000000000000000000000
```
**Significado**: Madre lago
**Contexto**: Fuente de vida y plantas acuáticas

### Slot 0x0F: `Ande Labs`
```
0x416e6465204c616273000000000000000000000000000000000000000000000000
```
**Significado**: Laboratorios de los Andes
**Contexto**: Identidad del proyecto

## 📡 Storage Slots 0x10 - 0x1F: Celestia DA Metadata

### Slot 0x10: Namespace ID
```
0x616e6465706c616e74735f763100000000000000000000000000000000000000
```
**Valor**: `andeplants_v1` (29 bytes en hex)
**Uso**: Identificador de namespace en Celestia

### Slot 0x11: Start Height
```
0x0000000000000000000000000000000000000000000000000000000000000000
```
**Valor**: TBD (se llenará después del upload)
**Uso**: Altura inicial de blobs en Celestia

### Slot 0x12: End Height
```
0x0000000000000000000000000000000000000000000000000000000000000000
```
**Valor**: TBD (se llenará después del upload)
**Uso**: Altura final de blobs en Celestia

### Slot 0x13: Total Blobs
```
0x000000000000000000000000000000000000000000000000000000000000008c
```
**Valor**: 140 (0x8c en hex)
**Uso**: Número total de blobs subidos a Celestia

## 🌱 Storage Slots 0x100 - 0x307: Plant Seeds

520 slots reservados para seeds criptográficos de plantas medicinales.

### Estructura:
```
Slot 0x100: Seed de planta #1
Slot 0x101: Seed de planta #2
...
Slot 0x307: Seed de planta #520 (0x100 + 519 = 0x307)
```

Cada seed es un hash keccak256 de:
```
keccak256(scientific_name + ":" + ncbi_code + ":ANDE:Genesis:2024:Block0")
```

## 🎯 Filosofía

Este genesis combina:

✅ **Sabiduría ancestral andina**: Términos quechua/aymara de medicina tradicional
✅ **Tecnología moderna**: Celestia DA, blockchain, criptografía
✅ **Respeto cultural**: Honrando el conocimiento de los yachaks
✅ **Ciencia**: Referencias NCBI, taxonomía botánica
✅ **Descentralización**: Seeds on-chain, metadata en DA layer

## 📚 Glosario

| Término | Idioma | Significado | Uso en Genesis |
|---------|--------|-------------|----------------|
| Yachak | Quechua | Sabio/Curandero | Identidad cultural |
| Hampi | Quechua | Medicina | Plantas medicinales |
| Koka | Quechua | Coca | Planta sagrada |
| Kamayoc | Quechua | Maestro/Experto | Especialización |
| Pachamama | Quechua | Madre Tierra | Origen de las plantas |
| Apu | Quechua | Espíritu montaña | Sabiduría sagrada |
| Sonk'o | Quechua | Corazón | Esencia vital |
| Wachary | Quechua | Nacer/Crecer | Nuevo comienzo |
| Taita | Quechua | Padre/Abuelo | Respeto ancestral |
| Allin | Quechua | Bueno | Calidad positiva |

## 🔍 Verificación On-Chain

Cualquier persona puede leer estos valores desde el genesis:

```javascript
// Leer slot 0x00 (Yachak)
const value = await provider.getStorageAt(
  "0x0000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000000"
);

// Decodificar hex a string
const decoded = ethers.utils.toUtf8String(value);
console.log(decoded); // "Yachak"
```

```rust
// Desde Rust
let storage_slot = U256::from(0);
let value = storage.get(contract_address, storage_slot);
let text = String::from_utf8(value.to_be_bytes().to_vec())?;
println!("{}", text); // "Yachak"
```

## 🌟 Inspiración

Este diseño honra:
- La medicina tradicional andina
- El conocimiento de los yachaks y curanderos
- La relación sagrada con Pachamama
- La transmisión intergeneracional de sabiduría
- La integración de ciencia ancestral y moderna

---

**Block 0 • Genesis • ANDE Chain**
*Donde la sabiduría ancestral encuentra la tecnología descentralizada*
