#!/bin/bash

# Script to run all tests and generate a report

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create a directory for test reports
mkdir -p test-reports

# Function to run a test and save the output
run_test() {
  local test_name=$1
  local test_command=$2
  local output_file="test-reports/${test_name}.log"
  
  echo -e "${BLUE}Running ${test_name}...${NC}"
  
  # Run the test and capture the output
  if $test_command > $output_file 2>&1; then
    echo -e "${GREEN}✓ ${test_name} passed${NC}"
    echo "PASS" >> $output_file
    return 0
  else
    echo -e "${RED}✗ ${test_name} failed${NC}"
    echo "FAIL" >> $output_file
    return 1
  fi
}

# Start time
start_time=$(date +%s)

# Run all tests
echo -e "${YELLOW}=== Running All Tests ===${NC}"

# Unit tests
run_test "Unit Tests" "cargo test -p bonding-contract --quiet"
unit_tests_result=$?

# Integration tests
run_test "Integration Tests" "cargo test --test integration_test --quiet"
integration_tests_result=$?

# Market simulation tests
run_test "Market Simulation Tests" "cargo test --test market_simulation_test --quiet"
market_tests_result=$?

# Benchmark tests
run_test "Benchmark Tests" "cargo test --test benchmark_test --quiet"
benchmark_tests_result=$?

# End time
end_time=$(date +%s)
duration=$((end_time - start_time))

# Generate report
echo -e "${YELLOW}=== Test Report ===${NC}"
echo "Test run completed in ${duration} seconds"
echo

# Check results
failures=0

if [ $unit_tests_result -eq 0 ]; then
  echo -e "${GREEN}✓ Unit Tests: PASS${NC}"
else
  echo -e "${RED}✗ Unit Tests: FAIL${NC}"
  failures=$((failures + 1))
fi

if [ $integration_tests_result -eq 0 ]; then
  echo -e "${GREEN}✓ Integration Tests: PASS${NC}"
else
  echo -e "${RED}✗ Integration Tests: FAIL${NC}"
  failures=$((failures + 1))
fi

if [ $market_tests_result -eq 0 ]; then
  echo -e "${GREEN}✓ Market Simulation Tests: PASS${NC}"
else
  echo -e "${RED}✗ Market Simulation Tests: FAIL${NC}"
  failures=$((failures + 1))
fi

if [ $benchmark_tests_result -eq 0 ]; then
  echo -e "${GREEN}✓ Benchmark Tests: PASS${NC}"
else
  echo -e "${RED}✗ Benchmark Tests: FAIL${NC}"
  failures=$((failures + 1))
fi

echo
if [ $failures -eq 0 ]; then
  echo -e "${GREEN}All tests passed!${NC}"
  exit 0
else
  echo -e "${RED}${failures} test suites failed.${NC}"
  echo -e "${YELLOW}Check the logs in the test-reports directory for details.${NC}"
  exit 1
fi
