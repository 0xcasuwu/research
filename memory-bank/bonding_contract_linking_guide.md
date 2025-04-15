# Bonding Contract Linking Guide

## Overview

The bonding contract in the Alkanes project is experiencing linking errors due to missing symbols from the runtime library. This document explains the issue and provides solutions specifically for the bonding contract.

## The Bonding Contract

The bonding contract implements a bonding curve mechanism, which is a mathematical concept used in token economics. The contract is located at:

```
/Users/erickdelgado/Documents/GitHub/boiler/contracts/bonding-contract/src/bonding_curve.rs
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

After successfully building the bonding contract, you can test it using the Alkanes test framework. The compiled library will be available at:

```
/Users/erickdelgado/Documents/GitHub/boiler/target/debug/libbonding_contract.dylib
```

## Related Libraries

The bonding contract is part of a larger ecosystem of Alkanes libraries, all of which may experience similar linking issues. If you need to build all related libraries, use the comprehensive build script:

```bash
./build_all_libraries.sh
```

This will build all the libraries that were experiencing linking errors, including the bonding contract.
