---
title: "Bonding Contract"
description: "Implementation of a bonding contract in the Alkanes metaprotocol"
category: "examples"
related: ["simple-token", "predicate"]
tags: ["bonding", "curve", "swap", "token"]
---

# Bonding Contract

## Summary

This document provides a complete implementation of a bonding contract in the Alkanes metaprotocol. The bonding contract allows users to swap an alkane for diesel (the genesis alkane) following a smooth price curve. As more alkane is swapped for diesel, the price becomes more expensive until the curve is filled.

## Key Concepts

- **Bonding Curve**: A mathematical function that determines the price of tokens based on the supply
- **Diesel**: The genesis alkane (AlkaneId [2, 0])
- **Swap**: Exchange of one alkane for another based on the bonding curve
- **Reserve**: The amount of diesel held by the contract
- **Supply**: The amount of alkane tokens in circulation

## Contract Structure

The bonding contract consists of:

1. **Contract Struct**: The main contract struct
2. **Message Enum**: The enum for opcode-based dispatch
3. **BondingCurve Trait**: A trait for bonding curve functionality
4. **Storage Functions**: Functions for interacting with storage
5. **Bonding Operations**: Functions for bonding operations
6. **AlkaneResponder Implementation**: Implementation of the AlkaneResponder trait

## Contract Implementation

