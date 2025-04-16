# Bonding Contract Proposed Fixes

This document contains proposed fixes for the issues identified in the bonding contract implementation. These changes are not meant to be applied directly, but rather serve as a reference for future improvements.

## 1. Fix for `redeem_bond_internal` Function

Add debug output to understand why the redemption is failing:

```rust
/// Redeem a bond
fn redeem_bond_internal(&mut self, bond_id: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let mut response = CallResponse::forward(&context.incoming_alkanes);
    
    // Check if the contract is paused
    if self.is_paused() {
        println!("DEBUG: Redemption failed - contract is paused");
        return Err(anyhow!("contract is paused"));
    }
    
    // Get the caller
    let caller = context.caller.block;
    
    // Get the bond
    let bond = match self.get_bond(caller, bond_id) {
        Some(b) => b,
        None => {
            println!("DEBUG: Redemption failed - bond not found for caller {} and bond_id {}", caller, bond_id);
            return Err(anyhow!("bond not found"));
        }
    };
    
    // Check if the bond is fully redeemed
    if bond.owed <= bond.redeemed {
        println!("DEBUG: Redemption failed - bond already fully redeemed (owed: {}, redeemed: {})", bond.owed, bond.redeemed);
        return Err(anyhow!("bond already fully redeemed"));
    }
    
    // Check if the bond is mature
    let current_time = self.get_current_timestamp();
    let term = self.term();
    println!("DEBUG: Current time: {}, Bond creation: {}, Term: {}", current_time, bond.creation, term);
    if current_time < bond.creation + term {
        println!("DEBUG: Redemption failed - bond not yet mature");
        return Err(anyhow!("bond not yet mature"));
    }
    
    // Calculate the amount to redeem
    let available_debt = self.available_debt();
    let remaining = bond.owed - bond.redeemed;
    let to_redeem = std::cmp::min(remaining, available_debt);
    
    println!("DEBUG: Available debt: {}, Remaining: {}, To redeem: {}", available_debt, remaining, to_redeem);
    
    if to_redeem == 0 {
        println!("DEBUG: Redemption failed - no debt available for redemption");
        return Err(anyhow!("no debt available for redemption"));
    }
    
    // Update the bond
    let mut updated_bond = bond.clone();
    updated_bond.redeemed += to_redeem;
    self.update_bond(caller, bond_id, updated_bond);
    
    // Add the alkane to the response
    response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
        id: context.myself,
        value: to_redeem,
    });
    
    println!("DEBUG: Redemption successful - redeemed {} alkane", to_redeem);
    
    Ok(response)
}
```

## 2. Fix for `transfer_bond_internal` Function

Add debug output to understand why the transfer is not properly removing the bond from the original owner:

```rust
/// Transfer a bond to another address
fn transfer_bond_internal(&mut self, to: u128, bond_id: u128) -> Result<CallResponse> {
    let context = self.context()?;
    let response = CallResponse::forward(&context.incoming_alkanes);
    
    // Get the caller
    let caller = context.caller.block;
    
    println!("DEBUG: Transfer bond - caller: {}, to: {}, bond_id: {}", caller, to, bond_id);
    println!("DEBUG: Before transfer - caller position count: {}, recipient position count: {}", 
             self.position_count_of(caller), self.position_count_of(to));
    
    // Get the bond
    let bond = match self.get_bond(caller, bond_id) {
        Some(b) => b,
        None => {
            println!("DEBUG: Transfer failed - bond not found");
            return Err(anyhow!("bond not found"));
        }
    };
    
    println!("DEBUG: Bond details - owed: {}, redeemed: {}, creation: {}", 
             bond.owed, bond.redeemed, bond.creation);
    
    // Add the bond to the recipient
    self.add_bond(to, bond);
    
    println!("DEBUG: After adding bond to recipient - recipient position count: {}", 
             self.position_count_of(to));
    
    // Delete the bond from the caller
    self.delete_bond(caller, bond_id);
    
    println!("DEBUG: After deleting bond from caller - caller position count: {}", 
             self.position_count_of(caller));
    
    Ok(response)
}
```

