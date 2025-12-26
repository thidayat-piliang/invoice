#!/bin/bash

# FlashBill Integration Test Runner
#
# This script runs the full integration test suite for FlashBill API
#
# Usage:
#   ./run_integration_tests.sh
#
# Requirements:
#   - PostgreSQL running
#   - DATABASE_URL environment variable set
#   - API server running (or will be started automatically)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== FlashBill Integration Tests ===${NC}"
echo ""

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}Error: DATABASE_URL not set${NC}"
    echo "Please set DATABASE_URL environment variable:"
    echo "  export DATABASE_URL=\"postgres://postgres:postgres@localhost:5432/flashbill_test\""
    exit 1
fi

# Check if API_URL is set, default to localhost:3000
if [ -z "$API_URL" ]; then
    export API_URL="http://localhost:3000"
    echo -e "${YELLOW}Note: API_URL not set, using default: http://localhost:3000${NC}"
fi

echo "Database URL: $DATABASE_URL"
echo "API URL: $API_URL"
echo ""

# Check if API is already running
echo -e "${YELLOW}Checking if API is running...${NC}"
if curl -s "$API_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}API is already running${NC}"
    API_STARTED_BY_SCRIPT=false
else
    echo -e "${YELLOW}API not running, starting it...${NC}"

    # Check if binary exists
    if [ ! -f "target/debug/flashbill-api" ]; then
        echo -e "${YELLOW}Building API...${NC}"
        cargo build
    fi

    # Start API in background
    cargo run --bin flashbill-api &
    API_PID=$!
    API_STARTED_BY_SCRIPT=true

    # Wait for API to be ready
    echo -e "${YELLOW}Waiting for API to start...${NC}"
    for i in {1..30}; do
        if curl -s "$API_URL/health" > /dev/null 2>&1; then
            echo -e "${GREEN}API is ready!${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}API failed to start within 30 seconds${NC}"
            kill $API_PID 2>/dev/null
            exit 1
        fi
        sleep 1
    done
fi

echo ""
echo -e "${YELLOW}Running integration tests...${NC}"
echo ""

# Run the tests
cargo test --test integration_test -- --test-threads=1

TEST_RESULT=$?

echo ""
if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}=== All integration tests passed! ===${NC}"
else
    echo -e "${RED}=== Some tests failed ===${NC}"
fi

# Cleanup
if [ "$API_STARTED_BY_SCRIPT" = true ]; then
    echo ""
    echo -e "${YELLOW}Stopping API server...${NC}"
    kill $API_PID 2>/dev/null
    wait $API_PID 2>/dev/null
    echo -e "${GREEN}API server stopped${NC}"
fi

exit $TEST_RESULT
