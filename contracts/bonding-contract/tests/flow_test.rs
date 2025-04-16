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

#[deprecated(
    since = "0.2.0",
    note = "This test uses context-based testing which is deprecated. Use block-based testing instead (see block_based_flow_test.rs)."
)]
#[test]
fn test_bond_lifecycle_flow() {
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
    contract.set_paused(true);
    
    // Verify initial contract state
    assert_eq!(contract.name(), "BondToken", "Name should be set correctly");
    assert_eq!(contract.symbol(), "BOND", "Symbol should be set correctly");
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves, "Virtual input reserves should be set correctly");
    assert_eq!(contract.virtual_output_reserves(), virtual_output_reserves, "Virtual output reserves should be set correctly");
    assert_eq!(contract.half_life(), half_life, "Half life should be set correctly");
    assert_eq!(contract.level_bips(), level_bips, "Level bips should be set correctly");
    assert_eq!(contract.term(), term, "Term should be set correctly");
    assert_eq!(contract.total_debt(), 0, "Total debt should be initialized to 0");
    assert_eq!(contract.owner(), caller.into_u128(), "Owner should be set to the caller");
    assert!(contract.is_paused(), "Contract should be paused initially");
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Unpause the contract
    contract.set_paused(false);
    
    // Verify the contract is unpaused
    assert!(!contract.is_paused(), "Contract should be unpaused");
    
    // STEP 1: Purchase a bond
    println!("STEP 1: Purchasing bond at time {}", get_mock_time());
    
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
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    println!("Calling purchase_bond with caller: {}, min_output: {}", caller.into_u128(), min_output);
    
    // Make sure the context is set right before calling purchase_bond
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
    
    mock_runtime::set_mock_context(context.clone());
    
    // Now call purchase_bond
    let purchase_result = contract.purchase_bond(caller.into_u128(), min_output);
    
    println!("Purchase result: {:?}", purchase_result);
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
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), bond.owed, "Total debt should match bond owed");
    
    // Verify the virtual input reserves were updated
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves + diesel_amount, 
        "Virtual input reserves should be updated");
    
    // After purchase, we can verify the bond was created successfully
    // We don't need to check the price directly
    
    // STEP 2: Simulate time passing (25% of term)
    println!("\nSTEP 2: Simulating time passing (25% of term)");
    let quarter_term = term / 4;
    set_mock_time(initial_time + quarter_term);
    println!("Time elapsed: {} seconds (25% of term)", quarter_term);
    
    // Calculate expected redeemable amount based on bond properties and elapsed time
    let elapsed = quarter_term as u128;
    let term_u128 = term as u128;
    let expected_redeemable = bond.owed * elapsed / term_u128;
    println!("Expected redeemable amount after 25% of term: {}", expected_redeemable);
    
    // STEP 3: Redeem part of the bond at 25% maturity
    println!("\nSTEP 3: Redeeming bond at 25% maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let actual_redeemed = response.alkanes.0[0].value;
    println!("Actual redeemed amount: {}", actual_redeemed);
    
    // Verify the redeemed amount is approximately what we expect
    let tolerance = bond.owed / 100; // 1% tolerance
    assert!(
        actual_redeemed >= expected_redeemable - tolerance && 
        actual_redeemed <= expected_redeemable + tolerance,
        "Redeemed amount should be approximately what we expect"
    );
    
    // Verify the bond was updated
    let updated_bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    assert_eq!(updated_bond.redeemed, actual_redeemed, "Bond redeemed should match actual redeemed amount");
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), bond.owed - actual_redeemed, 
        "Total debt should be reduced by redeemed amount");
    
    // STEP 4: Simulate time passing to full maturity
    println!("\nSTEP 4: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // Calculate expected redeemable amount at full maturity
    let expected_redeemable = bond.owed - updated_bond.redeemed;
    println!("Expected redeemable amount at full maturity: {}", expected_redeemable);
    
    // STEP 5: Redeem the rest of the bond at full maturity
    println!("\nSTEP 5: Redeeming the rest of the bond at full maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let actual_redeemed = response.alkanes.0[0].value;
    println!("Actual redeemed amount: {}", actual_redeemed);
    
    // Verify the redeemed amount is approximately what we expect
    let tolerance = bond.owed / 100; // 1% tolerance
    assert!(
        actual_redeemed >= expected_redeemable - tolerance && 
        actual_redeemed <= expected_redeemable + tolerance,
        "Redeemed amount should be approximately what we expect"
    );
    
    // Verify the bond was updated
    let final_bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    assert_eq!(final_bond.redeemed, bond.owed, "Bond should be fully redeemed");
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), 0, "Total debt should be 0");
    
    // STEP 6: Try to redeem again (should return 0)
    println!("\nSTEP 6: Trying to redeem again (should return 0)");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed but return 0");
    
    // Verify the response contains no alkane transfer or 0 value
    let response = redeem_result.unwrap();
    if !response.alkanes.0.is_empty() {
        assert_eq!(response.alkanes.0[0].value, 0, "Transfer value should be 0");
    }
    
    // Verify the bond remains fully redeemed
    let final_bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    assert_eq!(final_bond.redeemed, bond.owed, "Bond should still be fully redeemed");
    
    // Verify the total debt remains 0
    assert_eq!(contract.total_debt(), 0, "Total debt should still be 0");
    
    println!("\nBond lifecycle test completed successfully!");
}

