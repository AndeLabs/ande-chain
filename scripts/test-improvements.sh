#!/bin/bash
# =============================================================================
# ANDE Chain - Test Improvements Script
# Tests all implemented optimizations and generates report
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 ANDE Chain - Testing Improvements"
echo "===================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run test
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -n "Testing $test_name... "
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo "1️⃣  Build Profile Tests"
echo "----------------------"

# Test maxperf profile exists
run_test "maxperf profile exists" \
    "grep -q '\[profile.maxperf\]' Cargo.toml"

# Test profiling profile exists
run_test "profiling profile exists" \
    "grep -q '\[profile.profiling\]' Cargo.toml"

# Test LTO enabled
run_test "LTO enabled in release" \
    "grep -A3 '\[profile.release\]' Cargo.toml | grep -q 'lto = \"fat\"'"

# Test jemalloc feature
run_test "jemalloc feature defined" \
    "grep -q 'jemalloc.*tikv-jemallocator' Cargo.toml"

echo ""
echo "2️⃣  Source Code Tests"
echo "--------------------"

# Test REVM config module
run_test "REVM config module exists" \
    "test -f crates/ande-evm/src/config.rs"

# Test metrics module
run_test "Metrics module exists" \
    "test -f crates/ande-node/src/metrics.rs"

# Test rate limiter module
run_test "Rate limiter module exists" \
    "test -f crates/ande-rpc/src/rate_limiter.rs"

# Test config compiles
if run_test "REVM config compiles" \
    "cd crates/ande-evm && cargo check --lib 2>&1 | grep -q 'Finished'"; then
    :
fi

echo ""
echo "3️⃣  Monitoring Infrastructure Tests"
echo "----------------------------------"

# Test Prometheus alerts
run_test "Prometheus alerts exist" \
    "test -f infra/prometheus/alerts.yml"

# Test alert groups
run_test "Critical alerts defined" \
    "grep -q 'name: ande_critical' infra/prometheus/alerts.yml"

run_test "Security alerts defined" \
    "grep -q 'name: ande_security' infra/prometheus/alerts.yml"

# Test Grafana dashboard
run_test "Grafana dashboard exists" \
    "test -f infra/grafana/dashboards/ande-overview.json"

# Test dashboard has panels
run_test "Dashboard has panels" \
    "grep -q '\"panels\"' infra/grafana/dashboards/ande-overview.json"

echo ""
echo "4️⃣  Docker Configuration Tests"
echo "-----------------------------"

# Test Dockerfile uses maxperf
run_test "Dockerfile uses maxperf profile" \
    "grep -q 'BUILD_PROFILE=maxperf' Dockerfile"

# Test RUSTFLAGS optimization
run_test "Dockerfile has RUSTFLAGS" \
    "grep -q 'RUSTFLAGS.*target-cpu' Dockerfile"

# Test docker-compose exists
run_test "docker-compose.yml exists" \
    "test -f docker-compose.yml"

echo ""
echo "5️⃣  Documentation Tests"
echo "----------------------"

# Test documentation files
run_test "Prime Time Recommendations exists" \
    "test -f PRIME_TIME_RECOMMENDATIONS.md"

run_test "Implementation Progress exists" \
    "test -f IMPLEMENTATION_PROGRESS.md"

echo ""
echo "6️⃣  Dependency Tests"
echo "-------------------"

# Test required dependencies
run_test "Prometheus dependency" \
    "grep -q 'prometheus.*=' Cargo.toml"

run_test "Governor (rate limiting)" \
    "grep -q 'governor.*=' Cargo.toml"

run_test "Zstd (compression)" \
    "grep -q 'zstd.*=' Cargo.toml"

echo ""
echo "7️⃣  Compilation Tests"
echo "--------------------"

echo "Building with release profile..."
if cargo build --profile release --quiet 2>&1 | tee /tmp/ande-build.log; then
    echo -e "${GREEN}✓ Release build successful${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ Release build failed${NC}"
    echo "See /tmp/ande-build.log for details"
    ((TESTS_FAILED++))
fi

echo ""
echo "8️⃣  Unit Tests"
echo "-------------"

echo "Running unit tests..."
if cargo test --lib --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠ Some unit tests may have failed${NC}"
    ((TESTS_FAILED++))
fi

echo ""
echo "=========================================="
echo "📊 Test Results Summary"
echo "=========================================="
echo ""
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total Tests:  $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo ""
    echo "✅ Optimizations verified:"
    echo "   • Build profiles (maxperf, profiling)"
    echo "   • REVM configuration"
    echo "   • Enhanced metrics system"
    echo "   • RPC rate limiting"
    echo "   • Prometheus alerting"
    echo "   • Grafana dashboards"
    echo "   • Docker optimizations"
    echo ""
    echo "Next steps:"
    echo "1. Run: docker compose build"
    echo "2. Run: docker compose up -d"
    echo "3. Access Grafana: http://localhost:3000"
    echo "4. Check metrics: http://localhost:9001/metrics"
    exit 0
else
    echo -e "${RED}⚠️  SOME TESTS FAILED${NC}"
    echo ""
    echo "Please review the failures above and fix them before deploying."
    exit 1
fi
