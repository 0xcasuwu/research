# Free-Mint Test Paradigm: The Canonical Testing Approach

## Overview

This document outlines the adoption of the free-mint test paradigm as the canonical testing approach for the boiler repository, specifically for the bonding contract project. All other testing methods are to be deprecated in favor of this approach.

The free-mint test paradigm provides a realistic testing environment by simulating actual blockchain interactions, including block creation, transaction indexing, and balance verification through outpoints. This approach ensures that contracts are tested in conditions that closely resemble their actual deployment environment.

## Core Principles

1. **Block-Based Testing**: All tests should use block-based testing to simulate the actual blockchain environment.
2. **Transaction-Based Interactions**: Contract interactions should be performed through transactions.
3. **State Verification**: Contract state should be verified through balance sheets and outpoints.
4. **Realistic Environment**: Tests should simulate a realistic blockchain environment with blocks, transactions, and outpoints.

## Implementation Requirements

### Dependencies

To implement the free-mint test paradigm, the following dependencies are required:

```toml
[dependencies]
alkanes = { path = "/path/to/alkanes" }
alkanes-support = { path = "/path/to/alkanes-support" }
metashrew-core = { path = "/path/to/metashrew-core" }
protorune = { path = "/path/to/protorune" }
protorune-support = { path = "/path/to/protorune-support" }
bitcoin = "0.30.0"  # Must match the version used by alkanes
```

### Required Helper Functions

The following helper functions must be implemented for each contract:

1. **init_block_with_contract_deployment**: Creates a block with contract deployment
2. **create_contract_interaction_tx**: Creates a transaction for contract interaction
3. **get_sheet_for_outpoint**: Gets balance sheet for verification
4. **get_last_outpoint_sheet**: Gets the balance sheet for the last transaction in the block
5. **get_token_balance**: Gets token balance
6. **clear_environment**: Clears the test environment

### Test Structure

All tests should follow this structure:

1. Clear the environment
2. Create a block with contract deployment
3. Index the block
4. Verify the initial state
5. Create interaction transactions
6. Index the block again
7. Verify the final state

## Migration Plan

### Phase 1: Dependency Alignment

1. Update the boiler repository to use the same versions of dependencies as the alkanes-rs repository.
2. Resolve any naming discrepancies (e.g., metashrew vs metashrew-core).
3. Ensure all necessary modules are available and accessible.

### Phase 2: Test Migration

1. Identify all existing tests that need to be migrated.
2. Create block-based equivalents for each test.
3. Ensure all tests pass with the new approach.

### Phase 3: Deprecation

1. Mark all non-block-based tests as deprecated.
2. Remove deprecated tests after a transition period.
3. Update documentation to reflect the new testing approach.

## Example Implementation

Here's an example of how to implement a block-based test for the bonding contract:

```rust
#[test]
fn test_bonding_contract_block_based() -> Result<()> {
    // Clear environment
    clear_environment();
    
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

## Conclusion

The adoption of the free-mint test paradigm as the canonical testing approach for the boiler repository will ensure that all contracts are tested in a realistic environment. This will lead to more robust and reliable contracts that behave as expected when deployed on the blockchain.

All development efforts should focus on implementing this testing approach and migrating existing tests to the new paradigm. Any challenges encountered during this process should be addressed by aligning the boiler repository with the alkanes-rs repository, rather than by creating workarounds or alternative testing approaches.
