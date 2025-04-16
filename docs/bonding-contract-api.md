# Bonding Contract API Documentation

## Overview

The bonding contract implements a factory/child pattern where each bond is represented by an orbital token. This document provides a comprehensive guide to the API for interacting with the bonding contract and bond orbitals.

## Bonding Contract

### Initialization

#### InitContract

Initialize the bonding contract with basic functionality.

- **Opcode**: 0
- **Parameters**:
  - `name`: u128 - The name of the contract
  - `symbol`: u128 - The symbol of the contract
  - `k_factor`: u128 - The k factor for the bonding curve
  - `n_exponent`: u128 - The n exponent for the bonding curve
  - `initial_diesel_reserve`: u128 - The initial diesel reserve
- **Returns**: CallResponse

#### InitBondContract

Initialize the bonding contract with bond functionality.

- **Opcode**: 10
- **Parameters**:
  - `name`: u128 - The name of the contract
  - `symbol`: u128 - The symbol of the contract
  - `virtual_input_reserves`: u128 - The virtual input reserves for the bond curve
  - `virtual_output_reserves`: u128 - The virtual output reserves for the bond curve
  - `half_life`: u64 - The half life for the bond curve
  - `level_bips`: u64 - The level bips for the bond curve
  - `term`: u64 - The term in blocks for bonds
- **Returns**: CallResponse

### Bond Operations

#### PurchaseBond

Purchase a bond with diesel. This creates a new bond orbital token.

- **Opcode**: 11
- **Parameters**:
  - `to`: u128 - The address to receive the bond orbital
  - `min_output`: u128 - The minimum amount of alkane to receive
- **Returns**: CallResponse containing the bond orbital token

#### RedeemBond

Redeem a bond. This requires the bond orbital token.

- **Opcode**: 12
- **Parameters**:
  - `bond_id`: u128 - The ID of the bond to redeem
- **Returns**: CallResponse containing the redeemed alkane

#### RedeemBondBatch

Redeem multiple bonds. This requires the bond orbital tokens.

- **Opcode**: 13
- **Parameters**:
  - `bond_ids`: Vec<u128> - The IDs of the bonds to redeem
- **Returns**: CallResponse containing the redeemed alkane

#### TransferBond

Transfer a bond to another address. This requires the bond orbital token.

- **Opcode**: 14
- **Parameters**:
  - `to`: u128 - The address to receive the bond
  - `bond_id`: u128 - The ID of the bond to transfer
- **Returns**: CallResponse

### Bonding Curve Operations

#### BuyAlkane

Buy alkane with diesel.

- **Opcode**: 1
- **Parameters**:
  - `diesel_amount`: u128 - The amount of diesel to spend
- **Returns**: CallResponse containing the purchased alkane


### Query Operations

#### GetCurrentPrice

Get the current price of alkane in terms of diesel.

- **Opcode**: 3
- **Parameters**: None
- **Returns**: CallResponse containing the price

#### GetBuyAmount

Get the amount of alkane that can be received for a specific amount of diesel.

- **Opcode**: 4
- **Parameters**:
  - `diesel_amount`: u128 - The amount of diesel to spend
- **Returns**: CallResponse containing the amount of alkane

#### GetSellAmount

Get the amount of diesel that can be received for a specific amount of alkane.

- **Opcode**: 5
- **Parameters**:
  - `alkane_amount`: u128 - The amount of alkane to sell
- **Returns**: CallResponse containing the amount of diesel

#### GetBondAmount

Get the bond price (amount of alkane for a specific amount of diesel).

- **Opcode**: 15
- **Parameters**:
  - `diesel_amount`: u128 - The amount of diesel to spend
- **Returns**: CallResponse containing the amount of alkane

#### GetPositionCount

Get the number of bonds owned by an address.

- **Opcode**: 16
- **Parameters**:
  - `address`: u128 - The address to query
- **Returns**: CallResponse containing the number of bonds

#### GetAvailableDebt

Get the available debt (alkane available for redemption).

- **Opcode**: 17
- **Parameters**: None
- **Returns**: CallResponse containing the available debt

#### GetBond

Get bond details.

- **Opcode**: 18
- **Parameters**:
  - `address`: u128 - The address that owns the bond
  - `bond_id`: u128 - The ID of the bond
- **Returns**: CallResponse containing the bond details

### Admin Operations

#### SetVirtualInputReserves

Set the virtual input reserves.

- **Opcode**: 20
- **Parameters**:
  - `value`: u128 - The new virtual input reserves