```rust
use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, message::MessageDispatch};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// BondingCurve trait defines the interface for bonding curve functionality
pub trait BondingCurve: AlkaneResponder {
    /// Calculate the price for buying a specific amount of tokens
    fn calculate_buy_price(&self, amount: u128) -> Result<u128>;
    
    /// Calculate the price for selling a specific amount of tokens
    fn calculate_sell_price(&self, amount: u128) -> Result<u128>;
    
    /// Calculate the amount of tokens that can be bought with a specific amount of diesel
    fn calculate_buy_amount(&self, diesel_amount: u128) -> Result<u128>;
    
    /// Calculate the amount of diesel that can be received for selling a specific amount of tokens
    fn calculate_sell_amount(&self, token_amount: u128) -> Result<u128>;
    
    /// Buy tokens with diesel
    fn buy(&self, diesel_amount: u128) -> Result<CallResponse>;
    
    /// Sell tokens for diesel
    fn sell(&self, token_amount: u128) -> Result<CallResponse>;
    
    /// Get the current reserve (amount of diesel held by the contract)
    fn reserve(&self) -> u128;
    
    /// Get the current supply (amount of tokens in circulation)
    fn supply(&self) -> u128;
    
    /// Get the current price of the token
    fn current_price(&self) -> Result<u128>;
}

/// BondingContractAlkane implements a bonding contract
#[derive(Default)]
pub struct BondingContractAlkane(());

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum BondingContractAlkaneMessage {
    /// Initialize the contract
    #[opcode(0)]
    Initialize {
        name: u128,
        symbol: u128,
        initial_supply: u128,
        initial_reserve: u128,
    },
    
    /// Buy tokens with diesel
    #[opcode(1)]
    Buy,
    
    /// Sell tokens for diesel
    #[opcode(2)]
    Sell {
        amount: u128,
    },
    
    /// Get the current price of the token
    #[opcode(3)]
    #[returns(u128)]
    GetCurrentPrice,
    
    /// Get the price for buying a specific amount of tokens
    #[opcode(4)]
    #[returns(u128)]
    GetBuyPrice {
        amount: u128,
    },
    
    /// Get the price for selling a specific amount of tokens
    #[opcode(5)]
    #[returns(u128)]
    GetSellPrice {
        amount: u128,
    },
    
    /// Get the amount of tokens that can be bought with a specific amount of diesel
    #[opcode(6)]
    #[returns(u128)]
    GetBuyAmount {
        diesel_amount: u128,
    },
    
    /// Get the amount of diesel that can be received for selling a specific amount of tokens
    #[opcode(7)]
    #[returns(u128)]
    GetSellAmount {
        token_amount: u128,
    },
    
    /// Get the name of the token
    #[opcode(99)]
    #[returns(String)]
    GetName,
    
    /// Get the symbol of the token
    #[opcode(100)]
    #[returns(String)]
    GetSymbol,
    
    /// Get the total supply of the token
    #[opcode(101)]
    #[returns(u128)]
    GetTotalSupply,
    
    /// Get the reserve of the contract
    #[opcode(102)]
    #[returns(u128)]
    GetReserve,
}

impl BondingContractAlkane {
    // Storage functions
    
    /// Get the pointer to the name
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }
    
    /// Get the name
    fn name(&self) -> String {
        String::from_utf8_lossy(self.name_pointer().get().as_ref()).to_string()
    }
    
    /// Set the name
    fn set_name(&self, name: u128) {
        self.name_pointer().set(Arc::new(trim(name).as_bytes().to_vec()));
    }
    
    /// Get the pointer to the symbol
    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }
    
    /// Get the symbol
    fn symbol(&self) -> String {
        String::from_utf8_lossy(self.symbol_pointer().get().as_ref()).to_string()
    }
    
    /// Set the symbol
    fn set_symbol(&self, symbol: u128) {
        self.symbol_pointer().set(Arc::new(trim(symbol).as_bytes().to_vec()));
    }
    
    /// Get the pointer to the total supply
    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/totalsupply")
    }
    
    /// Get the total supply
    fn total_supply(&self) -> u128 {
        self.total_supply_pointer().get_value::<u128>()
    }
    
    /// Set the total supply
    fn set_total_supply(&self, supply: u128) {
        self.total_supply_pointer().set_value::<u128>(supply);
    }
    
    /// Get the pointer to the reserve
    fn reserve_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/reserve")
    }
    
    /// Get the reserve
    fn reserve(&self) -> u128 {
        self.reserve_pointer().get_value::<u128>()
    }
    
    /// Set the reserve
    fn set_reserve(&self, reserve: u128) {
        self.reserve_pointer().set_value::<u128>(reserve);
    }
    
    /// Get the balance of an address
    fn balance_of(&self, address: u128) -> u128 {
        StoragePointer::from_keyword("/balances/")
            .select(&address.to_le_bytes())
            .get_value::<u128>()
    }
    
    /// Set the balance of an address
    fn set_balance(&self, address: u128, balance: u128) {
        StoragePointer::from_keyword("/balances/")
            .select(&address.to_le_bytes())
            .set_value::<u128>(balance);
    }
    
    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<()> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set_value::<u8>(0x01);
            Ok(())
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    
    // Bonding curve functions
    
    /// Calculate the price for buying a specific amount of tokens
    /// Uses a quadratic bonding curve: price = reserve / (supply^2)
    fn calculate_buy_price(&self, amount: u128) -> Result<u128> {
        let supply = self.total_supply();
        let reserve = self.reserve();
        
        // Calculate the price using the integral of the price curve
        // For a quadratic curve: price = reserve * ((supply + amount)^2 - supply^2) / supply^2
        let new_supply = supply.checked_add(amount)
            .ok_or_else(|| anyhow!("supply overflow"))?;
        
        let new_supply_squared = new_supply.checked_mul(new_supply)
            .ok_or_else(|| anyhow!("calculation overflow"))?;
        
        let supply_squared = supply.checked_mul(supply)
            .ok_or_else(|| anyhow!("calculation overflow"))?;
        
        let supply_diff = new_supply_squared.checked_sub(supply_squared)
            .ok_or_else(|| anyhow!("calculation underflow"))?;
        
        let price = if supply_squared == 0 {
            // Initial price if supply is 0
            amount
        } else {
            reserve.checked_mul(supply_diff)
                .ok_or_else(|| anyhow!("calculation overflow"))?
                .checked_div(supply_squared)
                .ok_or_else(|| anyhow!("division by zero"))?
        };
        
        Ok(price)
    }
    
    /// Calculate the price for selling a specific amount of tokens
    /// Uses a quadratic bonding curve: price = reserve / (supply^2)
    fn calculate_sell_price(&self, amount: u128) -> Result<u128> {
        let supply = self.total_supply();
        let reserve = self.reserve();
        
        if amount > supply {
            return Err(anyhow!("insufficient supply"));
        }
        
        // Calculate the price using the integral of the price curve
        // For a quadratic curve: price = reserve * (supply^2 - (supply - amount)^2) / supply^2
        let new_supply = supply.checked_sub(amount)
            .ok_or_else(|| anyhow!("supply underflow"))?;
        
        let new_supply_squared = new_supply.checked_mul(new_supply)
            .ok_or_else(|| anyhow!("calculation overflow"))?;
        
        let supply_squared = supply.checked_mul(supply)
            .ok_or_else(|| anyhow!("calculation overflow"))?;
        
        let supply_diff = supply_squared.checked_sub(new_supply_squared)
            .ok_or_else(|| anyhow!("calculation underflow"))?;
        
        let price = reserve.checked_mul(supply_diff)
            .ok_or_else(|| anyhow!("calculation overflow"))?
            .checked_div(supply_squared)
            .ok_or_else(|| anyhow!("division by zero"))?;
        
        Ok(price)
    }
    
    /// Calculate the amount of tokens that can be bought with a specific amount of diesel
    fn calculate_buy_amount(&self, diesel_amount: u128) -> Result<u128> {
        let supply = self.total_supply();
        let reserve = self.reserve();
        
        if reserve == 0 || supply == 0 {
            // Initial case: 1:1 ratio
            return Ok(diesel_amount);
        }
        
        // For a quadratic curve, solving for amount:
        // diesel_amount = reserve * ((supply + amount)^2 - supply^2) / supply^2
        // Simplified: amount = supply * (sqrt(1 + diesel_amount * supply / reserve) - 1)
        
        // This is a simplified approximation for demonstration purposes
        // In a real implementation, you would use a more precise calculation
        let ratio = diesel_amount.checked_mul(supply)
            .ok_or_else(|| anyhow!("calculation overflow"))?
            .checked_div(reserve)
            .ok_or_else(|| anyhow!("division by zero"))?;
        
        // Approximate sqrt(1 + ratio) - 1 using a linear approximation
        // For small values of ratio, sqrt(1 + ratio) ≈ 1 + ratio/2
        let amount = supply.checked_mul(ratio)
            .ok_or_else(|| anyhow!("calculation overflow"))?
            .checked_div(2)
            .ok_or_else(|| anyhow!("division by zero"))?;
        
        Ok(amount)
    }
    
    /// Calculate the amount of diesel that can be received for selling a specific amount of tokens
    fn calculate_sell_amount(&self, token_amount: u128) -> Result<u128> {
        self.calculate_sell_price(token_amount)
    }
    
    /// Get the current price of the token
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
    
    // Contract operations
    
    /// Initialize the contract
    fn initialize(&self, name: u128, symbol: u128, initial_supply: u128, initial_reserve: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prevent multiple initializations
        self.observe_initialization()?;
        
        // Set contract properties
        self.set_name(name);
        self.set_symbol(symbol);
        self.set_total_supply(initial_supply);
        self.set_reserve(initial_reserve);
        
        // Assign initial supply to the creator
        let creator = context.caller.tx;
        self.set_balance(creator, initial_supply);
        
        // Add the minted tokens to the response
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: initial_supply,
        });
        
        Ok(response)
    }
    
    /// Buy tokens with diesel
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

fn sell(&self, token_amount: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::default();
    
    // Check if tokens were sent
    let mut tokens_received = 0;
    for transfer in &context.incoming_alkanes.0 {
        if transfer.id == context.myself {
            tokens_received += transfer.value;
        }
    }
    
    if tokens_received == 0 {
        return Err(anyhow!("no tokens received"));
    }
    
    // Calculate the amount of diesel to return
    let diesel_amount = self.calculate_sell_price(tokens_received)?;
    
    // Update the reserve
    let reserve = self.reserve();
    if diesel_amount > reserve {
        return Err(anyhow!("insufficient reserve"));
    }
    self.set_reserve(reserve - diesel_amount);
    
    // Update the supply
    let supply = self.total_supply();
    self.set_total_supply(supply - tokens_received);
    
    // Add the diesel to the response
    response.alkanes.0.push(AlkaneTransfer {
        id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
        value: diesel_amount,
    });
    
    Ok(response)
}
```

