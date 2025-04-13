//! Additional tests to achieve full coverage for the bonding contract

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use crate::mock_context::set_mock_context;
use crate::mock_runtime::clear_mock_storage;

// Helper function to create a context with incoming alkanes
fn create_context_with_alkanes(alkanes: Vec<AlkaneTransfer>) -> Context {
    Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: AlkaneTransferParcel(alkanes),
        vout: 0,
        inputs: vec![],
    }
}

// Helper function to initialize a contract for testing
fn init_test_contract() -> BondingContractAlkane {
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"TestToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"TEST";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1; // Linear bonding curve
    
    // Set up a default context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize the contract
    contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve).unwrap();
    
    contract
}

#[test]
fn test_buy_alkane_no_diesel_coverage() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Set up a context with no diesel
    let context = create_context_with_alkanes(vec![]);
    set_mock_context(context);
    
    // Try to buy alkane with no diesel in context
    // We're calling the method directly, not through the trait
    let result = contract.buy_alkane(1000);
    assert!(result.is_err(), "Buying alkane with no diesel should fail");
    assert_eq!(result.unwrap_err().to_string(), "no diesel received", 
               "Error message should indicate no diesel received");
}

#[test]
fn test_sell_alkane_no_alkane_coverage() {
    // Clear any previous state
    clear_mock_storage();

    // Create and initialize a contract
    let contract = init_test_contract();

    // Set up a context with no alkane
    let context = create_context_with_alkanes(vec![]);
    set_mock_context(context);

    // Try to sell alkane with no alkane in context
    // We're calling the method directly, not through the trait
    let result = contract.sell_alkane(1000);
    assert!(result.is_err(), "Selling alkane with no alkane should fail");
    
    // Get the error message
    let err_msg = result.unwrap_err().to_string();
    assert_eq!(err_msg, "no alkane received",
               "Error message should indicate no alkane received");
}