- **Returns**: CallResponse

#### SetVirtualOutputReserves

Set the virtual output reserves.

- **Opcode**: 21
- **Parameters**:
  - `value`: u128 - The new virtual output reserves
- **Returns**: CallResponse

#### SetHalfLife

Set the half life.

- **Opcode**: 22
- **Parameters**:
  - `value`: u64 - The new half life
- **Returns**: CallResponse

#### SetLevelBips

Set the level bips.

- **Opcode**: 23
- **Parameters**:
  - `value`: u64 - The new level bips
- **Returns**: CallResponse

#### SetLastUpdate

Set the last update timestamp.

- **Opcode**: 24
- **Parameters**: None
- **Returns**: CallResponse

#### SetPause

Toggle pause.

- **Opcode**: 25
- **Parameters**: None
- **Returns**: CallResponse

#### UpdatePricing

Update pricing parameters.

- **Opcode**: 26
- **Parameters**:
  - `new_virtual_input`: Option<u128> - The new virtual input reserves
  - `new_virtual_output`: Option<u128> - The new virtual output reserves
  - `new_half_life`: Option<u64> - The new half life
  - `new_level_bips`: Option<u64> - The new level bips
  - `update_timestamp`: bool - Whether to update the timestamp
  - `pause`: bool - Whether to pause the contract
- **Returns**: CallResponse

### Getters

#### GetName

Get the name of the token.

- **Opcode**: 99
- **Parameters**: None
- **Returns**: CallResponse containing the name

#### GetSymbol

Get the symbol of the token.

- **Opcode**: 100
- **Parameters**: None
- **Returns**: CallResponse containing the symbol

#### GetDieselReserve

Get the reserve of diesel.

- **Opcode**: 101
- **Parameters**: None
- **Returns**: CallResponse containing the diesel reserve

#### GetAlkaneSupply

Get the supply of alkane.

- **Opcode**: 102
- **Parameters**: None
- **Returns**: CallResponse containing the alkane supply

#### GetKFactor

Get the k factor.

- **Opcode**: 103
- **Parameters**: None
- **Returns**: CallResponse containing the k factor

#### GetNExponent

Get the n exponent.

- **Opcode**: 104
- **Parameters**: None
- **Returns**: CallResponse containing the n exponent

#### GetTerm

Get the term.

- **Opcode**: 105
- **Parameters**: None
- **Returns**: CallResponse containing the term

#### GetHalfLife

Get the half life.

- **Opcode**: 106
- **Parameters**: None
- **Returns**: CallResponse containing the half life

#### GetLevelBips

Get the level bips.

- **Opcode**: 107
- **Parameters**: None
- **Returns**: CallResponse containing the level bips

#### GetVirtualInputReserves

Get the virtual input reserves.

- **Opcode**: 108
- **Parameters**: None
- **Returns**: CallResponse containing the virtual input reserves

#### GetVirtualOutputReserves

Get the virtual output reserves.

- **Opcode**: 109
- **Parameters**: None
- **Returns**: CallResponse containing the virtual output reserves

#### GetLastUpdate

Get the last update timestamp.

- **Opcode**: 110
- **Parameters**: None
- **Returns**: CallResponse containing the last update timestamp

#### GetTotalDebt

Get the total debt.

- **Opcode**: 111
- **Parameters**: None
- **Returns**: CallResponse containing the total debt

#### GetPaused

Get the paused state.

- **Opcode**: 112
- **Parameters**: None
- **Returns**: CallResponse containing the paused state

## Bond Orbital

### Initialization

#### Initialize

Initialize the bond orbital.

- **Opcode**: 0
- **Parameters**:
  - `owed`: u128 - The amount owed to the bond holder
  - `creation`: u64 - The creation block number
  - `term`: u64 - The term in blocks
- **Returns**: CallResponse

### Bond Operations

#### Redeem

Redeem the bond.

- **Opcode**: 200
- **Parameters**: None
- **Returns**: CallResponse containing the redeemed alkane

### Query Operations

#### GetBondDetails

Get the bond details.

- **Opcode**: 102
- **Parameters**: None
- **Returns**: CallResponse containing the bond details

#### GetOwed

Get the amount owed to the bond holder.

- **Opcode**: 103
- **Parameters**: None
- **Returns**: CallResponse containing the amount owed

#### GetRedeemed

Get the amount already redeemed.

- **Opcode**: 104
- **Parameters**: None
- **Returns**: CallResponse containing the amount redeemed

#### GetCreation

