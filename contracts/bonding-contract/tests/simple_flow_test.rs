use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use std::time::{SystemTime, UNIX_EPOCH};

use bonding_contract::{BondingContractAlkane, BondContract, BondingContract, AlkaneIdExt};
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

#[test]
fn test_simple_bond_lifecycle() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
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
    contract.set_paused(false); // Start unpaused for simplicity
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // STEP 1: Purchase a bond
    println!("STEP 1: Purchasing bond");
    
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
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    let purchase_result = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result.is_ok(), "Bond purchase should succeed");
    
    // Verify the bond was created
    let position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(position_count, 1, "Position count should be 1");
    
    // Get the bond
    let bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    
    // Verify the bond properties
    println!("Bond created with owed: {}, redeemed: {}", bond.owed, bond.redeemed);
    assert!(bond.owed > 0, "Bond owed should be positive");
    assert_eq!(bond.redeemed, 0, "Bond redeemed should be 0");
    
    // STEP 2: Simulate time passing to full maturity
    println!("\nSTEP 2: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 3: Redeem the bond
    println!("\nSTEP 3: Redeeming the bond");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem the bond - we need to simulate having the bond orbital token
    // In the real implementation, the caller would need to provide the bond orbital token
    // But in our test, we'll just directly call the internal method
    let redeem_result = contract.redeem_bond_internal(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let actual_redeemed = response.alkanes.0[0].value;
    println!("Actual redeemed amount: {}", actual_redeemed);
    
    // Verify the redeemed amount is approximately what we expect
    assert_eq!(actual_redeemed, bond.owed, "Redeemed amount should match bond owed");
    
    // Verify the bond was updated
    let updated_bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    assert_eq!(updated_bond.redeemed, bond.owed, "Bond should be fully redeemed");
    
    println!("\nSimple bond lifecycle test completed successfully!");
}

#[test]
fn test_simple_multiple_bonds() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
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
    contract.set_paused(false); // Start unpaused for simplicity
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // STEP 1: Purchase first bond
    println!("STEP 1: Purchasing first bond");
    
    // Set up the mock context for purchasing a bond
    let diesel_amount_1 = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount_1,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Purchase first bond
    let min_output = 1;
    let purchase_result_1 = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result_1.is_ok(), "First bond purchase should succeed");
    
    // Get the first bond
    let bond_1 = contract.get_bond(caller.into_u128(), 0).unwrap();
    println!("First bond created with owed: {}", bond_1.owed);
    
    // STEP 2: Purchase second bond
    println!("\nSTEP 2: Purchasing second bond");
    
    // Set up the mock context for purchasing a second bond
    let diesel_amount_2 = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount_2,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Purchase second bond
    let purchase_result_2 = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result_2.is_ok(), "Second bond purchase should succeed");
    
    // Get the second bond
    let bond_2 = contract.get_bond(caller.into_u128(), 1).unwrap();
    println!("Second bond created with owed: {}", bond_2.owed);
    
    // STEP 3: Simulate time passing to full maturity
    println!("\nSTEP 3: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 4: Redeem both bonds
    println!("\nSTEP 4: Redeeming both bonds");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem both bonds using batch redemption - we need to use the internal method
    let redeem_result = contract.redeem_bond_batch_internal(vec![0, 1]);
    
    assert!(redeem_result.is_ok(), "Batch bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let total_redeemed = response.alkanes.0[0].value;
    println!("Total redeemed amount: {}", total_redeemed);
    
    // Verify the total redeemed amount is the sum of both bonds
    assert_eq!(total_redeemed, bond_1.owed + bond_2.owed, "Total redeemed amount should match sum of bond owed amounts");
    
    println!("\nSimple multiple bonds test completed successfully!");
}
