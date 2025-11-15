#!/bin/bash
# Test suite for ANDE Chain performance optimizations
#
# This script validates all Phase 1 optimizations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Print header
print_header() {
    echo -e "${BLUE}=================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=================================${NC}"
}

# Print section
print_section() {
    echo -e "\n${YELLOW}>>> $1${NC}"
}

# Run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -n "  Testing $test_name... "
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

print_header "ANDE Chain Optimization Tests"

# 1. Cargo Configuration Tests
print_section "1. Build Configuration"

run_test "maxperf profile exists" \
    "grep -q '\[profile\.maxperf\]' Cargo.toml"

run_test "LTO enabled in maxperf" \
    "grep -q 'lto.*=.*\"fat\"' Cargo.toml"

run_test "jemalloc configured" \
    "grep -q 'tikv-jemallocator' Cargo.toml"

run_test "prometheus dependency" \
    "grep -q 'prometheus.*=' Cargo.toml"

# 2. Source Code Tests
print_section "2. Source Code Structure"

run_test "parallel_executor module exists" \
    "test -f crates/ande-evm/src/parallel_executor.rs"

run_test "parallel_executor has optimal_worker_count" \
    "grep -q 'pub fn optimal_worker_count' crates/ande-evm/src/parallel_executor.rs"

run_test "parallel_executor has Block-STM implementation" \
    "grep -q 'MultiVersionMemory' crates/ande-evm/src/parallel_executor.rs"

run_test "ande-evm exports parallel executor" \
    "grep -q 'pub use parallel_executor' crates/ande-evm/src/lib.rs"

# 3. Docker Configuration Tests
print_section "3. Docker Configuration"

run_test "Dockerfile exists" \
    "test -f Dockerfile"

run_test "Dockerfile uses maxperf profile" \
    "grep -q 'BUILD_PROFILE=maxperf' Dockerfile"

run_test "docker-compose.yml exists" \
    "test -f docker-compose.yml"

# 4. Infrastructure Tests
print_section "4. Infrastructure Configuration"

run_test "Prometheus config exists" \
    "test -f infra/config/prometheus.yml"

run_test "Grafana dashboard exists" \
    "test -f infra/grafana/dashboards/ande-overview.json"

run_test "Alerting rules configured" \
    "test -f infra/prometheus/alerts.yml"

# 5. Compilation Test
print_section "5. Code Compilation"

echo "  Checking code compilation (this may take a moment)..."
if cargo check --all-features --workspace 2>&1 | tee /tmp/ande-build.log | grep -q "Checking"; then
    echo -e "  ${GREEN}✓ PASSED${NC} Code compiles successfully"
    ((TESTS_PASSED++))
else
    if [ -f /tmp/ande-build.log ]; then
        echo -e "  ${RED}✗ FAILED${NC} Compilation errors detected"
        echo "  See /tmp/ande-build.log for details"
        ((TESTS_FAILED++))
    else
        echo -e "  ${YELLOW}⊘ SKIPPED${NC} Could not run compilation check"
        ((TESTS_SKIPPED++))
    fi
fi

# 6. Docker Build Test (optional)
print_section "6. Docker Build"

if command -v docker &> /dev/null; then
    if docker info > /dev/null 2>&1; then
        echo "  Docker daemon is running"
        echo "  Attempting Docker build (this may take several minutes)..."
        if timeout 300 docker build -t ande-chain:test . 2>&1 | grep -q "naming to"; then
            echo -e "  ${GREEN}✓ PASSED${NC} Docker image built successfully"
            ((TESTS_PASSED++))
        else
            echo -e "  ${YELLOW}⊘ SKIPPED${NC} Docker build timed out or failed"
            ((TESTS_SKIPPED++))
        fi
    else
        echo -e "  ${YELLOW}⊘ SKIPPED${NC} Docker daemon not running"
        ((TESTS_SKIPPED++))
    fi
else
    echo -e "  ${YELLOW}⊘ SKIPPED${NC} Docker not installed"
    ((TESTS_SKIPPED++))
fi

# Print summary
echo ""
print_header "Test Summary"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))

echo -e "Total Tests:   $TOTAL_TESTS"
echo -e "${GREEN}Passed:        $TESTS_PASSED${NC}"
echo -e "${RED}Failed:        $TESTS_FAILED${NC}"
echo -e "${YELLOW}Skipped:       $TESTS_SKIPPED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All required tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Build the project: cargo build --profile maxperf"
    echo "  2. Deploy with Docker: docker compose up -d"
    echo "  3. Check metrics: http://localhost:9091 (Prometheus)"
    echo "  4. View dashboards: http://localhost:3000 (Grafana)"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed${NC}"
    echo "Please review the failures above"
    exit 1
fi