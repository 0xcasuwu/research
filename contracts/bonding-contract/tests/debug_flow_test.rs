use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use std::time::{SystemTime, UNIX_EPOCH};

use bonding_contract::{BondingContractAlkane, BondContract, AlkaneIdExt, Bond};
use bonding_contract::mock_runtime;
use bonding_contract::reset_mock_environment;

// Mock time function to control time in tests
fn set_mock_time(time: u64) {
    // This is just a placeholder - in a real implementation, we would need to
    // modify the contract to use this time instead of the system time
    println!("Setting mock time to: {}", time);
}

// Get the current mock time
fn get_mock_time() -> u64 {
    // For now, just return the current system time
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Debug version of create_bond_orbital that doesn't rely on the actual call
fn debug_create_bond_orbital(contract: &BondingContractAlkane, owed: u128, term: u64) -> AlkaneId {
    println!("DEBUG: Creating bond orbital with owed: {}, term: {}", owed, term);
    
    // Get the current block number
    let creation = contract.get_current_timestamp() / 10; // Convert timestamp to block number
    
    // Create a mock orbital ID
    let orbital_id = AlkaneId {
        block: 2, // Same as in the original implementation
        tx: 1000, // Use a fixed value for testing
    };
    
    println!("DEBUG: Created mock orbital ID: {:?}", orbital_id);
    
    orbital_id
}

#[test]
fn test_debug_bond_lifecycle() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours (1 day)
    
    // Manually set up the contract state instead of using init_bond_contract
    contract.set_name(name);
    contract.set_symbol(symbol);
    contract.set_virtual_input_reserves_internal(virtual_input_reserves);
    contract.set_virtual_output_reserves_internal(virtual_output_reserves);
    contract.set_half_life_internal(half_life);
    contract.set_level_bips_internal(level_bips);
    contract.set_term(term);
    contract.set_last_update_internal(contract.get_current_timestamp());
    contract.set_total_debt(0);
    contract.set_owner(caller.into_u128());
    contract.set_paused(false);
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // STEP 1: Debug the bond purchase process
    println!("STEP 1: Debugging bond purchase process at time {}", get_mock_time());
    
    // Set up the mock context for purchasing a bond
    let diesel_amount = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    println!("Purchasing bond with diesel amount: {}", diesel_amount);
    
    // Calculate the bond amount manually
    let mut bond_curve = contract.get_bond_curve();
    let available_debt = contract.available_debt();
    println!("Available debt: {}", available_debt);
    
    // Manually calculate the alkane amount
    let alkane_amount = bond_curve.purchase_bond(diesel_amount, available_debt);
    println!("Calculated alkane amount: {}", alkane_amount);
    
    // Update the total debt manually
    let current_debt = contract.total_debt();
    let new_debt = current_debt + alkane_amount;
    println!("Updating total debt: {} -> {}", current_debt, new_debt);
    contract.set_total_debt(new_debt);
    
    // Create a bond orbital using our debug function
    let orbital_id = debug_create_bond_orbital(&contract, alkane_amount, term);
    
    // Get the position count for the recipient
    let count = contract.position_count_of(caller.into_u128());
    println!("Current position count for {}: {}", caller.into_u128(), count);
    
    // Manually create a bond
    let bond = Bond {
        owed: alkane_amount,
        redeemed: 0,
        creation: (contract.get_current_timestamp() / 10) as u64, // Convert timestamp to block number
    };
    
    // Add the bond to the contract's storage
    // Note: We can't directly call add_bond since it's private, but in a real fix
    // we would need to modify the contract to expose this functionality or fix the
    // bond orbital creation issue
    
    // For now, we'll just verify that we've identified the issue
    println!("\nDEBUG RESULTS:");
    println!("1. Bond orbital creation is failing with 'failed to fill whole buffer'");
    println!("2. The issue is likely in the call to the orbital template");
    println!("3. Possible causes:");
    println!("   a. The orbital template ID is incorrect or not available");
    println!("   b. The buffer size for the call is insufficient");
    println!("   c. The mock environment doesn't properly support the call operation");
    println!("4. Recommended fixes:");
    println!("   a. Verify the orbital template ID is correct");
    println!("   b. Add more detailed error handling in the create_bond_orbital method");
    println!("   c. Create a mock implementation of the bond orbital for testing");
    
    println!("\nDebug bond lifecycle test completed!");
}
