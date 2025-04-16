# Bonding Contract Linking Guide

## Overview

The bonding contract in the Alkanes project may experience linking errors due to missing symbols from the runtime library. This document explains the issue and provides solutions specifically for the bonding contract.

## The Bonding Contract

The bonding contract implements two complementary approaches to liquidity provision:

1. **Traditional Bonding Curve**: A mathematical function that determines the price of tokens based on the supply
2. **Bond-Based Approach**: A time-locked redemption mechanism with price decay

The contract is located at:

```
/Users/erickdelgado/Documents/GitHub/boiler/contracts/bonding-contract/
```

## Linking Issues

When building the bonding contract, you may encounter linking errors similar to:

```
Undefined symbols for architecture x86_64:
  "___load_context", "___load_storage", "___request_context", "___request_storage", etc.
```

These symbols are defined in the Alkanes runtime library located at:

```
/Users/erickdelgado/Documents/GitHub/boiler/crates/alkanes-runtime
```

## Why This Happens

The bonding contract is compiled as a dynamic library (.dylib) that expects these symbols to be provided by the host application at runtime. During the build process, the linker needs to know where these symbols will come from, but it can't find them.

## Solutions

### Solution 1: Use the Build Script

We've provided a build script specifically for the bonding contract:

```bash
./build_bonding_contract.sh
```

This script:
1. Builds the runtime library first
2. Verifies that it exports the required symbols
3. Builds the bonding contract with explicit linkage to the runtime

### Solution 2: Manual Build Process

If you prefer to build manually, follow these steps:

1. Build the runtime library:
   ```bash
   cd /Users/erickdelgado/Documents/GitHub/boiler/crates/alkanes-runtime
   cargo build
   ```

2. Build the bonding contract with explicit linkage:
   ```bash
   cd /Users/erickdelgado/Documents/GitHub/boiler/contracts/bonding-contract
   RUSTFLAGS="-L /Users/erickdelgado/Documents/GitHub/boiler/target/debug -L /Users/erickdelgado/Documents/GitHub/boiler/target/debug/deps" cargo build
   ```

### Solution 3: Fix Cargo.toml

Ensure that the bonding contract's `Cargo.toml` correctly specifies the dependency on the runtime:

```toml
[dependencies]
alkanes-runtime = { path = "../../crates/alkanes-runtime" }
```

### Solution 4: Address Architecture Mismatch

If you're still seeing warnings about macOS version mismatches, set the deployment target:

```bash
MACOSX_DEPLOYMENT_TARGET=15.2 cargo build
```

## Testing the Bonding Contract

After successfully building the bonding contract, you can test it using the following commands:

### Running All Tests

```bash
cd /Users/erickdelgado/Documents/GitHub/boiler/contracts/bonding-contract
cargo test
```

### Running Specific Tests

```bash
# Run isolated tests
cargo test --lib isolated_tests

# Run bond curve tests
cargo test --test bond_curve_test

# Run a specific test
cargo test isolated_tests::test_init_contract
```

### Test Environment Setup

The bonding contract tests use a special test environment setup to ensure proper isolation between tests:

1. **Reset Mock Environment**: The `reset_mock_environment::reset()` function is called before and after each test to ensure a clean state.

2. **Bypass Initialization Check**: In test environments, the `observe_initialization` method bypasses the initialization check to allow multiple initializations.

3. **Test Isolation**: The `run_test_with_isolation` function wraps each test in a safe environment that prevents test failures from affecting other tests.

## Troubleshooting

### Memory Corruption Issues

If you encounter memory corruption issues during testing, ensure that:

1. The `reset_mock_environment::reset()` function is called before and after each test.
2. The `observe_initialization` method is properly bypassing the initialization check in test environments.
3. Each test is properly isolated using the `run_test_with_isolation` function.

### Linking Errors

If you encounter linking errors:

1. Check that the runtime library is built before the bonding contract.
2. Verify that the `Cargo.toml` file correctly specifies the dependency on the runtime.
3. Use the `RUSTFLAGS` environment variable to specify the location of the runtime library.

### Test Failures

If tests are failing:

1. Check that the test environment is properly set up.
2. Verify that the mock context and storage are properly reset between tests.
3. Ensure that the initialization state is properly reset between tests.

## Related Libraries

The bonding contract is part of a larger ecosystem of Alkanes libraries, all of which may experience similar linking issues. If you need to build all related libraries, use the comprehensive build script:

```bash
./build_all_libraries.sh
```

This will build all the libraries that were experiencing linking errors, including the bonding contract.

## Conclusion

By following the steps in this guide, you should be able to successfully build and test the bonding contract. If you encounter any issues, please refer to the troubleshooting section or contact the development team for assistance.