#[deprecated(
    since = "0.2.0",
    note = "This test uses context-based testing which is deprecated. Use block-based testing instead (see block_based_flow_test.rs)."
)]
#[test]
fn test_multiple_bonds_lifecycle() {
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
    contract.set_paused(true);
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Unpause the contract
    contract.set_paused(false);
    
    // Verify the contract is unpaused
    assert!(!contract.is_paused(), "Contract should be unpaused");
    
    // STEP 1: Purchase first bond
    println!("STEP 1: Purchasing first bond at time {}", get_mock_time());
    
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
    println!("Calling purchase_bond with caller: {}, min_output: {}", caller.into_u128(), min_output);
    
    // Make sure the context is set right before calling purchase_bond
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
    
    mock_runtime::set_mock_context(context.clone());
    
    // Now call purchase_bond
    let purchase_result_1 = contract.purchase_bond(caller.into_u128(), min_output);
    
    println!("Purchase result: {:?}", purchase_result_1);
    assert!(purchase_result_1.is_ok(), "First bond purchase should succeed");
    
    // Get the first bond
    let bond_1 = contract.get_bond(caller.into_u128(), 0).unwrap();
    println!("First bond created with owed: {}", bond_1.owed);
    
    // STEP 2: Simulate time passing (50% of term)
    println!("\nSTEP 2: Simulating time passing (50% of term)");
    let half_term = term / 2;
    set_mock_time(initial_time + half_term);
    println!("Time elapsed: {} seconds (50% of term)", half_term);
    
    // STEP 3: Purchase second bond at higher price
    println!("\nSTEP 3: Purchasing second bond at time {}", get_mock_time());
    
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
    
    // Verify the second bond gives less alkane for the same diesel amount
    assert!(bond_2.owed < bond_1.owed, 
        "Second bond should give less alkane for the same diesel amount due to price increase");
    
    // STEP 4: Redeem first bond at 50% maturity
    println!("\nSTEP 4: Redeeming first bond at 50% maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Calculate expected redeemable amount for first bond at 50% maturity
    let half_term_u128 = half_term as u128;
    let term_u128 = term as u128;
    let expected_redeemable_1 = bond_1.owed * half_term_u128 / term_u128;
    println!("Expected redeemable amount for first bond at 50% maturity: {}", expected_redeemable_1);
    
    // Redeem the first bond
    let redeem_result_1 = contract.redeem_bond(0);
    
    assert!(redeem_result_1.is_ok(), "First bond redemption should succeed");
    
    // Get the updated first bond
    let updated_bond_1 = contract.get_bond(caller.into_u128(), 0).unwrap();
    println!("First bond updated with redeemed: {}", updated_bond_1.redeemed);
    
    // STEP 5: Simulate time passing to full maturity
    println!("\nSTEP 5: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 6: Redeem both bonds
    println!("\nSTEP 6: Redeeming both bonds at full maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Calculate expected remaining redeemable amounts
    let remaining_redeemable_1 = bond_1.owed - updated_bond_1.redeemed;
    let term_u128 = term as u128;
    let expected_redeemable_2 = bond_2.owed * term_u128 / term_u128; // 100% maturity
    println!("Expected remaining redeemable amount for first bond: {}", remaining_redeemable_1);
    println!("Expected redeemable amount for second bond: {}", expected_redeemable_2);
    
    // Redeem both bonds using batch redemption
    let redeem_result = contract.redeem_bond_batch(vec![0, 1]);
    
    assert!(redeem_result.is_ok(), "Batch bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    assert_eq!(response.alkanes.0[0].value, remaining_redeemable_1 + expected_redeemable_2, 
        "Transfer value should match sum of redeemable amounts");
    
    // Verify the bonds were updated
    let final_bond_1 = contract.get_bond(caller.into_u128(), 0).unwrap();
    let final_bond_2 = contract.get_bond(caller.into_u128(), 1).unwrap();
    assert_eq!(final_bond_1.redeemed, bond_1.owed, "First bond should be fully redeemed");
    assert_eq!(final_bond_2.redeemed, bond_2.owed, "Second bond should be fully redeemed");
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), 0, "Total debt should be 0");
    
    println!("\nMultiple bonds lifecycle test completed successfully!");
}