Get the creation block number.

- **Opcode**: 105
- **Parameters**: None
- **Returns**: CallResponse containing the creation block number

#### GetTerm

Get the term in blocks.

- **Opcode**: 106
- **Parameters**: None
- **Returns**: CallResponse containing the term

#### GetMaturity

Get the maturity block number.

- **Opcode**: 107
- **Parameters**: None
- **Returns**: CallResponse containing the maturity block number

#### GetBondingContractId

Get the bonding contract ID.

- **Opcode**: 108
- **Parameters**: None
- **Returns**: CallResponse containing the bonding contract ID

### Getters

#### GetName

Get the name of the token.

- **Opcode**: 99
- **Parameters**: None
- **Returns**: CallResponse containing the name

#### GetSymbol

Get the symbol of the token.

- **Opcode**: 100
- **Parameters**: None
- **Returns**: CallResponse containing the symbol

#### GetTotalSupply

Get the total supply of the token.

- **Opcode**: 101
- **Parameters**: None
- **Returns**: CallResponse containing the total supply

## Examples

### Purchasing a Bond

```rust
// Create a cellpack to call the bonding contract's PurchaseBond opcode
let bonding_contract_id = AlkaneId { block: 3, tx: 0 };
let to = 123u128; // The address to receive the bond
let min_output = 100u128; // The minimum amount of alkane to receive

let cellpack = Cellpack {
    target: bonding_contract_id,
    inputs: vec![11, to, min_output], // PurchaseBond opcode with parameters
};

// Call the bonding contract
let response = call(
    &cellpack,
    &AlkaneTransferParcel::default(),
    fuel
)?;

// The response contains the bond orbital token
let orbital_transfer = &response.alkanes.0[0];
let orbital_id = orbital_transfer.id.clone();
```

### Redeeming a Bond

```rust
// Create a cellpack to call the bonding contract's RedeemBond opcode
let bonding_contract_id = AlkaneId { block: 3, tx: 0 };
let bond_id = 0u128; // The ID of the bond to redeem

let cellpack = Cellpack {
    target: bonding_contract_id,
    inputs: vec![12, bond_id], // RedeemBond opcode with parameters
};

// Include the bond orbital token in the incoming alkanes
let orbital_id = AlkaneId { block: 2, tx: 0 }; // The ID of the bond orbital
let incoming_alkanes = AlkaneTransferParcel(vec![
    AlkaneTransfer {
        id: orbital_id,
        value: 1u128, // Each orbital has a value of 1
    },
]);

// Call the bonding contract
let response = call(
    &cellpack,
    &incoming_alkanes,
    fuel
)?;

// The response contains the redeemed alkane
let alkane_transfer = &response.alkanes.0[0];
let alkane_amount = alkane_transfer.value;
```

### Transferring a Bond

```rust
// Create a cellpack to call the bonding contract's TransferBond opcode
let bonding_contract_id = AlkaneId { block: 3, tx: 0 };
let to = 456u128; // The address to receive the bond
let bond_id = 0u128; // The ID of the bond to transfer

let cellpack = Cellpack {
    target: bonding_contract_id,
    inputs: vec![14, to, bond_id], // TransferBond opcode with parameters
};

// Include the bond orbital token in the incoming alkanes
let orbital_id = AlkaneId { block: 2, tx: 0 }; // The ID of the bond orbital
let incoming_alkanes = AlkaneTransferParcel(vec![
    AlkaneTransfer {
        id: orbital_id,
        value: 1u128, // Each orbital has a value of 1
    },
]);

// Call the bonding contract
let response = call(
    &cellpack,
    &incoming_alkanes,
    fuel
)?;

// The response contains the bond orbital token for the recipient
let orbital_transfer = &response.alkanes.0[0];
let orbital_id = orbital_transfer.id.clone();
```

## Testing

The bonding contract includes comprehensive tests to ensure its functionality. These tests cover:

- Basic functionality (purchase, redemption, transfer)
- Edge cases (zero values, maximum values)
- Error handling (invalid parameters, unauthorized access)
- Integration with other contracts

### Block-Based Testing

The bonding contract uses block-based testing as the canonical testing approach. This approach provides a more realistic testing environment by simulating actual blockchain interactions, including block creation, transaction indexing, and balance verification through outpoints.

Here's an example of a block-based test for the bonding contract:

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

For more information on block-based testing, see the [Block-Based Testing Guide](./block_based_testing_guide.md).

To run the tests, use the following command:

```bash
cargo test -p bonding-contract