### Price Calculation

The contract calculates prices based on a quadratic bonding curve:

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

## Conclusion

This bonding contract implementation provides a complete solution for creating a token that follows a bonding curve. The quadratic curve ensures that the price increases as more tokens are minted, creating a smooth price curve that becomes more expensive as the supply increases.

The contract allows users to:
- Buy tokens with diesel (the genesis alkane)
- Sell tokens back to the contract for diesel
- Query the current price and other contract information

This implementation can be extended to support different curve types by modifying the price calculation functions.
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
    
    /// Get the name of the token
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.name().into_bytes();
        
        Ok(response)
    }
    
    /// Get the symbol of the token
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.symbol().into_bytes();
        
        Ok(response)
    }
    
    /// Get the total supply of the token
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.total_supply().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the reserve of the contract
    fn get_reserve(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.reserve().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the current price of the token
    fn get_current_price(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.current_price()?.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the price for buying a specific amount of tokens
    fn get_buy_price(&self, amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.calculate_buy_price(amount)?.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the price for selling a specific amount of tokens
    fn get_sell_price(&self, amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.calculate_sell_price(amount)?.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the amount of tokens that can be bought with a specific amount of diesel
    fn get_buy_amount(&self, diesel_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.calculate_buy_amount(diesel_amount)?.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the amount of diesel that can be received for selling a specific amount of tokens
    fn get_sell_amount(&self, token_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.calculate_sell_amount(token_amount)?.to_le_bytes().to_vec();
        
        Ok(response)
    }
}

impl BondingCurve for BondingContractAlkane {
    fn calculate_buy_price(&self, amount: u128) -> Result<u128> {
        self.calculate_buy_price(amount)
    }
    
    fn calculate_sell_price(&self, amount: u128) -> Result<u128> {
        self.calculate_sell_price(amount)
    }
    
    fn calculate_buy_amount(&self, diesel_amount: u128) -> Result<u128> {
        self.calculate_buy_amount(diesel_amount)
    }
    
    fn calculate_sell_amount(&self, token_amount: u128) -> Result<u128> {
        self.calculate_sell_amount(token_amount)
    }
    
    fn buy(&self, diesel_amount: u128) -> Result<CallResponse> {
        self.buy(diesel_amount)
    }
    
    fn sell(&self, token_amount: u128) -> Result<CallResponse> {
        self.sell(token_amount)
    }
    
    fn reserve(&self) -> u128 {
        self.reserve()
    }
    
    fn supply(&self) -> u128 {
        self.total_supply()
    }
    
    fn current_price(&self) -> Result<u128> {
        self.current_price()
    }
}

impl AlkaneResponder for BondingContractAlkane {
    fn context(&self) -> Result<Context> {
        // This would be implemented by the runtime
        // Simplified for example purposes
        Ok(Context::default())
    }
    
    fn execute(&self) -> Result<CallResponse> {
        // This method should not be called directly when using MessageDispatch
        Err(anyhow!("Use the declare_alkane macro instead"))
    }
}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for BondingContractAlkane {
        type Message = BondingContractAlkaneMessage;
    }
}

// Helper function to trim a u128 value to a String by removing trailing zeros
fn trim(v: u128) -> String {
    String::from_utf8(
        v.to_le_bytes()
            .into_iter()
            .fold(Vec::<u8>::new(), |mut r, v| {
                if v != 0 {
                    r.push(v)
                }
                r
            }),
    )
    .unwrap_or_default()
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

// Get the sell price for a specific amount
let get_sell_price_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        5,        // GetSellPrice opcode
        1000,     // Amount
    ],
};

// Get the amount of tokens that can be bought with a specific amount of diesel
let get_buy_amount_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        6,        // GetBuyAmount opcode
        1000,     // Diesel amount
    ],
};

