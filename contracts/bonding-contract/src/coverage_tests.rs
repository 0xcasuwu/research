//! Coverage tests for the bonding contract
//!
//! This module contains tests specifically designed to improve code coverage
//! by testing functions that are not covered by other tests.

use crate::{BondingContractAlkane, BondingContract, BondContract, Bond};
use crate::reset_mock_environment;
use crate::mock_runtime;
use crate::mock_context;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::context::Context;
use alkanes_runtime::storage::StoragePointer;
use metashrew_support::index_pointer::KeyValuePointer;
use std::time::{SystemTime, UNIX_EPOCH};

// Helper function to reset the contract state
fn reset_contract_state() -> BondingContractAlkane {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Reset the initialization state
    let mut init_pointer = StoragePointer::from_keyword("/initialized");
    init_pointer.set_value::<u8>(0);
    
    // Create a new contract instance with default values
    BondingContractAlkane::default()
}

// Helper function to safely run a test with proper environment setup and teardown
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


/// Test redeeming a bond
#[test]
fn test_redeem_bond() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
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
        let term = 1; // 1 second for testing
        
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
                AlkaneTransfer {
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
        
        // Wait for the bond to mature (sleep for term + 1 second)
        std::thread::sleep(std::time::Duration::from_secs(term + 1));
        
        // Redeem the bond
        let bond_id = 0;
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
        
        let result = contract.redeem_bond(bond_id);
        assert!(result.is_ok());
        
        // Check the response
        let response = result.unwrap();
        
        // The response should contain the alkane token
        let alkane_transfer = response.alkanes.0.iter()
            .find(|transfer| transfer.id == myself)
            .expect("Expected to find alkane transfer");
        
        // Check that the alkane value is positive
        assert!(alkane_transfer.value > 0, "Expected alkane value to be positive");
        
        // Check the bond state
        let bond = contract.get_bond(to, bond_id).unwrap();
        assert_eq!(bond.redeemed, bond.owed);
    });
}

/// Test redeeming multiple bonds in batch
#[test]
fn test_redeem_bond_batch() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
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
        let term = 1; // 1 second for testing
        
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
        
        // Purchase multiple bonds
        let to = caller.block;
        
        // Purchase first bond
        let diesel_amount = 1000;
        let min_output = 1;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Purchase second bond
        let diesel_amount = 2000;
        let min_output = 1;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Wait for the bonds to mature (sleep for term + 1 second)
        std::thread::sleep(std::time::Duration::from_secs(term + 1));
        
        // Redeem the bonds in batch
        let bond_ids = vec![0, 1];
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
        
        let result = contract.redeem_bond_batch(bond_ids);
        assert!(result.is_ok());
        
        // Check the response
        let response = result.unwrap();
        
        // The response should contain the alkane token
        let alkane_transfer = response.alkanes.0.iter()
            .find(|transfer| transfer.id == myself)
            .expect("Expected to find alkane transfer");
        
        // Check that the alkane value is positive
        assert!(alkane_transfer.value > 0, "Expected alkane value to be positive");
        
        // Check the bond states
        let bond0 = contract.get_bond(to, 0).unwrap();
        let bond1 = contract.get_bond(to, 1).unwrap();
        assert_eq!(bond0.redeemed, bond0.owed);
        assert_eq!(bond1.redeemed, bond1.owed);
    });
}

/// Test transferring a bond to another address
#[test]
fn test_transfer_bond() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
        // Set up the context
        let caller = AlkaneId { block: 1, tx: 0 };
        let recipient = AlkaneId { block: 2, tx: 1 };
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
                AlkaneTransfer {
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
        
        // Transfer the bond
        let bond_id = 0;
        let recipient_address = recipient.block;
        
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
        
        let result = contract.transfer_bond(recipient_address, bond_id);
        assert!(result.is_ok());
        
        // Check the bond ownership
        assert_eq!(contract.position_count_of(to), 0);
        assert_eq!(contract.position_count_of(recipient_address), 1);
        
        // Check the bond details
        let bond = contract.get_bond(recipient_address, 0).unwrap();
        assert_eq!(bond.redeemed, 0);
        assert!(bond.owed > min_output);
    });
}

/// Test getting the current price
#[test]
fn test_get_current_price() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
        // Set up the context
        let caller = AlkaneId { block: 1, tx: 0 };
        let myself = AlkaneId { block: 3, tx: 0 };
        
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
        
        // Get the current price
        let result = contract.current_price();
        assert!(result.is_ok());
        
        // Check the response
        let response = result.unwrap();
        
        // The response should contain the price data
        assert!(!response.data.is_empty());
        
        // Convert the data to a u128
        let price = u128::from_le_bytes(response.data[0..16].try_into().unwrap());
        
        // Check that the price is positive
        assert!(price > 0, "Expected price to be positive");
    });
}

/// Test getting the available debt
#[test]
fn test_available_debt() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
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
        let initial_supply = 1000000;
        contract.set_alkane_supply(initial_supply);
        
        // Unpause the contract
        contract.set_paused(false);
        
        // Check the initial available debt
        let initial_available_debt = contract.available_debt();
        assert_eq!(initial_available_debt, initial_supply);
        
        // Purchase a bond
        let diesel_amount = 1000;
        let min_output = 1;
        let to = caller.block;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Get the bond
        let bond = contract.get_bond(to, 0).unwrap();
        
        // Check the available debt after purchasing a bond
        let expected_available_debt = initial_supply - bond.owed;
        let actual_available_debt = contract.available_debt();
        assert_eq!(actual_available_debt, expected_available_debt);
    });
}

