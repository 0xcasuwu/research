#!/bin/bash

# Build script for the bonding contract

set -e

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
  echo "Installing wasm32-unknown-unknown target..."
  rustup target add wasm32-unknown-unknown
fi

# Build the bonding contract
echo "Building bonding contract..."
cd "$(dirname "$0")/.."
cargo build --target wasm32-unknown-unknown --release -p bonding-contract

# Copy the wasm file to a more accessible location
mkdir -p dist
cp target/wasm32-unknown-unknown/release/bonding_contract.wasm dist/

echo "Build completed successfully!"
echo "WASM file is available at: dist/bonding_contract.wasm"

# Calculate the size of the WASM file
WASM_SIZE=$(du -h dist/bonding_contract.wasm | cut -f1)
echo "WASM file size: $WASM_SIZE"

# Optional: Run tests
if [ "$1" == "--test" ]; then
  echo "Running tests..."
  cargo test -p bonding-contract
fi

# Optional: Run example
if [ "$1" == "--example" ]; then
  echo "Running example..."
  cargo run --example bonding_contract_example
fi
