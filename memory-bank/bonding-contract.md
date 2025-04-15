---
title: "Bonding Contract"
description: "Implementation of a bonding contract in the Alkanes metaprotocol"
category: "examples"
related: ["simple-token", "predicate"]
tags: ["bonding", "curve", "swap", "token", "bonds"]
---

# Bonding Contract

## Summary

This document provides a complete implementation of a bonding contract in the Alkanes metaprotocol. The bonding contract offers two complementary approaches to liquidity provision:

1. **Traditional Bonding Curve**: Allows users to swap an alkane for diesel (the genesis alkane) following a smooth price curve. As more alkane is swapped for diesel, the price becomes more expensive until the curve is filled.

2. **Bond-Based Approach**: Offers time-locked redemption with a price decay mechanism, providing more flexibility and control for token issuers.

## Key Concepts

- **Bonding Curve**: A mathematical function that determines the price of tokens based on the supply
- **Diesel**: The genesis alkane (AlkaneId [2, 0])
- **Swap**: Exchange of one alkane for another based on the bonding curve
- **Reserve**: The amount of diesel held by the contract
- **Supply**: The amount of alkane tokens in circulation
- **Bond**: A claim on future alkane tokens that matures over time
- **Maturity**: The process by which bonds become redeemable over time
- **Price Decay**: A mechanism that reduces the price of bonds over time

## Contract Structure

The bonding contract consists of:

1. **Contract Struct**: The main contract struct (`BondingContractAlkane`)
2. **Message Enum**: The enum for opcode-based dispatch (`BondingContractAlkaneMessage`)
3. **Interface Traits**: Traits for bonding curve and bond functionality (`BondingContract`, `BondContract`)
4. **Storage Functions**: Functions for interacting with storage
5. **Bonding Operations**: Functions for bonding operations (buy, sell)
6. **Bond Operations**: Functions for bond operations (purchase, redeem, transfer)
7. **AlkaneResponder Implementation**: Implementation of the AlkaneResponder trait

## Traditional Bonding Curve

The traditional bonding curve implementation allows users to:

- Buy alkane with diesel
- Sell alkane for diesel
- Query the current price and other contract information

### Price Calculation

The contract calculates prices based on a configurable bonding curve:

```rust
fn current_price(&self) -> Result<u128> {
    let supply = self.total_supply();
    let reserve = self.reserve();
    
    if supply == 0 {
        // Initial price if supply is 0
        return Ok(1);
    }
    
    // For a quadratic curve: price = reserve / supply^2
    let supply_squared = supply.checked_mul(supply)
        .ok_or_else(|| anyhow!("calculation overflow"))?;
    
    let price = reserve.checked_div(supply_squared)
        .ok_or_else(|| anyhow!("division by zero"))?;
    
    Ok(price)
}
```

### Buy and Sell Operations

The contract implements buy and sell operations:

```rust
fn buy(&self, diesel_amount: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default();
    
    // Check if diesel was sent
    let mut diesel_received = 0;
    for transfer in &context.incoming_alkanes.0 {
        if transfer.id == AlkaneId { block: 2, tx: 0 } { // Diesel is [2, 0]
            diesel_received += transfer.value;
        }
    }
    
    if diesel_received == 0 {
        return Err(anyhow!("no diesel received"));
    }
    
    // Calculate the amount of tokens to mint
    let token_amount = self.calculate_buy_amount(diesel_received)?;
    
    // Update the reserve
    let reserve = self.reserve();
    self.set_reserve(reserve + diesel_received);
    
    // Update the supply
    let supply = self.total_supply();
    self.set_total_supply(supply + token_amount);
    
    // Update the buyer's balance
    let buyer = context.caller.tx;
    let buyer_balance = self.balance_of(buyer);
    self.set_balance(buyer, buyer_balance + token_amount);
    
    // Add the minted tokens to the response
    response.alkanes.0.push(AlkaneTransfer {
        id: context.myself.clone(),
        value: token_amount,
    });
    
    Ok(response)
}
```

