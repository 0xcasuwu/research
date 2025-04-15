//! Isolated tests for the bonding contract
//!
//! This module contains isolated tests for the bonding contract functionality.

use crate::{BondingContractAlkane, BondingContract, BondContract};
use crate::reset_mock_environment;
use crate::mock_runtime;
use crate::mock_context;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use std::sync::Arc;

/// Test the initialization of the bonding contract
// Helper function to safely run a test with proper environment setup and teardown
fn run_test_with_isolation<F>(test_fn: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    // Reset the mock environment before the test
    reset_mock_environment::reset();
    
    // Run the test function in a catch_unwind to prevent test failures from affecting other tests
    let result = std::panic::catch_unwind(test_fn);
    
    // Reset the mock environment after the test regardless of success or failure
    reset_mock_environment::reset();
    
    // If the test panicked, resume the panic
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn test_init_contract() {
    run_test_with_isolation(|| {
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    // Initialize the contract
    let name = u128::from_le_bytes(*b"TestToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"TT\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let k_factor = 1000;
    let n_exponent = 2;
    let initial_diesel_reserve = 1000000;
    
    println!("Initializing contract with name: {}, symbol: {}, k_factor: {}, n_exponent: {}, initial_diesel_reserve: {}", 
             String::from_utf8_lossy(&name.to_le_bytes()), 
             String::from_utf8_lossy(&symbol.to_le_bytes()),
             k_factor, n_exponent, initial_diesel_reserve);
    
    let result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    if let Err(e) = &result {
        println!("Initialization failed: {}", e);
        });
}
    assert!(result.is_ok());
    
    // Check the contract state
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve);
    assert_eq!(contract.alkane_supply(), initial_diesel_reserve);
    assert_eq!(contract.name(), "TestToken");
    assert_eq!(contract.symbol(), "TT");
}

/// Test the initialization of the bond contract
#[test]
fn test_init_bond_contract() {
    run_test_with_isolation(|| {
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    // Initialize the contract
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BT\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1000000;
    let virtual_output_reserves = 2000000;
    let half_life = 86400; // 1 day in seconds
    let level_bips = 100; // 1%
    let term = 604800; // 1 week in seconds
    
    let result = contract.init_bond_contract(
        name, 
        symbol, 
        virtual_input_reserves, 
        virtual_output_reserves, 
        half_life, 
        level_bips, 
        term
    );
    assert!(result.is_ok());
    
    // Check the contract state
    assert_eq!(contract.name(), "BondToken");
    assert_eq!(contract.symbol(), "BT");
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves);
    assert_eq!(contract.virtual_output_reserves(), virtual_output_reserves);
    assert_eq!(contract.half_life(), half_life);
    assert_eq!(contract.level_bips(), level_bips);
    assert_eq!(contract.term(), term);
    assert_eq!(contract.total_debt(), 0);
    assert!(contract.is_paused());
    });
}

/// Test buying alkane with diesel
#[test]
fn test_buy_alkane() {
    run_test_with_isolation(|| {
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    // Initialize the contract
    let name = u128::from_le_bytes(*b"TestToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"TT\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let k_factor = 1000;
    let n_exponent = 2;
    let initial_diesel_reserve = 1000000;
    
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    let result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    assert!(result.is_ok());
    
    // Buy alkane with diesel
    let diesel_amount = 1000;
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![
            alkanes_support::parcel::AlkaneTransfer {
                id: diesel_id,
                value: diesel_amount,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    let result = contract.buy_alkane(diesel_amount);
    assert!(result.is_ok());
    
    // Check the contract state - the diesel reserve should increase by the amount provided
    let expected_diesel_reserve = initial_diesel_reserve + diesel_amount;
    let actual_diesel_reserve = contract.diesel_reserve();
    println!("Expected diesel reserve: {}, Actual diesel reserve: {}", expected_diesel_reserve, actual_diesel_reserve);
    assert_eq!(actual_diesel_reserve, expected_diesel_reserve);
    
    // Check the response
    let response = result.unwrap();
    println!("Response alkanes length: {}", response.alkanes.0.len());
    for (i, alkane) in response.alkanes.0.iter().enumerate() {
        println!("Alkane {}: id = {:?}, value = {}", i, alkane.id, alkane.value);
        });
}
    
    // The response should contain 2 alkanes:
    // 1. The original diesel token being returned
    // 2. The newly minted alkane token
    assert_eq!(response.alkanes.0.len(), 2, "Expected 2 alkane transfers in response");
    
    // Find the alkane token (the one with ID matching the contract's ID)
    let alkane_transfer = response.alkanes.0.iter()
        .find(|transfer| transfer.id == myself)
        .expect("Expected to find alkane transfer with contract ID");
    
    // Check that the alkane value is positive
    assert!(alkane_transfer.value > 0, "Expected alkane value to be positive");
    
    // Find the diesel token (the one with ID matching the diesel ID)
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    let diesel_transfer = response.alkanes.0.iter()
        .find(|transfer| transfer.id == diesel_id)
        .expect("Expected to find diesel transfer with diesel ID");
    
    // Check that the diesel value matches what we sent
    assert_eq!(diesel_transfer.value, diesel_amount, "Expected diesel value to match what we sent");
}

/// Test purchasing a bond
#[test]
fn test_purchase_bond() {
    run_test_with_isolation(|| {
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    // Initialize the contract
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BT\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1000000;
    let virtual_output_reserves = 2000000;
    let half_life = 86400; // 1 day in seconds
    let level_bips = 100; // 1%
    let term = 604800; // 1 week in seconds
    
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    let result = contract.init_bond_contract(
        name, 
        symbol, 
        virtual_input_reserves, 
        virtual_output_reserves, 
        half_life, 
        level_bips, 
        term
    );
    assert!(result.is_ok());
    
    // Set initial alkane supply
    contract.set_alkane_supply(1000000);
    
    // Unpause the contract
    contract.set_paused(false);
    
    // Purchase a bond
    let diesel_amount = 1000;
    let min_output = 1;
    let to = caller.block;
    
    let context = Context {
        caller,
        myself,
        incoming_alkanes: AlkaneTransfers(vec![
            alkanes_support::parcel::AlkaneTransfer {
                id: diesel_id,
                value: diesel_amount,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    // Set the context in both modules
    mock_context::set_mock_context(context.clone());
    mock_runtime::set_mock_context(context);
    
    let result = contract.purchase_bond(to, min_output);
    assert!(result.is_ok());
    
    // Check the contract state
    assert_eq!(contract.position_count_of(to), 1);
    
    // Get the bond
    let bond = contract.get_bond(to, 0).unwrap();
    assert_eq!(bond.redeemed, 0);
    assert!(bond.owed > min_output);
    
    // Check the total debt
    assert_eq!(contract.total_debt(), bond.owed);
    });
}
