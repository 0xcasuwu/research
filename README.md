# Bonding Contract for Alkanes

This repository contains an implementation of a bonding contract for the Alkanes metaprotocol. The bonding contract allows users to swap an alkane for diesel (the genesis alkane) following a smooth price curve. As more alkane is swapped for diesel, the price becomes more expensive until the curve is filled.

## Project Structure

```
boiler/
├── Cargo.toml                 # Workspace manifest
├── contracts/                 # Directory for contract crates
│   └── bonding-contract/      # Bonding contract implementation
│       ├── Cargo.toml         # Contract manifest
│       └── src/               # Contract source code
│           └── lib.rs         # Contract implementation
└── crates/                    # Supporting crates
    ├── alkanes-runtime/       # Runtime environment
    ├── alkanes-support/       # Support utilities
    ├── alkanes-macros/        # Macros for contract development
    └── ...                    # Other supporting crates
```

## Bonding Contract

The bonding contract implements a quadratic bonding curve where the price of tokens increases as more tokens are minted. The contract provides the following functionality:

- **Buy tokens with diesel**: Users can send diesel to the contract and receive tokens based on the bonding curve.
- **Sell tokens for diesel**: Users can send tokens back to the contract and receive diesel based on the bonding curve.
- **Query contract information**: Users can query the current price, total supply, reserve, and other contract information.

### Bonding Curve

The contract uses a quadratic bonding curve where the price is determined by the formula:

```
price = reserve / (supply^2)
```

This creates a smooth curve that becomes more expensive as the supply increases.

## Building and Deploying

### Prerequisites

- Rust and Cargo
- wasm32-unknown-unknown target

### Building

To build the bonding contract, use the provided build script:

```bash
./scripts/build.sh
```

This will:
1. Check if the wasm32-unknown-unknown target is installed and install it if needed
2. Build the bonding contract
3. Copy the WASM file to the `dist` directory

You can also run tests or examples with the build script:

```bash
./scripts/build.sh --test    # Run tests
./scripts/build.sh --example # Run example
```

### Testing

The project includes several types of tests:

#### Running All Tests

To run all tests and generate a report:

```bash
./scripts/run_tests.sh
```

This script will run all test suites and generate a report with the results. The test logs are saved in the `test-reports` directory.

#### Individual Test Suites

You can also run individual test suites:

##### Unit Tests

Basic unit tests for the bonding contract:

```bash
cargo test -p bonding-contract
```

##### Integration Tests

Integration tests that simulate real-world usage of the contract:

```bash
cargo test --test integration_test
```

##### Market Simulation Tests

Tests that simulate market behavior with multiple participants:

```bash
cargo test --test market_simulation_test
```

##### Benchmark Tests

Performance benchmarks for different operations:

```bash
cargo test --test benchmark_test
```

These tests help ensure that the bonding contract works correctly and efficiently in various scenarios.

### Configuration

The bonding contract can be configured for different environments using the provided configuration script:

```bash
# Generate deployment scripts and configuration summaries for the default environment
./scripts/configure.js

# Generate for a specific environment (testnet or mainnet)
./scripts/configure.js testnet
./scripts/configure.js mainnet
```

This will generate:
- Environment-specific deployment scripts in the `scripts` directory
- Configuration summaries in the `docs` directory

### Deploying

To deploy the bonding contract, use the provided deploy scripts:

```bash
# Deploy with default configuration
./scripts/deploy.sh

# Deploy with environment-specific configuration
./scripts/deploy-testnet.sh
./scripts/deploy-mainnet.sh
```

These scripts provide guidance on how to deploy the contract using the Alkanes CLI or SDK.

In a real deployment, you would use code like this:

```rust
let cellpack = Cellpack {
    target: AlkaneId { block: 1, tx: 0 },
    inputs: vec![
        0,              // Initialize opcode
        0x424f4e44,     // "BOND" as u128 (name)
        0x424e44,       // "BND" as u128 (symbol)
        1000000,        // Initial supply
        1000000,        // Initial reserve
    ],
};
```

The deployment parameters can be customized in the `contracts/bonding-contract/config.json` file.

## Usage

### Buying Tokens

To buy tokens with diesel:

```rust
// First, send diesel to the contract
let diesel_transfer = AlkaneTransfer {
    id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
    value: 1000,
};

// Then call the buy function
let buy_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        1,        // Buy opcode
    ],
};
```

### Selling Tokens

To sell tokens for diesel:

```rust
// First, send tokens to the contract
let token_transfer = AlkaneTransfer {
    id: contract_id,
    value: 1000,
};

// Then call the sell function
let sell_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        2,        // Sell opcode
        1000,     // Amount
    ],
};
```

### Querying Contract Information

To get information about the contract:

```rust
// Get the current price
let get_current_price_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        3,        // GetCurrentPrice opcode
    ],
};

// Get the buy price for a specific amount
let get_buy_price_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        4,        // GetBuyPrice opcode
        1000,     // Amount
    ],
};
```

## License

MIT
