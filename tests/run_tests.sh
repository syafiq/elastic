#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "Running Clock Interface Tests"
echo "============================"

# Check if wit-bindgen is installed
if ! command -v wit-bindgen &> /dev/null; then
    echo -e "${RED}Error: wit-bindgen is not installed${NC}"
    echo "Please install wit-bindgen first:"
    echo "cargo install wit-bindgen-cli"
    exit 1
fi

# Validate WIT file and generate bindings
echo -e "\nValidating WIT interface..."
if wit-bindgen rust tests/wit/clock.wit; then
    echo -e "${GREEN}✓ WIT interface validation passed${NC}"
else
    echo -e "${RED}✗ WIT interface validation failed${NC}"
    exit 1
fi

# Run Python tests
echo -e "\nRunning Python implementation tests..."
if python3 -m unittest tests/python/test_clock.py -v; then
    echo -e "${GREEN}✓ Python implementation tests passed${NC}"
else
    echo -e "${RED}✗ Python implementation tests failed${NC}"
    exit 1
fi

echo -e "\n${GREEN}All tests passed successfully!${NC}" 