/// Test management functions (set_virtual_input_reserves, set_virtual_output_reserves, set_half_life, set_level_bips)
#[test]
fn test_management_functions() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
        // Set up the context
        let caller = AlkaneId { block: 1, tx: 0 };
        let myself = AlkaneId { block: 3, tx: 0 };
        
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
        
        // Test set_virtual_input_reserves_internal
        let new_virtual_input_reserves = 2000000;
        contract.set_virtual_input_reserves_internal(new_virtual_input_reserves);
        assert_eq!(contract.virtual_input_reserves(), new_virtual_input_reserves);
        
        // Test set_virtual_output_reserves_internal
        let new_virtual_output_reserves = 3000000;
        contract.set_virtual_output_reserves_internal(new_virtual_output_reserves);
        assert_eq!(contract.virtual_output_reserves(), new_virtual_output_reserves);
        
        // Test set_half_life_internal
        let new_half_life = 172800; // 2 days in seconds
        contract.set_half_life_internal(new_half_life);
        assert_eq!(contract.half_life(), new_half_life);
        
        // Test set_level_bips_internal
        let new_level_bips = 200; // 2%
        contract.set_level_bips_internal(new_level_bips);
        assert_eq!(contract.level_bips(), new_level_bips);
        
        // Test set_last_update_internal
        let new_last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        contract.set_last_update_internal(new_last_update);
        assert_eq!(contract.last_update(), new_last_update);
    });
}

/// Test deleting a bond
#[test]
fn test_delete_bond() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
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
        
        // Purchase multiple bonds
        let to = caller.block;
        
        // Purchase first bond
        let diesel_amount = 1000;
        let min_output = 1;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Purchase second bond
        let diesel_amount = 2000;
        let min_output = 1;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Check the initial position count
        assert_eq!(contract.position_count_of(to), 2);
        
        // Get the second bond details
        let bond1 = contract.get_bond(to, 1).unwrap();
        
        // Delete the first bond
        contract.delete_bond(to, 0);
        
        // Check the position count after deletion
        assert_eq!(contract.position_count_of(to), 1);
        
        // Check that the second bond is now at position 0
        let moved_bond = contract.get_bond(to, 0).unwrap();
        assert_eq!(moved_bond.owed, bond1.owed);
        assert_eq!(moved_bond.redeemed, bond1.redeemed);
        assert_eq!(moved_bond.creation, bond1.creation);
        
        // Delete the last bond
        contract.delete_bond(to, 0);
        
        // Check that the position count is now 0
        assert_eq!(contract.position_count_of(to), 0);
    });
}

/// Test getting the bond amount
#[test]
fn test_get_bond_amount() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
        // Set up the context
        let caller = AlkaneId { block: 1, tx: 0 };
        let myself = AlkaneId { block: 3, tx: 0 };
        
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
        
        // Get the bond curve
        let mut curve = contract.get_bond_curve();
        
        // Calculate the bond amount for a specific diesel amount
        let diesel_amount = 1000;
        let available_debt = contract.available_debt();
        let expected_bond_amount = curve.purchase_bond(diesel_amount, available_debt);
        
        // Check that the bond amount is positive
        assert!(expected_bond_amount > 0, "Expected bond amount to be positive");
    });
}

/// Test pausing and unpausing the contract
#[test]
fn test_pause_unpause() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
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
        
        // Check that the contract is initially paused
        assert!(contract.is_paused());
        
        // Unpause the contract
        contract.set_paused(false);
        
        // Check that the contract is now unpaused
        assert!(!contract.is_paused());
        
        // Try to purchase a bond (should succeed now that the contract is unpaused)
        let diesel_amount = 1000;
        let min_output = 1;
        let to = caller.block;
        
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        
        // Pause the contract again
        contract.set_paused(true);
        
        // Check that the contract is now paused
        assert!(contract.is_paused());
        
        // Try to purchase a bond (should fail now that the contract is paused)
        let context = Context {
            caller,
            myself,
            incoming_alkanes: AlkaneTransfers(vec![
                AlkaneTransfer {
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
        assert!(result.is_err());
    });
}

/// Test getting the buy amount
#[test]
fn test_get_buy_amount() {
    run_test_with_isolation(|| {
        // Create a new bonding contract with reset state
        let mut contract = reset_contract_state();
        
        // Set up the context
        let caller = AlkaneId { block: 1, tx: 0 };
        let myself = AlkaneId { block: 3, tx: 0 };
        
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
        
        // Get the buy amount for a specific diesel amount
        let diesel_amount = 1000;
        let result = contract.get_buy_amount(diesel_amount);
        assert!(result.is_ok());
        
        // Check the response
        let response = result.unwrap();
        
        // The response should contain the buy amount data
        assert!(!response.data.is_empty());
        
        // Convert the data to a u128
        let buy_amount = u128::from_le_bytes(response.data[0..16].try_into().unwrap());
        
        // Check that the buy amount is positive
        assert!(buy_amount > 0, "Expected buy amount to be positive");
    });
}
