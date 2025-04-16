# Bonding Contract Factory/Child Pattern Implementation

## Overview

The bonding contract has been redesigned to use the factory/child pattern, where each bond is represented by an orbital token. This architectural change provides significant improvements to the ownership model, authentication, and maturity calculation.

## Key Components

### BondOrbital

The `BondOrbital` struct represents a single bond as an orbital token. It contains the following key properties:

- **owed**: The amount of alkane owed to the bond holder
- **redeemed**: The amount of alkane already redeemed
- **creation**: The block number when the bond was created
- **term**: The term in blocks until the bond matures
- **bonding_contract_id**: The ID of the bonding contract that created the bond

### BondingContractAlkane as Factory

The `BondingContractAlkane` struct has been updated to act as a factory for bond orbitals. It includes the following new functionality:

- **create_bond_orbital**: Creates a new bond orbital with the specified parameters
- **get_bond_orbital_id**: Retrieves the orbital ID for a specific bond ID
- **set_bond_orbital_id**: Sets the orbital ID for a specific bond ID
- **redeem_orbital_internal**: Redeems a bond orbital

## Implementation Details

### Bond Creation

When a user purchases a bond, the following steps occur:

1. The bonding contract calculates the amount of alkane to be issued based on the diesel amount and the bond curve
2. The bonding contract creates a new bond orbital with the following parameters:
   - **owed**: The amount of alkane to be issued
   - **creation**: The current block number
   - **term**: The term in blocks until the bond matures
3. The bonding contract stores the orbital ID in its registry
4. The bonding contract transfers the orbital token to the user

```rust
// Create a bond orbital
let orbital_id = self.create_bond_orbital(alkane_amount, term)?;

// Store the orbital ID in the bond orbitals registry
self.set_bond_orbital_id(count, &orbital_id);

// Add the orbital to the response
response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
    id: orbital_id,
    value: 1u128, // Each orbital has a value of 1
});
```

### Bond Redemption

When a user redeems a bond, the following steps occur:

1. The user presents the bond orbital token to the bonding contract
2. The bonding contract verifies that the user has the correct orbital token
3. The bonding contract calls the orbital to get the bond details
4. The bonding contract verifies that the bond is mature and not fully redeemed
5. The bonding contract calculates the amount to redeem
6. The bonding contract transfers the alkane to the user

```rust
// Get the orbital ID for this bond
let orbital_id = self.get_bond_orbital_id(bond_id)
    .ok_or_else(|| anyhow!("bond orbital not found"))?;

// Check if the caller has the bond orbital token
let mut has_token = false;
for transfer in &context.incoming_alkanes.0 {
    if transfer.id == orbital_id && transfer.value >= 1 {
        has_token = true;
        break;
    }
}

if !has_token {
    return Err(anyhow!("Caller does not have the bond orbital token"));
}

// Call the redeem_orbital_internal method to redeem the bond
let redeem_response = self.redeem_orbital_internal(orbital_id.tx)?;
```

### Bond Transfer

When a user transfers a bond to another user, the following steps occur:

1. The user presents the bond orbital token to the bonding contract
2. The bonding contract verifies that the user has the correct orbital token
3. The bonding contract updates its registry to associate the orbital with the new owner
4. The bonding contract transfers the orbital token to the new owner

```rust
// Get the orbital ID for this bond
let orbital_id = self.get_bond_orbital_id(bond_id)
    .ok_or_else(|| anyhow!("bond orbital not found"))?;

// Check if the caller has the bond orbital token
let mut has_token = false;
for transfer in &context.incoming_alkanes.0 {
    if transfer.id == orbital_id && transfer.value >= 1 {
        has_token = true;
        break;
    }
}

if !has_token {
    return Err(anyhow!("Caller does not have the bond orbital token"));
}

// Get the position count for the recipient
let count = self.position_count_of(to);

// Store the orbital ID in the bond orbitals registry for the recipient
self.set_bond_orbital_id(count, &orbital_id);

// Increment the position count for the recipient
self.set_position_count(to, count + 1);

// Add the orbital to the response for the recipient
response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
    id: orbital_id,
    value: 1u128, // Each orbital has a value of 1
});
```

## Block-Based Maturity Calculation

The bond orbital uses block numbers instead of timestamps for maturity calculation. This provides several advantages:

1. More reliable and consistent than timestamp-based calculation
2. Less susceptible to manipulation
3. Easier to reason about and test

```rust
/// Get the current block number
fn get_current_block_number(&self) -> u64 {
    // Get the current block number from the context
    match self.context() {
        Ok(context) => context.myself.block as u64,
        Err(_) => {
            // Fallback to timestamp-based calculation if context is not available
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            // Convert seconds to blocks (assuming 1 block per 10 seconds)
            (now / 10) as u64
        }
    }
}

/// Get the maturity block number
pub fn maturity(&self) -> u64 {
    self.creation() + self.term()
}

/// Check if the bond is mature
pub fn is_mature(&self) -> bool {
    let current_block = self.get_current_block_number();
    current_block >= self.maturity()
}
```

## Benefits of the Factory/Child Pattern

1. **Improved Ownership Model**: Each bond is represented by a token, which can be transferred between users
2. **Simplified Authentication**: Users must present the bond orbital token to redeem the bond
3. **Enhanced Security**: The bond orbital token provides a secure way to authenticate bond ownership
4. **Better Maturity Calculation**: Block-based maturity calculation is more reliable and consistent
5. **Cleaner Code**: The factory/child pattern separates concerns and makes the code more maintainable

## Future Improvements

1. **Variable-Price Sales**: Support for bonds with variable prices based on market conditions
2. **Additional Bond Types**: Support for different types of bonds with different terms and yields
3. **Batch Operations**: Support for purchasing and redeeming multiple bonds in a single transaction
4. **Enhanced Registry**: Improved registry for tracking bond orbitals and their owners
5. **Performance Optimization**: Optimize storage and computation for large numbers of bonds