## 3. Fix for `delete_bond` Function

The issue might be in the `delete_bond` function. Here's a potential fix with added debug output:

```rust
/// Delete a bond of an address
fn delete_bond(&self, address: u128, bond_id: u128) {
    let count = self.position_count_of(address);
    
    println!("DEBUG: Delete bond - address: {}, bond_id: {}, current count: {}", 
             address, bond_id, count);
    
    if bond_id >= count {
        println!("DEBUG: Bond ID out of range, nothing to delete");
        return;
    }
    
    // If it's the last bond, just decrease the count
    if bond_id == count - 1 {
        println!("DEBUG: Deleting last bond, decreasing count to {}", count - 1);
        self.set_position_count(address, count - 1);
        return;
    }
    
    // Otherwise, move the last bond to the deleted position
    let last_bond = match self.get_bond(address, count - 1) {
        Some(b) => b,
        None => {
            println!("DEBUG: Failed to get last bond");
            return;
        }
    };
    
    println!("DEBUG: Moving last bond to position {}", bond_id);
    self.update_bond(address, bond_id, last_bond);
    
    // Decrease the count
    println!("DEBUG: Decreasing count to {}", count - 1);
    self.set_position_count(address, count - 1);
    
    // Verify the deletion
    println!("DEBUG: After deletion - position count: {}", self.position_count_of(address));
}
```

## 4. Fix for `position_count_of` Function

Ensure the position count is being properly retrieved:

```rust
/// Get the position count of an address (internal method)
fn position_count_of_internal(&self, address: u128) -> u128 {
    let count = self.position_count_pointer(address).get_value::<u128>();
    println!("DEBUG: Position count for address {}: {}", address, count);
    count
}
```

## 5. Fix for `add_bond` Function

Ensure bonds are being properly added:

```rust
/// Add a bond to an address
fn add_bond(&self, address: u128, bond: Bond) {
    let pointer = self.bonds_pointer(address);
    let count = self.position_count_of(address);
    
    println!("DEBUG: Adding bond to address {} at position {}", address, count);
    println!("DEBUG: Bond details - owed: {}, redeemed: {}, creation: {}", 
             bond.owed, bond.redeemed, bond.creation);
    
    let bond_pointer = pointer.select(&count.to_le_bytes().to_vec());
    
    // Convert byte slices to Vec<u8> for select method
    bond_pointer.select(&b"owed".to_vec()).set_value::<u128>(bond.owed);
    bond_pointer.select(&b"redeemed".to_vec()).set_value::<u128>(bond.redeemed);
    bond_pointer.select(&b"creation".to_vec()).set_value::<u64>(bond.creation);
    
    // Update the count
    println!("DEBUG: Updating position count from {} to {}", count, count + 1);
    self.set_position_count(address, count + 1);
    
    // Verify the addition
    println!("DEBUG: After addition - position count: {}", self.position_count_of(address));
}
```

## 6. Fix for Test Environment

In the test environment, ensure the term is set to 0 and the bond creation timestamp is set correctly:

```rust
// In test_redeem_bond function
// Set term to 0 explicitly
contract.set_term(0);

// Print the term to verify
println!("Term set to: {}", contract.term());

// Purchase a bond
let diesel_amount = 1000;
let min_output = 1;
let to = caller.block;

// ... rest of the test ...

// Before redeeming, print the bond details
let bond = contract.get_bond(to, 0).unwrap();
println!("Bond details before redemption - owed: {}, redeemed: {}, creation: {}", 
         bond.owed, bond.redeemed, bond.creation);

// Get the current timestamp
let current_time = contract.get_current_timestamp();
println!("Current timestamp: {}", current_time);

// Now redeem the bond
let redeem_result = contract.redeem_bond(0);
```

These changes should help identify and fix the issues in the bonding contract implementation.
