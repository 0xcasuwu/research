# Block-Based Testing Guide

This guide provides detailed instructions for implementing block-based testing for contracts in the boiler repository. Block-based testing is the canonical testing approach for all contracts, as it provides a more realistic testing environment by simulating actual blockchain interactions.

## Overview

Block-based testing simulates the actual blockchain environment by creating blocks, adding transactions to those blocks, and indexing the blocks to update the state. This approach allows for more realistic testing of contracts, as it tests them in conditions that closely resemble their actual deployment environment.

## Prerequisites

Before implementing block-based testing, ensure that your project has the following dependencies:

```toml
[dev-dependencies]
alkanes = { git = "https://github.com/kungfuflex/alkanes-rs", features = ["test-utils"] }
metashrew-core = { git = "https://github.com/sandshrewmetaprotocols/metashrew", features = ["test-utils"] }
wasm-bindgen-test = "0.3.40"
```

Also, ensure that the protorune crate has the test-utils feature enabled:

```toml
protorune = { path = "/path/to/protorune", features = ["test-utils"] }
```

## Helper Functions

The following helper functions are provided in the `block_test_helpers.rs` module:

### init_block_with_contract_deployment

Creates a block with contract deployment.

```rust
pub fn init_block_with_contract_deployment(
    contract_bytes: Vec<u8>,
    init_params: Vec<u128>,
    target: AlkaneId,
) -> Result<(bitcoin::Block, AlkaneId)>
```

### create_contract_interaction_tx

Creates a transaction for contract interaction.

```rust
pub fn create_contract_interaction_tx(
    test_block: &mut bitcoin::Block,
    contract_id: AlkaneId,
    operation: u128,
    params: Vec<u128>,
    previous_outpoint: OutPoint,
) -> OutPoint
```

### get_sheet_for_outpoint

Gets balance sheet for verification.

```rust
pub fn get_sheet_for_outpoint(
    test_block: &bitcoin::Block,
    tx_num: usize,
    vout: u32,
) -> Result<BalanceSheet<IndexPointer>>
```

### get_last_outpoint_sheet

Gets the balance sheet for the last transaction in the block.

```rust
pub fn get_last_outpoint_sheet(test_block: &bitcoin::Block) -> Result<BalanceSheet<IndexPointer>>
```

### get_token_balance

Gets token balance.

```rust
pub fn get_token_balance(block: &bitcoin::Block, token_id: AlkaneId) -> Result<u128>
```

### clear_environment

Clears the test environment.

```rust
pub fn clear_environment()
```

## Test Structure

All block-based tests should follow this structure:

1. Clear the environment
2. Create a block with contract deployment
3. Index the block
4. Verify the initial state
5. Create interaction transactions
6. Index the block again
7. Verify the final state

Here's an example of a block-based test:

```rust
#[wasm_bindgen_test]
fn test_bonding_contract_block_based() -> Result<()> {
    // Clear environment
    clear_environment();
    
    // Set up test parameters
    let block_height = 840_000;
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondingToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BND";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let k_factor = 1;
    let n_exponent = 1;
    let initial_reserve = 1_000_000;
    
    // Create block with bonding contract deployment
    let (mut test_block, bonding_contract_id) = init_block_with_contract_deployment(
        bonding_contract_build::get_bytes(),
        vec![name, symbol, k_factor, n_exponent, initial_reserve],
        AlkaneId::new(3, ALKANE_FACTORY_BONDING_CONTRACT_ID),
    )?;
    
    // Index the block
    index_block(&test_block, block_height)?;
    
    // Verify initial state
    let initial_balance = get_token_balance(&test_block, bonding_contract_id)?;
    assert_eq!(initial_balance, initial_reserve, "Initial token balance should match initial reserve");
    
    // Create buy transaction
    let previous_outpoint = OutPoint {
        txid: test_block.txdata[0].compute_txid(),
        vout: 0,
    };
    let buy_amount = 50_000;
    let buy_outpoint = create_contract_interaction_tx(
        &mut test_block,
        bonding_contract_id,
        BUY_OPCODE,
        vec![buy_amount],
        previous_outpoint,
    );
    
    // Index the block
    index_block(&test_block, block_height + 1)?;
    
    // Verify final state
    let final_balance = get_token_balance(&test_block, bonding_contract_id)?;
    assert!(final_balance > initial_balance, "Balance should increase after buy operation");
    
    Ok(())
}
```

## Common Testing Scenarios

### Testing Contract Deployment

```rust
// Create block with contract deployment
let (test_block, contract_id) = init_block_with_contract_deployment(
    contract_bytes,
    init_params,
    target,
)?;

// Index the block
index_block(&test_block, block_height)?;

// Verify initial state
let initial_balance = get_token_balance(&test_block, contract_id)?;
assert_eq!(initial_balance, expected_balance, "Initial token balance should match expected balance");
```

### Testing Contract Interaction

```rust
// Create interaction transaction
let previous_outpoint = OutPoint {
    txid: test_block.txdata[0].compute_txid(),
    vout: 0,
};
let interaction_outpoint = create_contract_interaction_tx(
    &mut test_block,
    contract_id,
    OPERATION_CODE,
    params,
    previous_outpoint,
);

// Index the block
index_block(&test_block, block_height + 1)?;

// Verify state after interaction
let final_balance = get_token_balance(&test_block, contract_id)?;
assert_eq!(final_balance, expected_balance, "Balance should match expected balance after interaction");
```

### Testing Multiple Interactions

```rust
// Create first interaction transaction
let previous_outpoint = OutPoint {
    txid: test_block.txdata[0].compute_txid(),
    vout: 0,
};
let first_interaction_outpoint = create_contract_interaction_tx(
    &mut test_block,
    contract_id,
    FIRST_OPERATION_CODE,
    first_params,
    previous_outpoint,
);

// Create second interaction transaction
let second_interaction_outpoint = create_contract_interaction_tx(
    &mut test_block,
    contract_id,
    SECOND_OPERATION_CODE,
    second_params,
    first_interaction_outpoint,
);

// Index the block
index_block(&test_block, block_height + 1)?;

// Verify state after interactions
let final_balance = get_token_balance(&test_block, contract_id)?;
assert_eq!(final_balance, expected_balance, "Balance should match expected balance after interactions");
```

## Best Practices

1. **Use wasm_bindgen_test**: Use the `#[wasm_bindgen_test]` attribute for all block-based tests.
2. **Clear the environment**: Always clear the environment before starting a test.
3. **Verify state**: Always verify the state after each operation.
4. **Use descriptive assertions**: Use descriptive assertion messages to make it clear what is being tested.
5. **Test edge cases**: Test edge cases to ensure that the contract behaves correctly in all situations.
6. **Test error cases**: Test error cases to ensure that the contract handles errors correctly.
7. **Test multiple interactions**: Test multiple interactions to ensure that the contract behaves correctly in complex scenarios.

## Migrating from Context-Based Testing

When migrating from context-based testing to block-based testing, follow these steps:

1. Identify the test to migrate.
2. Create a new test function with the `_block_based` suffix.
3. Add the `#[wasm_bindgen_test]` attribute to the new test function.
4. Implement the test using the block-based testing approach.
5. Add a deprecation notice to the old test function.

Here's an example of a deprecation notice:

```rust
#[deprecated(
    since = "0.2.0",
    note = "This test uses context-based testing which is deprecated. Use block-based testing instead."
)]
```

## Conclusion

Block-based testing provides a more realistic testing environment for contracts by simulating actual blockchain interactions. By following the guidelines in this document, you can implement block-based testing for your contracts and ensure that they behave correctly in a realistic environment.
