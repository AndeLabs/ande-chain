# ANDE Consensus Contract ABIs

Este directorio contiene los ABIs compilados de los contratos Solidity de consenso de ANDE Chain.

## Contratos Disponibles

### 1. AndeConsensus.json
- **Ubicación fuente**: `contracts/src/consensus/AndeConsensus.sol`
- **Propósito**: Contrato principal de consenso multi-sequencer con CometBFT
- **Funciones clave**:
  - `getActiveValidators()` - Obtener validadores activos
  - `getValidatorInfo(address)` - Información detallada del validador
  - `getCurrentProposer()` - Proposer actual para el bloque
  - `isValidator(address)` - Verificar si una dirección es validador
  - `currentEpoch()` - Época actual
  - `totalVotingPower()` - Poder de voto total

### 2. AndeSequencerCoordinator.json
- **Ubicación fuente**: `contracts/src/consensus/AndeSequencerCoordinator.sol`
- **Propósito**: Coordinador de secuenciadores con rotación y timeout
- **Funciones clave**:
  - `currentLeader()` - Líder actual
  - `getActiveSequencers()` - Secuenciadores activos
  - `isTimeoutReached()` - Verificar timeout
  - `checkTimeout()` - Verificar y procesar timeout
  - `recordBlockProduced(address, uint256, uint256)` - Registrar bloque producido

## Uso con ethers-rs

Los ABIs están siendo utilizados en `crates/ande-consensus/src/contract_client.rs` mediante el macro `abigen!` de ethers-rs:

```rust
abigen!(
    AndeConsensus,
    r#"[
        function getActiveValidators() external view returns (address[])
        function getValidatorInfo(address) external view returns (tuple(...))
        // ... más funciones
    ]"#
);
```

### Opción 1: ABI Inline (actual)
Ventajas:
- No requiere archivos externos
- Cambios versionados con el código
- Compilación más rápida

### Opción 2: ABI desde archivo
```rust
abigen!(
    AndeConsensus,
    "./abi/AndeConsensus.json"
);
```

Ventajas:
- ABI completo con todas las funciones
- Sincronización automática con contratos Solidity
- Generación de todos los tipos

## Sincronización con Contratos Solidity

Para actualizar los ABIs después de modificar los contratos Solidity:

```bash
# Desde el directorio raíz del proyecto
cd contracts
forge build

# Copiar ABIs actualizados
cp out/AndeConsensus.sol/AndeConsensus.json ../crates/ande-consensus/abi/
cp out/AndeSequencerCoordinator.sol/AndeSequencerCoordinator.json ../crates/ande-consensus/abi/
```

## Formato ABI

Los archivos JSON contienen:
- **abi**: Array de definiciones de funciones, eventos y errores
- **bytecode**: Bytecode compilado del contrato
- **deployedBytecode**: Bytecode después del despliegue
- **metadata**: Información del compilador y fuentes

## Integración Actual

El cliente de contratos en `contract_client.rs` usa estos ABIs para:

1. **Lectura de estado on-chain**:
   - Obtener lista de validadores activos
   - Consultar información de validadores
   - Verificar proposer actual
   - Obtener época y poder de voto

2. **Eventos** (próximamente):
   - `ValidatorSetUpdated` - Cambios en el conjunto de validadores
   - `BlockProposed` - Bloque propuesto
   - `BlockFinalized` - Bloque finalizado
   - `LeaderRotated` - Rotación de líder

## Seguridad

⚠️ **IMPORTANTE**: Estos ABIs deben coincidir exactamente con los contratos desplegados en la red.

- Verificar checksum del contrato desplegado
- Comparar bytecode con el compilado localmente
- Validar direcciones de contratos en configuración

## Referencias

- Documentación de ethers-rs: https://docs.rs/ethers
- ABIGen guide: https://docs.rs/ethers/latest/ethers/contract/macro.abigen.html
- Contratos Solidity: `../../contracts/src/consensus/`
