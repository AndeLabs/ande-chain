# Contributing to Ande Chain

¡Gracias por tu interés en contribuir a Ande Chain!

## 🚀 Inicio Rápido

1. Fork el repositorio
2. Clona tu fork
3. Crea una branch para tu feature
4. Haz tus cambios
5. Ejecuta tests
6. Abre un Pull Request

## 📝 Convenciones de Código

### Rust

- Usar `cargo fmt` antes de cada commit
- Ejecutar `cargo clippy` y resolver todos los warnings
- Añadir tests para nueva funcionalidad
- Documentar APIs públicas con doc comments

### Solidity

- Seguir el style guide de Solidity
- Usar `forge fmt` para formatear
- Añadir tests para todos los contratos
- Documentar funciones públicas con NatSpec

## 🧪 Testing

```bash
# Ejecutar todos los tests
./scripts/test-all.sh

# Tests Rust
cargo test --workspace

# Tests Solidity
cd contracts && forge test -vvv
```

## 📋 Checklist de Pull Request

- [ ] Código formateado (`cargo fmt`, `forge fmt`)
- [ ] Linting pasa (`cargo clippy`)
- [ ] Tests añadidos/actualizados
- [ ] Documentación actualizada
- [ ] Commit messages siguen Conventional Commits
- [ ] CI pasa

## 🎯 Áreas de Contribución

- **Core Protocol**: Mejoras al execution client
- **Smart Contracts**: Nuevos contratos o mejoras
- **Documentation**: Mejoras a la documentación
- **Testing**: Añadir más tests
- **Tooling**: Herramientas de desarrollo
- **Infrastructure**: Docker, K8s, CI/CD

## 💬 Comunicación

- GitHub Issues para bugs y features
- GitHub Discussions para preguntas
- Discord para chat en tiempo real

## 📜 Licencia

Al contribuir, aceptas que tus contribuciones se licencien bajo MIT o Apache-2.0.