## Bond-Based Approach

The bond-based approach offers a more sophisticated liquidity provision mechanism:

- Time-locked redemption with linear maturity
- Exponential price decay mechanism with configurable floor
- Bond transfer functionality
- Owner management functions for pricing parameters
- Emergency pause capability

### Bond Structure

```rust
pub struct Bond {
    /// The amount of alkane owed to the bond holder
    pub owed: u128,
    /// The amount of alkane already redeemed
    pub redeemed: u128,
    /// The timestamp when the bond was created
    pub creation: u64,
}
```

### Pricing Mechanism

The bond-based approach uses a pricing mechanism based on virtual reserves:

```rust
pub struct Pricing {
    /// Virtual input reserves (diesel)
    pub virtual_input_reserves: u128,
    /// Virtual output reserves (alkane)
    pub virtual_output_reserves: u128,
    /// Half-life in seconds
    pub half_life: u64,
    /// Level in basis points (percentage of original price)
    pub level_bips: u64,
    /// Last update timestamp
    pub last_update: u64,
}
```

### Bond Operations

The contract implements bond operations:

```rust
fn purchase_bond(&mut self, to: u128, min_output: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Check if the contract is paused
    if self.is_paused() {
        return Err(anyhow!("contract is paused"));
    }
    
    // Get the diesel from the incoming alkanes
    let mut diesel_amount = 0;
    for alkane in &context.incoming_alkanes.0 {
        if alkane.id.block == 2 && alkane.id.tx == 0 {
            // This is diesel
            diesel_amount += alkane.value;
        }
    }
    
    if diesel_amount == 0 {
        return Err(anyhow!("no diesel provided"));
    }
    
    // Get the bond curve
    let mut curve = self.get_bond_curve();
    
    // Calculate the amount of alkane to mint
    let current_time = self.get_current_timestamp();
    let available_debt = self.available_debt();
    let alkane_amount = curve.purchase_bond(diesel_amount, available_debt);
    
    // Create a new bond
    let bond = Bond {
        owed: alkane_amount,
        redeemed: 0,
        creation: current_time,
    };
    
    if alkane_amount < min_output {
        return Err(anyhow!("output less than minimum"));
    }
    
    // Update the total debt
    self.set_total_debt(self.total_debt() + bond.owed);
    
    // Add the bond to the recipient
    self.add_bond(to, bond);
    
    // Update the contract state
    self.set_virtual_input_reserves_internal(curve.pricing.virtual_input_reserves);
    self.set_virtual_output_reserves_internal(curve.pricing.virtual_output_reserves);
    self.set_last_update_internal(curve.pricing.last_update);
    
    // Update the bond curve instance
    self.bond_curve = Some(curve);
    
    Ok(response)
}
```

## Test Environment

The bonding contract includes a comprehensive test suite that verifies all aspects of its functionality. The test environment is designed to ensure proper isolation between tests:

```rust
fn run_test_with_isolation<F>(test_fn: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    // Reset the mock environment before the test
    reset_mock_environment::reset();
    
    // Reset the initialization state
    let mut init_pointer = StoragePointer::from_keyword("/initialized");
    init_pointer.set_value::<u8>(0);
    
    // Run the test function in a catch_unwind to prevent test failures from affecting other tests
    let result = std::panic::catch_unwind(test_fn);
    
    // Reset the mock environment after the test regardless of success or failure
    reset_mock_environment::reset();
    
    // Reset the initialization state again
    let mut init_pointer = StoragePointer::from_keyword("/initialized");
    init_pointer.set_value::<u8>(0);
    
    // If the test panicked, resume the panic
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
```

## Usage Guide

### Deployment

To deploy the bonding contract:

1. Compile the contract to WebAssembly:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy the contract using a cellpack with `[1, 0]` header:
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

