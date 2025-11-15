# ⚡ ANDE Chain - Quick Start Guide

Guía rápida para probar todas las implementaciones de Phase 1.

---

## 🚀 Opción 1: Testing Rápido (5 minutos)

```bash
# 1. Verificar que todo está implementado
./scripts/test-improvements.sh

# Deberías ver:
# ✓ Tests Passed: 23+
# ✓ ALL TESTS PASSED!
```

---

## 🐳 Opción 2: Deploy Completo con Docker (10 minutos)

```bash
# 1. Build la imagen optimizada
docker compose build ande-node

# 2. Iniciar todos los servicios
docker compose up -d

# 3. Verificar que todo está running
docker compose ps

# 4. Ver logs en tiempo real
docker compose logs -f ande-node

# 5. Acceder a los dashboards
open http://localhost:3000  # Grafana (admin/andechain2024)
open http://localhost:9090  # Prometheus
open http://localhost:4000  # Blockscout Explorer
```

---

## 📊 Opción 3: Verificar Métricas (2 minutos)

```bash
# Ver métricas en formato Prometheus
curl http://localhost:9001/metrics | grep ande_

# Deberías ver métricas como:
# ande_parallel_execution_success_total
# ande_mev_value_extracted_wei
# ande_da_submissions_total
# ande_validator_participation_rate_percent
# ... y 26+ métricas más
```

---

## 🔥 Opción 4: Build Local Optimizado (15 minutos)

```bash
# Build con maximum performance
RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld" \
  cargo build --profile maxperf --features "jemalloc,asm-keccak"

# El binario estará en:
# target/maxperf/ande-node

# Ejecutar con métricas
./target/maxperf/ande-node node \
  --metrics 0.0.0.0:9001 \
  --datadir ./data \
  --http --http.addr 0.0.0.0 --http.port 8545

# En otra terminal, ver métricas
curl http://localhost:9001/metrics
```

---

## 🧪 Opción 5: Testing Individual (variable)

### Test 1: Rate Limiter
```bash
# Simular muchas requests rápidas
for i in {1..150}; do
  curl -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' &
done

# Deberías ver rate limit después de ~100 requests
# Métrica: ande_rpc_rate_limit_hits_total debería incrementar
```

### Test 2: Parallel Execution Metrics
```bash
# Enviar transacciones y observar métricas paralelas
watch -n 1 'curl -s http://localhost:9001/metrics | grep parallel'

# Observa:
# - ande_parallel_execution_success_total (incrementa)
# - ande_parallel_workers_active (debería ser 16)
# - ande_parallel_throughput_tps (TPS actual)
```

### Test 3: Alertas Prometheus
```bash
# Ver alertas activas
curl http://localhost:9090/api/v1/alerts | jq '.data.alerts'

# Ver reglas de alertas
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].rules[].name'

# Deberías ver 22+ reglas de alertas
```

---

## 📈 Endpoints Importantes

| Servicio | URL | Descripción |
|----------|-----|-------------|
| **RPC Node** | http://localhost:8545 | JSON-RPC endpoint |
| **WebSocket** | ws://localhost:8546 | WebSocket endpoint |
| **Metrics** | http://localhost:9001/metrics | Prometheus metrics |
| **Grafana** | http://localhost:3000 | Dashboards (admin/andechain2024) |
| **Prometheus** | http://localhost:9090 | Metrics & Alerts |
| **Blockscout** | http://localhost:4000 | Block explorer |
| **Engine API** | http://localhost:8551 | Consensus engine |

---

## 🔍 Verificación de Implementaciones

### 1. Build Profiles ✅
```bash
# Verificar perfiles en Cargo.toml
grep -A5 "profile.maxperf" Cargo.toml
grep -A5 "profile.release" Cargo.toml

# Deberías ver lto = "fat", codegen-units = 1
```

### 2. REVM Config ✅
```bash
# Verificar módulo existe
test -f crates/ande-evm/src/config.rs && echo "✓ REVM config exists"

# Ver configuración
head -50 crates/ande-evm/src/config.rs
```

### 3. Metrics System ✅
```bash
# Verificar módulo existe
test -f crates/ande-node/src/metrics.rs && echo "✓ Metrics module exists"

# Contar métricas implementadas
grep -c "IntCounter\|IntGauge\|Histogram" crates/ande-node/src/metrics.rs
# Debería mostrar 30+
```

### 4. Rate Limiter ✅
```bash
# Verificar módulo existe
test -f crates/ande-rpc/src/rate_limiter.rs && echo "✓ Rate limiter exists"

# Ver configuración
grep "requests_per_second" crates/ande-rpc/src/rate_limiter.rs
```

### 5. Prometheus Alerts ✅
```bash
# Verificar alertas existen
test -f infra/prometheus/alerts.yml && echo "✓ Alerts exist"

# Contar grupos de alertas
grep -c "name: ande_" infra/prometheus/alerts.yml
# Debería mostrar 5 grupos

# Contar alertas totales
grep -c "alert:" infra/prometheus/alerts.yml
# Debería mostrar 22+
```

### 6. Grafana Dashboard ✅
```bash
# Verificar dashboard existe
test -f infra/grafana/dashboards/ande-overview.json && echo "✓ Dashboard exists"

# Contar paneles
grep -c '"id":' infra/grafana/dashboards/ande-overview.json
# Debería mostrar 14+
```