// Get the amount of diesel that can be received for selling a specific amount of tokens
let get_sell_amount_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        7,        // GetSellAmount opcode
        1000,     // Token amount
    ],
};

// Get the name
let get_name_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        99,       // GetName opcode
    ],
};

// Get the symbol
let get_symbol_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        100,      // GetSymbol opcode
    ],
};

// Get the total supply
let get_total_supply_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        101,      // GetTotalSupply opcode
    ],
};

// Get the reserve
let get_reserve_cellpack = Cellpack {
    target: contract_id,
    inputs: vec![
        102,      // GetReserve opcode
    ],
};
```

## Example Scenario: Token Swap

Let's consider a scenario where a user wants to buy tokens with diesel:

```rust
// Deploy the bonding contract
let bonding_cellpack = Cellpack {
    target: AlkaneId { block: 1, tx: 0 },
    inputs: vec![
        0,              // Initialize opcode
        0x424f4e44,     // "BOND" as u128 (name)
        0x424e44,       // "BND" as u128 (symbol)
        1000000,        // Initial supply
        1000000,        // Initial reserve
    ],
};

// The bonding contract is deployed at address [2, bonding_sequence]

// User sends diesel to the contract
let diesel_transfer = AlkaneTransfer {
    id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
    value: 1000,
};

// User calls the buy function
let buy_cellpack = Cellpack {
    target: AlkaneId { block: 2, tx: bonding_sequence },
    inputs: vec![
        1,        // Buy opcode
    ],
};

// The user receives tokens based on the bonding curve
// The amount of tokens received depends on the current state of the curve
```

## Key Features

### Bonding Curve

The contract uses a quadratic bonding curve to determine the price of tokens:

```rust
fn calculate_buy_price(&self, amount: u128) -> Result<u128> {
    let supply = self.total_supply();
    let reserve = self.reserve();
    
    // Calculate the price using the integral of the price curve
    // For a quadratic curve: price = reserve * ((supply + amount)^2 - supply^2) / supply^2
    let new_supply = supply.checked_add(amount)
        .ok_or_else(|| anyhow!("supply overflow"))?;
    
    let new_supply_squared = new_supply.checked_mul(new_supply)
        .ok_or_else(|| anyhow!("calculation overflow"))?;
    
    let supply_squared = supply.checked_mul(supply)
        .ok_or_else(|| anyhow!("calculation overflow"))?;
    
    let supply_diff = new_supply_squared.checked_sub(supply_squared)
        .ok_or_else(|| anyhow!("calculation underflow"))?;
    
    let price = if supply_squared == 0 {
        // Initial price if supply is 0
        amount
    } else {
        reserve.checked_mul(supply_diff)
            .ok_or_else(|| anyhow!("calculation overflow"))?
            .checked_div(supply_squared)
            .ok_or_else(|| anyhow!("division by zero"))?
    };
    
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