3. The contract will be deployed at address `[2, n]`, where n is the next available sequence number.

### Traditional Bonding Curve Operations

#### Buying Tokens

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

#### Selling Tokens

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

### Bond-Based Operations

#### Purchasing Bonds

To purchase bonds with diesel:

```rust
// First, send diesel to the contract
let diesel_transfer = AlkaneTransfer {
    id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
    value: 1000,
};

// Then call the purchase bond function
let purchase_bond_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        11,       // PurchaseBond opcode
        recipient_address, // Recipient address
        100,      // Minimum output
    ],
};
```

#### Redeeming Bonds

To redeem a bond:

```rust
// Call the redeem bond function
let redeem_bond_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        12,       // RedeemBond opcode
        0,        // Bond ID
    ],
};
```

#### Transferring Bonds

To transfer a bond to another address:

```rust
// Call the transfer bond function
let transfer_bond_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        14,       // TransferBond opcode
        recipient_address, // Recipient address
        0,        // Bond ID
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

// Get the bond amount for a specific diesel amount
let get_bond_amount_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        15,       // GetBondAmount opcode
        1000,     // Diesel amount
    ],
};
```

## Example Scenario: Token Launch

Let's consider a scenario where a protocol wants to launch a token using the bonding contract:

1. **Deploy the Bonding Contract**:
   ```rust
   let bonding_cellpack = Cellpack {
       target: AlkaneId { block: 1, tx: 0 },
       inputs: vec![
           10,             // InitBondContract opcode
           0x50524f54,     // "PROT" as u128 (name)
           0x5054,         // "PT" as u128 (symbol)
           1000000,        // Virtual input reserves
           2000000,        // Virtual output reserves
           86400,          // Half-life (1 day in seconds)
           100,            // Level bips (1%)
           604800,         // Term (1 week in seconds)
       ],
   };
   ```

2. **Set Initial Alkane Supply**:
   ```rust
   let set_alkane_supply_cellpack = Cellpack {
       target: contract_id,
       inputs: vec![
           // SetAlkaneSupply opcode
           1000000,        // Initial supply
       ],
   };
   ```

3. **Unpause the Contract**:
   ```rust
   let unpause_cellpack = Cellpack {
       target: contract_id,
       inputs: vec![
           25,             // SetPause opcode
       ],
   };
   ```

4. **Users Purchase Bonds**:
   ```rust
   let purchase_bond_cellpack = Cellpack {
       target: contract_id,
       inputs: vec![
           11,             // PurchaseBond opcode
           user_address,   // User address
           100,            // Minimum output
       ],
   };
   ```

5. **Users Redeem Bonds After Maturity**:
   ```rust
   let redeem_bond_cellpack = Cellpack {
       target: contract_id,
       inputs: vec![
           12,             // RedeemBond opcode
           0,              // Bond ID
       ],
   };
   ```

## Key Features

### Traditional Bonding Curve

- **Configurable Curve**: Support for different curve types (constant, linear, quadratic, etc.)
- **Immediate Liquidity**: Tokens can be bought and sold immediately
- **Simple Price Model**: Price is determined by the reserve ratio

### Bond-Based Approach

- **Time-Locked Redemption**: Bonds mature over time, allowing for controlled token distribution
- **Price Decay Mechanism**: Implements an exponential price decay with a floor level
- **Bond Transfer**: Bonds can be transferred to other addresses
- **Owner Controls**: Provides management functions for adjusting pricing parameters
- **Pause Functionality**: Includes emergency pause capability for added security

## Conclusion

The bonding contract implementation provides a complete solution for creating a token that follows either a traditional bonding curve or a bond-based approach. The contract is fully tested and ready for deployment.

The traditional bonding curve provides immediate liquidity with a simple price model, while the bond-based approach offers time-locked redemption with a more sophisticated pricing mechanism. Both approaches can be used depending on the specific requirements of the project.
