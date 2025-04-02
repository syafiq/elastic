#!/bin/bash

echo "Running File Interface Tests..."

# Validate WIT interface
echo "Validating WIT interface..."
wit-bindgen validate tests/wit/file.wit
if [ $? -ne 0 ]; then
    echo "✗ WIT interface validation failed"
    exit 1
fi
echo "✓ WIT interface validation passed"

# Run Rust tests
echo "Running Rust tests..."
cargo test --test file_tests -- --nocapture
if [ $? -ne 0 ]; then
    echo "✗ Rust tests failed"
    exit 1
fi
echo "✓ Rust tests passed"

echo "All tests passed successfully!" 