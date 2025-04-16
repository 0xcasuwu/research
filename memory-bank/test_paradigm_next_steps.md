# Free-Mint Test Paradigm: Detailed Next Steps

This document outlines the detailed next steps for implementing the free-mint test paradigm as the canonical testing approach for the bonding contract project. These steps are intended to guide the implementation process and ensure a smooth transition from the current testing approach to the block-based testing approach.

## 1. Fix Dependency Issues

### 1.1 Update Cargo.toml

```toml
[dev-dependencies]
alkanes = { git = "https://github.com/kungfuflex/alkanes-rs", features = ["test-utils"] }
metashrew-core = { git = "https://github.com/sandshrewmetaprotocols/metashrew", features = ["test-utils"] }
wasm-bindgen-test = "0.3.40"
```

### 1.2 Enable Test-Utils Feature for Protorune

```toml
protorune = { path = "/Users/erickdelgado/Documents/GitHub/boiler/crates/protorune", features = ["test-utils"] }
```

### 1.3 Update Bitcoin Version

```toml
bitcoin = "0.32.4"
```

## 2. Fix Block Test Helpers

### 2.1 Update Import Paths

- Replace `metashrew::get_cache` with `metashrew_core::get_cache` in test code
- Use conditional compilation with `#[cfg(test)]` for test-specific imports

### 2.2 Fix Txid Creation

```rust
// Use from_raw_hash instead of from_byte_array
let hash = [0u8; 32];
bitcoin::Txid::from_raw_hash(bitcoin::bitcoin_hashes::sha256d::Hash::from_inner(hash))
```

### 2.3 Fix Protocol Tag

```rust
// Use the correct protocol tag
RuneTable::for_protocol(alkanes::message::AlkaneMessageContext::protocol_tag())
```

## 3. Migrate Existing Tests

### 3.1 Identify Tests to Migrate

- context_test.rs
- flow_test.rs
- public_api_test.rs
- debug_flow_test.rs
- simple_flow_test.rs
- orbital_flow_test.rs
- orbital_test.rs

### 3.2 Create Block-Based Equivalents

- For each test, create a block-based version that follows the canonical structure
- Example: convert `test_purchase_bond` to use block-based testing

### 3.3 Add Deprecation Notices

```rust
#[deprecated(
    since = "0.2.0",
    note = "This test uses context-based testing which is deprecated. Use block-based testing instead."
)]
```

## 4. Update Documentation

### 4.1 Update API Documentation

- Add examples of block-based testing to `bonding-contract-api.md`
- Document the canonical testing approach in all relevant files

### 4.2 Create Testing Guide

- Create a new file `block_based_testing_guide.md` with detailed instructions
- Include examples for common testing scenarios

### 4.3 Update Memory Bank

- Ensure `activeContext.md` and `progress.md` reflect the new testing approach
- Add entries about the migration to block-based testing

## 5. Verify and Test

### 5.1 Run All Tests

```bash
cargo test -p bonding-contract
```

### 5.2 Fix Any Remaining Issues

- Address any compilation errors or test failures
- Ensure all tests pass with the new approach

### 5.3 Validate Test Coverage

- Ensure all functionality is covered by the new block-based tests
- Add additional tests for any gaps in coverage

## 6. Final Steps

### 6.1 Remove Mock Implementations

- Once all tests are migrated, remove any mock implementations
- Use the actual alkanes crate functionality instead

### 6.2 Update CI/CD Pipeline

- Ensure CI/CD pipeline uses the new testing approach
- Add checks to prevent non-block-based tests from being added

### 6.3 Train Team Members

- Provide training on the new testing approach
- Ensure all team members understand how to write block-based tests

## Example Migration: test_purchase_bond

Here's an example of how to migrate the `test_purchase_bond` test from context-based to block-based testing:

### Original Context-Based Test

```rust
#[test]
fn test_purchase_bond() -> Result<()> {
    let mut contract = reset_contract_state();
    
    // Set up test parameters
    let diesel_amount = 1000;
    let min_output = 1;
    let to = 1; // Address 1
    
    // Create context with diesel tokens
    let mut context = create_context_with_diesel(diesel_amount);
    
    // Purchase bond
    let response = contract.purchase_bond(&mut context, to, min_output)?;
    
    // Verify bond was created
    assert!(response.success, "Bond purchase should succeed");
    
    // Verify diesel was transferred
    assert_eq!(contract.diesel_reserve(), diesel_amount, "Diesel reserve should match amount sent");
    
    // Verify bond was created for the correct address
    let bonds = contract.get_bonds(to);
    assert_eq!(bonds.len(), 1, "One bond should be created");
    
    Ok(())
}
```

### Migrated Block-Based Test

```rust
#[wasm_bindgen_test]
fn test_purchase_bond_block_based() -> Result<()> {
    // Clear environment
    clear_environment();
    
    // Set up test parameters
    let block_height = 840_000;
    let diesel_amount = 1000;
    let min_output = 1;
    let to = 1; // Address 1
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BT";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    // Bond parameters
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 2_000_000;
    let half_life = 86400; // 1 day in seconds
    let level_bips = 100; // 1%
    let term = 604800; // 1 week in seconds
    
    // Create block with bond contract deployment
    let (mut test_block, bond_contract_id) = init_block_with_contract_deployment(
        bonding_contract_build::get_bytes(),
        vec![name, symbol, virtual_input_reserves, virtual_output_reserves, half_life, level_bips, term],
        AlkaneId::new(3, ALKANE_FACTORY_BONDING_CONTRACT_ID),
    )?;
    
    // Index the block
    index_block(&test_block, block_height)?;
    
    // Create purchase bond transaction
    let previous_outpoint = OutPoint {
        txid: test_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let purchase_outpoint = create_contract_interaction_tx(
        &mut test_block,
        bond_contract_id,
        11, // PurchaseBond opcode
        vec![to, min_output],
        previous_outpoint,
    );
    
    // Index the block
    index_block(&test_block, block_height + 1)?;
    
    // Verify diesel was transferred
    let diesel_balance = get_token_balance(&test_block, AlkaneId::new(0, 0))?; // Diesel token ID
    assert_eq!(diesel_balance, diesel_amount, "Diesel balance should match amount sent");
    
    // Verify bond was created
    // This would require additional helper functions to check bond state
    // For example, we could add a get_bond_count helper function
    
    Ok(())
}
```

## Conclusion

Following these detailed steps will ensure a smooth transition from the current testing approach to the block-based testing approach. The migration process should be done incrementally, starting with the most critical tests and gradually moving to less critical ones. This will allow for early detection of any issues and ensure that the migration process does not disrupt the development workflow.