### 7. Docker Optimization ✅
```bash
# Verificar Dockerfile usa maxperf
grep "BUILD_PROFILE=maxperf" Dockerfile && echo "✓ Maxperf enabled"

# Verificar RUSTFLAGS
grep "RUSTFLAGS.*target-cpu" Dockerfile && echo "✓ RUSTFLAGS optimized"
```

---

## 🎯 Performance Testing

### Load Test con k6 (si lo tienes instalado)
```javascript
// load-test.js
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 100,        // 100 usuarios virtuales
  duration: '1m',  // 1 minuto
};

export default function () {
  const payload = JSON.stringify({
    jsonrpc: '2.0',
    method: 'eth_blockNumber',
    params: [],
    id: 1,
  });

  const res = http.post('http://localhost:8545', payload, {
    headers: { 'Content-Type': 'application/json' },
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'not rate limited': (r) => r.status !== 429,
  });
}
```

```bash
# Ejecutar load test
k6 run load-test.js

# Observar métricas en Grafana mientras corre el test
```

### Benchmark Local
```bash
# Crear script de benchmark
cat > bench.sh << 'EOF'
#!/bin/bash
echo "Starting benchmark..."
START=$(date +%s)

for i in {1..1000}; do
  curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    > /dev/null
done

END=$(date +%s)
DURATION=$((END - START))
TPS=$((1000 / DURATION))

echo "1000 requests in ${DURATION}s = ${TPS} req/s"
EOF

chmod +x bench.sh
./bench.sh
```

---

## 🐛 Troubleshooting

### Docker no inicia
```bash
# Ver logs de error
docker compose logs ande-node

# Rebuild con verbose
docker compose build --no-cache --progress=plain ande-node

# Verificar puertos no estén en uso
lsof -i :8545
lsof -i :9001
```

### Métricas no aparecen
```bash
# Verificar endpoint de métricas
curl http://localhost:9001/metrics

# Si no responde, verificar que el nodo esté corriendo
docker compose ps ande-node

# Ver logs del nodo
docker compose logs -f ande-node | grep metrics
```

### Rate limiter muy agresivo
```bash
# Editar crates/ande-rpc/src/rate_limiter.rs
# Cambiar requests_per_second a un valor mayor
# Rebuild y reiniciar
```

### Grafana no muestra datos
```bash
# Verificar que Prometheus está scrapeando
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets'

# Verificar que hay métricas
curl http://localhost:9090/api/v1/query?query=ande_blocks_proposed_total

# Reimportar dashboard en Grafana
```

---

## 📋 Checklist Pre-Deploy

Antes de deployar a testnet/mainnet:

- [ ] ✅ Todos los tests pasan (`./scripts/test-improvements.sh`)
- [ ] ✅ Build con maxperf funciona
- [ ] ✅ Docker image se construye sin errores
- [ ] ✅ Métricas son accesibles
- [ ] ✅ Grafana muestra datos reales
- [ ] ✅ Alertas se activan correctamente
- [ ] ✅ Rate limiter funciona
- [ ] ⏳ Load test sostenido (10 min+) sin crashes
- [ ] ⏳ Memory usage estable (no leaks)
- [ ] ⏳ Disk space management verificado

---

## 🎓 Learning Resources

### Ver código implementado
```bash
# REVM config
cat crates/ande-evm/src/config.rs

# Metrics system
cat crates/ande-node/src/metrics.rs

# Rate limiter
cat crates/ande-rpc/src/rate_limiter.rs

# Alerts
cat infra/prometheus/alerts.yml
```

### Documentación completa
- [PHASE_1_COMPLETE.md](./PHASE_1_COMPLETE.md) - Resumen ejecutivo
- [PRIME_TIME_RECOMMENDATIONS.md](./PRIME_TIME_RECOMMENDATIONS.md) - Best practices completas
- [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) - Detalles técnicos

---

## 🚀 Production Deploy Command

Cuando estés listo para producción:

```bash
# 1. Build producción optimizada
RUSTFLAGS="-C target-cpu=native" \
  cargo build --profile maxperf --features "jemalloc,asm-keccak"

# 2. Build Docker image con todas las optimizaciones
docker build -t ande-chain:v1.0.0 \
  --build-arg BUILD_PROFILE=maxperf \
  --build-arg FEATURES="jemalloc asm-keccak" \
  --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
  --build-arg VCS_REF=$(git rev-parse --short HEAD) \
  .

# 3. Tag para registry
docker tag ande-chain:v1.0.0 your-registry.io/ande-chain:v1.0.0
docker tag ande-chain:v1.0.0 your-registry.io/ande-chain:latest

# 4. Push to registry
docker push your-registry.io/ande-chain:v1.0.0
docker push your-registry.io/ande-chain:latest

# 5. Deploy con compose en servidor
docker compose -f docker-compose.prod.yml up -d
```

---

## ⚡ Ultra Quick Start (1 comando)

```bash
# Test everything + Start Docker
./scripts/test-improvements.sh && docker compose up -d && \
  echo "✅ ANDE Chain is running!" && \
  echo "📊 Grafana: http://localhost:3000" && \
  echo "📈 Metrics: http://localhost:9001/metrics" && \
  echo "🔗 RPC: http://localhost:8545"
```

---

**¿Tienes problemas?** Revisa los logs:
```bash
docker compose logs -f ande-node
```

**¿Todo funciona?** 🎉 ¡Felicidades! Phase 1 está completa.

**Próximo paso:** Load testing y optimizaciones de Phase 2.
